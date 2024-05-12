use just_x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    keysym::KeySym,
    requests::{
        self, GContextSettings, GetGeometry, KeyCode, PutImage, PutImageFormat,
        WindowCreationAttributes,
    },
    Drawable, GContextId, WindowClass, WindowId, WindowVisual, XDisplay,
};
use just_x11_simple::{keys::KeySymbols, X11Connection};
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

const FPS: u64 = 60;

#[derive(Clone, Copy)]
enum KeyAction {
    LeftUp,
    LeftDown,
    RightUp,
    RightDown,
    Restart,
    Quit,
}

struct KeyBindings {
    bindings: HashMap<KeyCode, KeyAction>,
    key_symbols: KeySymbols,
}

impl KeyBindings {
    fn new(conn: &mut X11Connection) -> Result<Self, Error> {
        let key_symbols = KeySymbols::new(conn.display_mut())?;
        Ok(Self {
            bindings: HashMap::new(),
            key_symbols,
        })
    }

    fn add_binding(&mut self, key_sym: KeySym, action: KeyAction) {
        let key_codes = self.key_symbols.get_keycodes(key_sym);
        for key in key_codes {
            self.bindings.insert(key, action);
        }
    }

    fn get_action(&self, key_code: KeyCode) -> Option<KeyAction> {
        self.bindings.get(&key_code).copied()
    }
}

pub fn go() -> Result<(), Error> {
    let mut conn = X11Connection::new(XDisplay::open()?);

    let mut bindings = KeyBindings::new(&mut conn)?;
    bindings.add_binding(KeySym::d, KeyAction::LeftDown);
    bindings.add_binding(KeySym::f, KeyAction::LeftUp);
    bindings.add_binding(KeySym::j, KeyAction::RightDown);
    bindings.add_binding(KeySym::k, KeyAction::RightUp);
    bindings.add_binding(KeySym::r, KeyAction::Restart);
    bindings.add_binding(KeySym::q, KeyAction::Quit);

    let window = {
        let window_id = WindowId::from(conn.display_mut().id_allocator().allocate_id());
        let window_attributes = WindowCreationAttributes::new().set_event_mask(
            EventType::KEY_PRESS | EventType::KEY_RELEASE | EventType::STRUCTURE_NOTIFY,
        );
        let create_window = requests::CreateWindow {
            depth: conn.default_screen().root_depth,
            wid: window_id,
            parent: conn.default_screen().root,
            x: 0,
            y: 0,
            width: 600,
            height: 800,
            border_width: 0,
            window_class: WindowClass::CopyFromParent,
            visual: WindowVisual::CopyFromParent,
            attributes: window_attributes,
        };
        conn.display_mut().send_request(&create_window)?;
        window_id
    };

    conn.display_mut()
        .send_request(&requests::MapWindow { window })?;

    let gc = {
        let gc_id = GContextId::from(conn.display_mut().id_allocator().allocate_id());
        conn.display_mut().send_request(&requests::CreateGC {
            cid: gc_id,
            drawable: Drawable::Window(window),
            values: GContextSettings::new(),
        })?;
        gc_id
    };

    let pad_size = V2 { x: 20, y: 150 };

    let mut left_pad = Pad {
        position: V2 { x: 0, y: 0 },
        size: pad_size,
        direction: PadDirection::None,
        player: Player::Left,
    };

    let mut right_pad = Pad {
        position: V2 { x: 0, y: 0 },
        size: pad_size,
        direction: PadDirection::None,
        player: Player::Right,
    };

    conn.display_mut().send_request(&GetGeometry {
        drawable: Drawable::Window(window),
    })?;

    let window_geometry_pending = conn.display_mut().send_request(&GetGeometry {
        drawable: Drawable::Window(window),
    })?;
    conn.flush()?;
    let window_geometry = conn
        .display_mut()
        .await_pending_reply(window_geometry_pending)?
        .unwrap();

    let mut window_size = V2 {
        x: window_geometry.width,
        y: window_geometry.height,
    };

    let mut ball = Ball {
        position: V2 { x: 0, y: 0 },
        size: V2 { x: 32, y: 32 },
        direction: V2 { x: 0, y: 0 },
    };
    ball.reset(window_size);

    let mut canvas = Canvas::new(window_size.x, window_size.y);

    'main_loop: loop {
        let frame_start = std::time::Instant::now();

        // Background
        canvas.rectangle(window_size.x, window_size.y, 0, 0, 0x222222);

        left_pad.update(window_size);
        right_pad.update(window_size);
        ball.update(window_size, &left_pad, &right_pad);

        left_pad.display(&mut canvas);
        right_pad.display(&mut canvas);
        ball.display(&mut canvas);

        canvas.display(gc, window_geometry.depth, window, conn.display_mut())?;
        conn.flush()?;

        for error in conn.display_mut().errors() {
            dbg!(error);
        }

        for event in conn.display_mut().events()? {
            match event {
                SomeEvent::KeyPress(event) => {
                    if let Some(action) = bindings.get_action(event.detail) {
                        match action {
                            KeyAction::Quit => break 'main_loop,
                            KeyAction::RightDown => right_pad.direction = PadDirection::Down,
                            KeyAction::RightUp => right_pad.direction = PadDirection::Up,
                            KeyAction::LeftDown => left_pad.direction = PadDirection::Down,
                            KeyAction::LeftUp => left_pad.direction = PadDirection::Up,
                            KeyAction::Restart => ball.reset(window_size),
                        }
                    }
                }
                SomeEvent::KeyRelease(event) => {
                    if let Some(action) = bindings.get_action(event.detail) {
                        match action {
                            KeyAction::RightDown => right_pad.direction = PadDirection::None,
                            KeyAction::RightUp => right_pad.direction = PadDirection::None,
                            KeyAction::LeftDown => left_pad.direction = PadDirection::None,
                            KeyAction::LeftUp => left_pad.direction = PadDirection::None,
                            _ => {}
                        }
                    }
                }
                SomeEvent::ConfigureNotify(event) => {
                    if event.event == window {
                        window_size.x = event.width;
                        window_size.y = event.height;
                        canvas = Canvas::new(window_size.x, window_size.y);
                    }
                }
                _event => {}
            }
        }

        let frame_end = std::time::Instant::now();
        let frame_duration = frame_end - frame_start;
        if let Some(final_sleep) = Duration::from_micros(1000000 / FPS).checked_sub(frame_duration)
        {
            std::thread::sleep(final_sleep);
        }
    }

    Ok(())
}

