# `justshow_x11`

Lightweight X11 client

## Examples

### `xlsfonts` clone

[Original implementation](https://gitlab.freedesktop.org/xorg/app/xlsfonts)

```console
cargo run --release --package just_x11 --example xinfo -- ls fonts
```

[Source](./examples/xinfo.rs)

### MIT-SHM Screenshot utility

[Original implementation](https://gist.github.com/rexim/2febe9f5a5376b476d33d5d16590ecfd)

```console
cargo run --release --package just_x11 --example screenshot_shm
```

[Source](./examples/screenshot_shm.rs)
