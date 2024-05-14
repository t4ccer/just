use crate::backend::{bitmap::BitmapBackend, x11_mit_shm::X11MitShmBackend, Backend};
use std::fmt::Debug;

mod backend;
pub mod draw;

pub const BYTES_PER_PIXEL: u32 = 4;

#[derive(Debug)]
pub enum CanvasError {
    X11ProtocolError(just_x11::error::Error),
}

impl From<just_x11::error::Error> for CanvasError {
    fn from(err: just_x11::error::Error) -> Self {
        Self::X11ProtocolError(err)
    }
}

pub type Result<T> = core::result::Result<T, CanvasError>;

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
    pub position: Vector2<u32>,
    pressed_mask: ButtonMask,
    clicked_this_frame: ButtonMask,
}

impl Pointer {
    #[inline]
    /// Pointer state at `x = 0, y = 0` with no buttons pressed
    fn new() -> Self {
        Self {
            position: Vector2 { x: 0, y: 0 },
            pressed_mask: ButtonMask::zeroed(),
            clicked_this_frame: ButtonMask::zeroed(),
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

pub enum BackendType {
    X11MitShm,
    Bitmap { size: Vector2<u32> },
}

pub struct Canvas {
    backend: Box<dyn Backend>,
    pointer: Pointer,
    resized: bool,
    should_close: bool,
}

impl Canvas {
    #[inline]
    pub fn new(title: &str) -> Result<Self> {
        let backend = X11MitShmBackend::new(title)?;
        Ok(Self::with_backend(Box::new(backend)))
    }

    #[inline]
    fn with_backend(backend: Box<dyn Backend>) -> Self {
        Self {
            backend,
            pointer: Pointer::new(),
            resized: false,
            should_close: false,
        }
    }

    #[inline]
    pub fn with_backend_type(title: &str, backend: BackendType) -> Result<Self> {
        let backend: Box<dyn Backend> = match backend {
            BackendType::X11MitShm => Box::new(X11MitShmBackend::new(title)?),
            BackendType::Bitmap { size } => Box::new(BitmapBackend::new(size)),
        };
        Ok(Self::with_backend(backend))
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer {
        &self.pointer
    }

    #[inline]
    pub fn resized(&self) -> bool {
        self.resized
    }

    #[inline]
    pub fn should_close(&self) -> bool {
        self.should_close
    }

    #[inline]
    pub fn raw_buf_mut(&mut self) -> &mut [u8] {
        self.backend.buf_mut()
    }

    #[inline]
    pub fn raw_buf(&self) -> &[u8] {
        self.backend.buf()
    }

    pub fn process_events(&mut self) -> Result<()> {
        self.resized = false;

        for n in 0..u8::MAX {
            if self.pointer.clicked_this_frame.get(n as usize) {
                self.pointer.set_released(n);
                self.pointer.clicked_this_frame.clear(n as usize);
            }
        }

        // NOTE: During quick clicks pressed and released event may come in one frame
        // thus we keep track of these and release after rendering so the user code
        // can detect the click. This assumes that release event will come after press
        let mut pressed_this_frame = ButtonMask::zeroed();

        for event in self.backend.events()? {
            match event {
                Event::Resize { new_size } => {
                    self.backend.resize(new_size)?;
                    self.resized = true;
                }
                Event::ButtonPress { button } => {
                    pressed_this_frame.set(button as usize);
                    self.pointer.set_pressed(button);
                }
                Event::ButtonRelease { button } => {
                    if pressed_this_frame.get(button as usize) {
                        self.pointer.clicked_this_frame.set(button as usize);
                    } else {
                        self.pointer.set_released(button);
                    }
                }
                Event::PointerMotion { position } => {
                    self.pointer.position = position;
                }
                Event::Shutdown => {
                    self.should_close = true;
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub fn window_size(&self) -> Vector2<u32> {
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
    Resize { new_size: Vector2<u32> },
    ButtonPress { button: u8 },
    ButtonRelease { button: u8 },
    PointerMotion { position: Vector2<u32> },
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

#[derive(Debug, Clone, Copy)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    #[inline(always)]
    pub fn clamp(self, lower_bound: Self, upper_bound: Self) -> Self
    where
        T: Ord,
    {
        Self {
            x: self.x.clamp(lower_bound.x, upper_bound.x),
            y: self.y.clamp(lower_bound.y, upper_bound.y),
        }
    }
}

impl Vector2<i32> {
    #[inline(always)]
    pub fn as_u32(self) -> Vector2<u32> {
        Vector2 {
            x: self.x as u32,
            y: self.y as u32,
        }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Vector2 { x: 0, y: 0 }
    }
}

impl Vector2<u32> {
    #[inline(always)]
    pub fn as_i32(self) -> Vector2<i32> {
        Vector2 {
            x: self.x as i32,
            y: self.y as i32,
        }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        Vector2 { x: 0, y: 0 }
    }
}
