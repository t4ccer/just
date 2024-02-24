# `justshow`

The top level crate is just a placeholder for now for the common interface library.

Zero dependency (only stdlib) graphics library.

## Roadmap

- [ ] Underlying graphics backends
  - [ ] X11
    - [x] Raw interface (100% of the [standard](https://www.x.org/releases/X11R7.7/doc/xproto/x11protocol.html))
    - [ ] Friendly interface (Nice types for bitmasks, etc. tracking resources with lifetimes)
    - [ ] Extensions
      - [ ] MIT-SHM
      - [ ] XINERAMA
      - [ ] GLX
  - [ ] Wayland
  - [ ] WinAPI
  - [ ] Web/Canvas?
  - [ ] Mobile?
- [ ] Sound
- [ ] Friendly backend-agnostic wrapper

## Examples

### X11 Window Manager

[justwindows](./crates/justwindows)

### Pong

```console
cargo run --release --package justshow_x11 --bin pong
```

Controls:

- Quit: `q`
- Left player:`d/f`
- Right player: `j/k`
- Restart: `r`


[Source](./crates/justshow_x11_simple/src/bin/pong.rs)

![preview](./img/pong.png)

### `xlsfonts` clone

[Original implementation](https://gitlab.freedesktop.org/xorg/app/xlsfonts)

```console
cargo run --release --package justshow_x11 --bin xinfo ls fonts
```

[Source](./crates/justshow_x11/src/bin/xinfo.rs)
