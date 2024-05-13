use crate::backend::{x11_mit_shm::X11MitShmBackend, Backend};
use std::{fmt::Debug, time::Duration};

mod backend;
pub mod draw;

const BYTES_PER_PIXEL: u32 = 4;

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
    fn new(array: [u8; SIZE]) -> Self {
        Self { array }
    }

    #[inline]
    fn zeroed() -> Self {
        Self::new([0; SIZE])
    }

    #[inline]
    fn set(&mut self, key: usize) {
        self.array[(key / 8) as usize] |= 1 << (key % 8);
    }

    #[inline]
    fn clear(&mut self, key: usize) {
        self.array[(key / 8) as usize] &= !(1 << (key % 8));
    }

    #[inline]
    fn get(&self, key: usize) -> bool {
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
            pressed_mask: ButtonMask::zeroed(),
        }
    }

    #[inline]
    fn set_pressed(&mut self, key: u8) {
        self.pressed_mask.set(key as usize);
    }

    #[inline]
    fn set_released(&mut self, key: u8) {
        self.pressed_mask.clear(key as usize);
    }

    #[inline]
    pub fn is_pressed(&self, key: u8) -> bool {
        self.pressed_mask.get(key as usize)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiId(u32);

pub struct Context {
    pub(crate) backend: Box<dyn Backend>,
    pointer: Pointer,
    next_id: u32,
    hot: Option<UiId>,
    active: Option<UiId>,
}

impl Context {
    #[inline]
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
    #[inline]
    pub fn next_id(&mut self) -> UiId {
        let res = self.next_id;
        self.next_id += 1;
        UiId(res)
    }

    #[inline]
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

    #[inline]
    pub fn make_hot(&mut self, id: UiId) {
        match self.hot {
            None => {
                self.hot = Some(id);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn make_active(&mut self, id: UiId) {
        match self.hot {
            Some(hot) if hot == id => {
                self.active = Some(id);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn is_hot(&self, id: UiId) -> bool {
        self.hot.is_some_and(|hot| hot == id)
    }

    #[inline]
    pub fn is_active(&self, id: UiId) -> bool {
        self.active.is_some_and(|active| active == id)
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    pub fn fps_limited_loop<F>(&mut self, fps: u64, mut draw: F) -> Result<()>
    where
        F: FnMut(&mut Self),
    {
        let mut should_close = false;
        while !should_close {
            let frame_start = std::time::Instant::now();

            // NOTE: During quick clicks pressed and released event may come in one frame
            // thus we keep track of these and release after rendering so the user code
            // can detect the click. This assumes that release event will come after press
            let mut pressed_this_frame = ButtonMask::zeroed();
            let mut clicked_this_frame = ButtonMask::zeroed();

            for event in self.backend.events()? {
                match event {
                    Event::Resize {
                        new_width,
                        new_height,
                    } => {
                        self.backend.resize(new_width, new_height)?;
                    }
                    Event::ButtonPress { button } => {
                        pressed_this_frame.set(button as usize);
                        self.pointer.set_pressed(button);
                    }
                    Event::ButtonRelease { button } => {
                        if pressed_this_frame.get(button as usize) {
                            clicked_this_frame.set(button as usize);
                        } else {
                            self.pointer.set_released(button);
                        }
                    }
                    Event::PointerMotion { x, y } => {
                        self.pointer.x = x;
                        self.pointer.y = y;
                    }
                    Event::Shutdown => {
                        should_close = true;
                    }
                }
            }

            draw(self);
            self.flush()?;

            for n in 0..u8::MAX {
                if clicked_this_frame.get(n as usize) {
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
    pub fn window_size(&self) -> (u32, u32) {
        self.backend.size()
    }

    #[inline]
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
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn from_components(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// AARRGGBB
    pub const fn from_raw(raw: u32) -> Self {
        let [a, r, g, b] = raw.to_be_bytes();
        Self { a, r, g, b }
    }
}
