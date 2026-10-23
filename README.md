# pngv

A minimal PNG viewer for the desktop, written in Rust. Give it a file and it opens the image in a window.

## Usage

```
pngv [OPTIONS] <file.png>
```

| Option | Description |
| --- | --- |
| `--fit` | Scale the image to fit the screen, keeping its proportions (default) |
| `--full` | Show the image at full size (1 image pixel = 1 screen pixel) |
| `-h`, `--help` | Show help and exit |

Press `Esc` or `q` to close the window.

The window is sized to the image and can't be resized. In `--fit` mode, the image is scaled up or down until it touches an edge of the screen. Transparent pixels are drawn over black.

## Install

Built with Rust 1.98.1.

```sh
git clone https://github.com/jsprada/pngv.git
cd pngv
./install.sh
```

`install.sh` builds a release binary and symlinks it to `/usr/local/bin/pngv`. It uses `sudo` if `/usr/local/bin` isn't writable.

To build without installing, run `cargo build --release`. The binary ends up at `target/release/pngv`.

## License

[MIT](LICENSE)
