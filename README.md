# `justshow`

Zero dependency (only stdlib) graphics library.

## Roadmap

- [ ] Underlying graphics backends
  - [ ] X11
    - [x] Raw interface (100% of the [standard](https://www.x.org/releases/X11R7.7/doc/xproto/x11protocol.html))
    - [ ] Friendly interface (Nice types for bitmasks, etc. tracking resources with lifetimes)
    - [ ] Extensions
      - [ ] MIT-SHM
  - [ ] Wayland
  - [ ] WinAPI
  - [ ] Web/Canvas?
  - [ ] Mobile?
- [ ] Sound
- [ ] Friendly backend-agnostic wrapper

## Examples

### Pong

```console
cargo run --release --bin pong
```

Controls:

- Quit: `q`
- Left player:`d/f`
- Right player: `j/k`
- Restart: `r`


[Source](./src/bin/pong.rs)

![preview](./img/pong.png)

### `xlsfonts` clone

[Original implementation](https://gitlab.freedesktop.org/xorg/app/xlsfonts)

```console
cargo run --release --bin xlsfonts
```

[Source](./src/bin/xlsfonts.rs)
