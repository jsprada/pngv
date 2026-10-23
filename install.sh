#!/usr/bin/env bash
# Build pngv in release mode and symlink it into /usr/local/bin.
set -euo pipefail

cd "$(dirname "$(realpath "$0")")"

cargo build --release

bin="$PWD/target/release/pngv"
link=/usr/local/bin/pngv

if [ -w /usr/local/bin ]; then
    ln -sfn "$bin" "$link"
else
    sudo ln -sfn "$bin" "$link"
fi

echo "Linked $link -> $bin"
