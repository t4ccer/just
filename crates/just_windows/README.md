# `just_windows`

Lightweight X11 window manager.

> :building_construction: **VERY** in progress, do not expect it to work.


## Developing

```console
Xephyr :1 -resizeable
DISPLAY=:1 cargo run --package justwindows --release
```

## Usage

You can compile `just_windows` to a single statically linked ELF file with no runtime dependencies (other than running X11 server) and share it with your friends or distribute however you wish.

```console
cargo build --package just_windows --release --target x86_64-unknown-linux-musl
```
