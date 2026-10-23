use image::{imageops, imageops::FilterType, RgbaImage};
use softbuffer::{Context, Surface};
use std::{env, num::NonZeroU32, process, rc::Rc};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const HELP: &str = "\
Usage: pngv [OPTIONS] <file.png>

Display a PNG image in a window.

Options:
  --fit       Scale the image to fit the screen, keeping its proportions (default)
  --full      Show the image at full size (1 image pixel = 1 screen pixel)
  -h, --help  Show this help and exit

Keys:
  Esc, q      Close the window";

fn die(msg: &str) -> ! {
    eprintln!("pngv: {msg}");
    process::exit(1);
}

/// Size of a `w`x`h` image scaled, keeping proportions, until it touches an edge of `max_w`x`max_h`.
fn fit(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64);
    let dim = |n: u32| ((n as f64 * scale).round() as u32).max(1);
    (dim(w), dim(h))
}

struct App {
    path: String,
    full: bool,
    img: RgbaImage,
    size: PhysicalSize<u32>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    frame: Option<(PhysicalSize<u32>, Vec<u32>)>,
}

impl App {
    /// Render the image centered on black into a `ww`x`wh` buffer of 0RGB pixels.
    fn render(&self, ww: u32, wh: u32) -> Vec<u32> {
        let (w, h) = self.img.dimensions();
        let (sw, sh) = if self.full { (w, h) } else { fit(w, h, ww, wh) };
        let scaled = if (sw, sh) == (w, h) {
            self.img.clone()
        } else {
            imageops::resize(&self.img, sw, sh, FilterType::Triangle)
        };

        let (ox, oy) = ((ww as i64 - sw as i64) / 2, (wh as i64 - sh as i64) / 2);
        let mut out = vec![0u32; (ww * wh) as usize];
        for y in 0..wh {
            let sy = y as i64 - oy;
            if sy < 0 || sy >= sh as i64 {
                continue;
            }
            for x in 0..ww {
                let sx = x as i64 - ox;
                if sx < 0 || sx >= sw as i64 {
                    continue;
                }
                // Blend alpha over black.
                let [r, g, b, a] = scaled.get_pixel(sx as u32, sy as u32).0.map(u32::from);
                out[(y * ww + x) as usize] =
                    ((r * a / 255) << 16) | ((g * a / 255) << 8) | (b * a / 255);
            }
        }
        out
    }

    fn draw(&mut self) {
        let Some(window) = self.window.clone() else { return };
        let size = window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return;
        };
        if self.frame.as_ref().map(|f| f.0) != Some(size) {
            self.frame = Some((size, self.render(size.width, size.height)));
        }

        let surface = self.surface.as_mut().unwrap();
        surface.resize(w, h).unwrap();
        let mut buf = surface.buffer_mut().unwrap();
        buf.copy_from_slice(&self.frame.as_ref().unwrap().1);
        buf.present().unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let (w, h) = self.img.dimensions();
        let monitor = el.primary_monitor().or_else(|| el.available_monitors().next());
        if let (false, Some(m)) = (self.full, monitor) {
            let screen = m.size();
            let (fw, fh) = fit(w, h, screen.width, screen.height);
            self.size = PhysicalSize::new(fw, fh);
        }

        let attrs = Window::default_attributes()
            .with_title(&self.path)
            .with_inner_size(self.size)
            .with_min_inner_size(self.size)
            .with_max_inner_size(self.size);
        let window = Rc::new(el.create_window(attrs).unwrap_or_else(|e| die(&e.to_string())));
        let ctx = Context::new(window.clone()).unwrap_or_else(|e| die(&e.to_string()));
        self.surface = Some(Surface::new(&ctx, window.clone()).unwrap_or_else(|e| die(&e.to_string())));
        self.window = Some(window);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key, state: ElementState::Pressed, .. },
                ..
            } => match logical_key {
                Key::Named(NamedKey::Escape) => el.exit(),
                Key::Character(c) if c == "q" => el.exit(),
                _ => {}
            },
            // Keep the window's size in real screen pixels on scaled (HiDPI) displays. Size hints are
            // used because compositors like Hyprland ignore plain resize requests from clients.
            WindowEvent::ScaleFactorChanged { mut inner_size_writer, .. } => {
                let window = self.window.as_ref().unwrap();
                window.set_min_inner_size(Some(self.size));
                window.set_max_inner_size(Some(self.size));
                let _ = inner_size_writer.request_inner_size(self.size);
            }
            WindowEvent::Resized(_) => self.window.as_ref().unwrap().request_redraw(),
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
    }
}

fn main() {
    let mut full = false;
    let mut path = None;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return;
            }
            "--fit" => full = false,
            "--full" => full = true,
            s if s.starts_with('-') => die(&format!("unknown option '{s}'\nTry 'pngv --help'.")),
            _ if path.is_none() => path = Some(arg),
            _ => die("only one file may be given\nTry 'pngv --help'."),
        }
    }
    let path = path.unwrap_or_else(|| die("no file given\nTry 'pngv --help'."));

    let img = image::open(&path)
        .unwrap_or_else(|e| die(&format!("{path}: {e}")))
        .to_rgba8();
    let size = PhysicalSize::new(img.width(), img.height());

    let event_loop = EventLoop::new().unwrap_or_else(|e| die(&e.to_string()));
    let mut app = App { path, full, img, size, window: None, surface: None, frame: None };
    event_loop.run_app(&mut app).unwrap_or_else(|e| die(&e.to_string()));
}