fn main() {
    match go() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("pong: error: {}", err);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Player {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
enum PadDirection {
    Down,
    Up,
    None,
}

#[derive(Debug, Clone, Copy)]
struct V2<T> {
    x: T,
    y: T,
}

#[derive(Debug, Clone, Copy)]
struct Pad {
    position: V2<i16>,
    size: V2<u16>,
    direction: PadDirection,
    player: Player,
}

impl Pad {
    fn update(&mut self, window_size: V2<u16>) {
        let velocity = 15;

        match self.direction {
            PadDirection::Down => self.position.y += velocity,
            PadDirection::Up => self.position.y -= velocity,
            PadDirection::None => {}
        }

        if self.position.y < 0 {
            self.position.y = 0;
        } else if self.position.y > window_size.y as i16 - self.size.y as i16 {
            self.position.y = window_size.y as i16 - self.size.y as i16;
        }

        self.position.x = match self.player {
            Player::Right => window_size.x as i16 - self.size.x as i16 * 7 / 5,
            Player::Left => self.size.x as i16 * 2 / 5,
        };
    }

    fn display(&self, canvas: &mut Canvas) {
        let color = match self.player {
            Player::Left => 0x4eb4fa,
            Player::Right => 0xf92672,
        };

        canvas.rectangle(
            self.size.x,
            self.size.y,
            self.position.x as u16,
            self.position.y as u16,
            color,
        );
    }
}

#[derive(Debug, Clone, Copy)]
struct Ball {
    position: V2<i16>,
    size: V2<u16>,
    direction: V2<i16>,
}

impl Ball {
    fn update(&mut self, window_size: V2<u16>, left_pad: &Pad, right_pad: &Pad) {
        self.position.x += self.direction.x;
        self.position.y += self.direction.y;

        let offset = if self.position.y >= left_pad.position.y
            && self.position.y <= left_pad.position.y + left_pad.size.y as i16
        {
            left_pad.position.x + left_pad.size.x as i16
        } else {
            0
        };
        let bound = offset;
        if self.position.x <= bound {
            if offset == 0 {
                self.direction.x = 0;
                self.direction.y = 0;
            }

            self.position.x = bound;
            self.direction.x *= -1;
        }

        let offset = if self.position.y >= right_pad.position.y
            && self.position.y <= right_pad.position.y + right_pad.size.y as i16
        {
            window_size.x as i16 - right_pad.position.x
        } else {
            0
        };
        let bound = window_size.x as i16 - self.size.x as i16 - offset;
        if self.position.x >= bound {
            if offset == 0 {
                self.direction.x = 0;
                self.direction.y = 0;
            }

            self.position.x = bound;
            self.direction.x *= -1;
        }

        let bound = 0;
        if self.position.y <= bound {
            self.position.y = bound;
            self.direction.y *= -1;
        }

        let bound = window_size.y as i16 - self.size.y as i16;
        if self.position.y >= bound {
            self.position.y = bound;
            self.direction.y *= -1;
        }
    }

    fn display(&self, canvas: &mut Canvas) {
        canvas.rectangle(
            self.size.x,
            self.size.y,
            self.position.x as u16,
            self.position.y as u16,
            0xffffff,
        );
    }

    fn reset(&mut self, window_size: V2<u16>) {
        // rng without rng

        let now = SystemTime::now();
        let r = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let r = r.as_millis() as u32;

        let bound: u16 = 50;

        self.position.x =
            (r.wrapping_mul(7) as u16 % (window_size.x - bound)) as i16 + (bound as i16);
        self.position.y =
            (r.wrapping_mul(13) as u16 % (window_size.y - bound)) as i16 + (bound as i16);

        let base_velocity = 5;

        let v = [-base_velocity, base_velocity];
        self.direction.x = v[r.wrapping_mul(17) as usize % 2];
        self.direction.y = v[r.wrapping_mul(19) as usize % 2];
    }
}

// TODO: Move it its own module
struct Canvas {
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl Canvas {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            data: vec![0; width as usize * height as usize * 4],
        }
    }

    fn rectangle(&mut self, width: u16, height: u16, start_x: u16, start_y: u16, color: u32) {
        let [a, r, g, b] = color.to_le_bytes();
        for y in start_y..(start_y + height) {
            for x in start_x..(start_x + width) {
                let index = (y as usize * self.width as usize + x as usize) * 4;

                self.data[index] = a;
                self.data[index + 1] = r;
                self.data[index + 2] = g;
                self.data[index + 3] = b;
            }
        }
    }

    fn display(
        &self,
        gc: GContextId,
        depth: u8,
        window: WindowId,
        display: &mut XDisplay,
    ) -> Result<(), Error> {
        let chunk_size: u16 =
            ((display.maximum_request_length() as usize - 24) / self.width as usize) as u16;
        for line in (0..self.height).step_by(chunk_size as usize) {
            let chunk_size = if line + chunk_size > self.height {
                self.height - line
            } else {
                chunk_size
            };

            let data = &self.data[(self.width as usize * line as usize * 4)
                ..(self.width as usize * (line as usize + chunk_size as usize) * 4)];
            assert_eq!(data.len(), self.width as usize * chunk_size as usize * 4);

            let req = PutImage {
                format: PutImageFormat::ZPixmap,
                drawable: Drawable::Window(window),
                gc,
                width: self.width,
                height: chunk_size,
                dst_x: 0,
                dst_y: line as i16,
                left_pad: 0,
                depth,
                data,
            };
            display.send_request(&req)?;
        }
        Ok(())
    }
}
