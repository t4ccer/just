# `justwindows`

Lightweight X11 window manager.

## Developing

```console
Xephyr :1 -resizeable
DISPLAY=:1 cargo run --package justwindows --release
```

## Usage

You can compile `justwindows` to a single statically linked ELF file with no runtime dependencies (other than running X11 server) and share it with your friends or distribute however you wish.

```console
cargo build --package justwindows --release --target x86_64-unknown-linux-musl
```
