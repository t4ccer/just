use justshow::x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    requests::{GContextSettings, GetGeometry, PutImage, PutImageFormat, WindowCreationAttributes},
    Drawable, GContext, Window, XDisplay,
};
use std::time::{Duration, SystemTime};

const FPS: u64 = 60;

pub fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;

    let window_attributes = WindowCreationAttributes::new().set_event_mask(
        EventType::KEY_PRESS | EventType::KEY_RELEASE | EventType::STRUCTURE_NOTIFY,
    );
    let window = display.create_simple_window(0, 0, 600, 800, 0, window_attributes)?;

    window.map(&mut display)?;

    let gc = window.create_gc(&mut display, GContextSettings::new())?;

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

    display.send_request(&GetGeometry {
        drawable: Drawable::Window(window),
    })?;

    let window_geometry_pending = display.send_request(&GetGeometry {
        drawable: Drawable::Window(window),
    })?;
    display.flush()?;
    let window_geometry = display.await_pending_reply(window_geometry_pending)?;

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

        canvas.display(gc, window_geometry.depth, window, &mut display)?;
        display.flush()?;

        for error in display.errors() {
            dbg!(error);
        }

        for event in display.events()? {
            match event {
                SomeEvent::KeyPress(event) => match event.detail {
                    24 => break 'main_loop,
                    44 => right_pad.direction = PadDirection::Down,
                    45 => right_pad.direction = PadDirection::Up,
                    40 => left_pad.direction = PadDirection::Down,
                    41 => left_pad.direction = PadDirection::Up,
                    27 => ball.reset(window_size),
                    _ => {}
                },
                SomeEvent::KeyRelease(event) => match event.detail {
                    44 => right_pad.direction = PadDirection::None,
                    45 => right_pad.direction = PadDirection::None,
                    40 => left_pad.direction = PadDirection::None,
                    41 => left_pad.direction = PadDirection::None,
                    _ => {}
                },
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
        gc: GContext,
        depth: u8,
        window: Window,
        display: &mut XDisplay,
    ) -> Result<(), Error> {
        let chunk_size: u16 =
            ((display.maximum_request_length as usize - 24) / self.width as usize) as u16;
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
