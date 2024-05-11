use crate::backend::{x11_mit_shm::X11MitShmBackend, Backend};
use std::{fmt::Debug, time::Duration};

mod backend;

#[derive(Debug)]
pub enum ImmUiError {
    X11ProtocolError(just_x11::error::Error),
}

impl From<just_x11::error::Error> for ImmUiError {
    fn from(err: just_x11::error::Error) -> Self {
        Self::X11ProtocolError(err)
    }
}

pub type Result<T> = core::result::Result<T, ImmUiError>;

#[derive(Debug)]
struct BitArray<const SIZE: usize> {
    array: [u8; SIZE],
}

impl<const SIZE: usize> BitArray<SIZE> {
    #[inline]
    fn new() -> Self {
        Self { array: [0; SIZE] }
    }

    #[inline]
    fn set(&mut self, key: u8) {
        self.array[(key / 8) as usize] |= 1 << (key % 8);
    }

    #[inline]
    fn clear(&mut self, key: u8) {
        self.array[(key / 8) as usize] &= !(1 << (key % 8));
    }

    #[inline]
    fn get(&self, key: u8) -> bool {
        (self.array[(key / 8) as usize] & (1 << (key % 8))) != 0
    }
}

type ButtonMask = BitArray<32>;

#[derive(Debug)]
pub struct Pointer {
    pub x: u32,
    pub y: u32,
    pressed_mask: ButtonMask,
}

impl Pointer {
    #[inline]
    /// Pointer state at `x = 0, y = 0` with no buttons pressed
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            pressed_mask: ButtonMask::new(),
        }
    }

    #[inline]
    fn set_pressed(&mut self, key: u8) {
        self.pressed_mask.set(key);
    }

    #[inline]
    fn set_released(&mut self, key: u8) {
        self.pressed_mask.clear(key);
    }

    #[inline]
    pub fn is_pressed(&self, key: u8) -> bool {
        self.pressed_mask.get(key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiId(u32);

pub struct Context {
    backend: Box<dyn Backend>,
    pointer: Pointer,
    next_id: u32,
    hot: Option<UiId>,
    active: Option<UiId>,
}

impl Context {
    pub fn new(title: &str) -> Result<Self> {
        let backend = X11MitShmBackend::new(title)?;
        Ok(Self {
            backend: Box::new(backend),
            pointer: Pointer::new(),
            next_id: 0,
            hot: None,
            active: None,
        })
    }

    // TODO: Remove. This will break in conditional rendering
    pub fn next_id(&mut self) -> UiId {
        let res = self.next_id;
        self.next_id += 1;
        UiId(res)
    }

    pub fn make_inactive(&mut self, id: UiId) {
        match self.hot {
            Some(hot) if hot == id => {
                self.hot = None;
            }
            _ => {}
        }

        match self.active {
            Some(active) if active == id => {
                self.active = None;
            }
            _ => {}
        }
    }

    pub fn make_hot(&mut self, id: UiId) {
        match self.hot {
            None => {
                self.hot = Some(id);
            }
            _ => {}
        }
    }

    pub fn make_active(&mut self, id: UiId) {
        match self.hot {
            Some(hot) if hot == id => {
                self.active = Some(id);
            }
            _ => {}
        }
    }

    pub fn is_hot(&self, id: UiId) -> bool {
        self.hot.is_some_and(|hot| hot == id)
    }

    pub fn is_active(&self, id: UiId) -> bool {
        self.active.is_some_and(|active| active == id)
    }

    fn should_close_window(&self) -> bool {
        false
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    #[inline]
    pub fn fps_limited_loop<F>(&mut self, fps: u64, mut draw: F) -> Result<()>
    where
        F: FnMut(&mut Self),
    {
        while !self.should_close_window() {
            let frame_start = std::time::Instant::now();

            // NOTE: During quick clicks pressed and released event may come in one frame
            // thus we keep track of these and release after rendering so the user code
            // can detect the click. This assumes that release event will come after press
            let mut pressed_this_frame = ButtonMask::new();
            let mut clicked_this_frame = ButtonMask::new();

            for event in self.backend.events()? {
                match event {
                    Event::Resize {
                        new_width,
                        new_height,
                    } => {
                        self.backend.resize(new_width, new_height)?;
                    }
                    Event::ButtonPress { button } => {
                        pressed_this_frame.set(button);
                        self.pointer.set_pressed(button);
                    }
                    Event::ButtonRelease { button } => {
                        if pressed_this_frame.get(button) {
                            clicked_this_frame.set(button);
                        } else {
                            self.pointer.set_released(button);
                        }
                    }
                    Event::PointerMotion { x, y } => {
                        self.pointer.x = x;
                        self.pointer.y = y;
                    }
                }
            }

            draw(self);
            self.flush()?;

            for n in 0..u8::MAX {
                if clicked_this_frame.get(n) {
                    self.pointer.set_released(n);
                }
            }
            self.next_id = 0;

            let frame_end = std::time::Instant::now();
            let frame_duration = frame_end - frame_start;
            let final_sleep = Duration::from_micros(1000000 / fps).checked_sub(frame_duration);
            if let Some(final_sleep) = final_sleep {
                std::thread::sleep(final_sleep);
            }
        }

        Ok(())
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.backend.size()
    }

    #[inline]
    pub fn background(&mut self, color: Color) {
        let (width, height) = self.size();
        self.rectangle(0, 0, width, height, color)
    }

    #[inline]
    pub fn rectangle(&mut self, x: u32, y: u32, width: u32, height: u32, color: Color) {
        let (window_width, window_height) = self.size();

        for cy in y..(y + height).clamp(0, window_height) {
            for cx in x..(x + width).clamp(0, window_width) {
                self.backend.draw_pixel(cx, cy, color);
            }
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        self.backend.flush_window()
    }
}

// TODO: Transalte button codes

#[derive(Debug)]
pub(crate) enum Event {
    Resize { new_width: u32, new_height: u32 },
    ButtonPress { button: u8 },
    ButtonRelease { button: u8 },
    PointerMotion { x: u32, y: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_components(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// AARRGGBB
    pub fn from_raw(raw: u32) -> Self {
        let [a, r, g, b] = raw.to_be_bytes();
        Self { a, r, g, b }
    }
}
