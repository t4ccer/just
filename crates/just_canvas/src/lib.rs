// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

use backend::shared_bitmap;

use crate::backend::{owned_bitmap::OwnedBitmapBackend, x11_mit_shm::X11MitShmBackend, Backend};
use std::{
    cmp,
    fmt::Debug,
    ops::{Add, Sub},
};

mod backend;
pub mod draw;

pub const BYTES_PER_PIXEL: u32 = 4;

#[derive(Debug)]
pub enum CanvasError {
    X11ProtocolError(just_x11::error::Error),
    SharedBitmapError(shared_bitmap::Error),
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
        self.array[key / 8] |= 1 << (key % 8);
    }

    #[inline]
    fn clear(&mut self, key: usize) {
        self.array[key / 8] &= !(1 << (key % 8));
    }

    #[inline]
    fn get(&self, key: usize) -> bool {
        (self.array[key / 8] & (1 << (key % 8))) != 0
    }
}

#[derive(Debug)]
struct ButtonMask {
    buttons: BitArray<32>,
}

impl ButtonMask {
    fn new() -> Self {
        Self {
            buttons: BitArray::zeroed(),
        }
    }

    #[inline]
    fn set_pressed(&mut self, key: PointerButton) {
        self.buttons.set(key as usize);
    }

    #[inline]
    fn set_released(&mut self, key: PointerButton) {
        self.buttons.clear(key as usize);
    }

    #[inline]
    pub fn is_pressed(&self, key: PointerButton) -> bool {
        self.buttons.get(key as usize)
    }
}

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
            pressed_mask: ButtonMask::new(),
            clicked_this_frame: ButtonMask::new(),
        }
    }

    #[inline]
    fn set_pressed(&mut self, key: PointerButton) {
        self.pressed_mask.set_pressed(key);
    }

    #[inline]
    fn set_released(&mut self, key: PointerButton) {
        self.pressed_mask.set_released(key);
    }

    #[inline]
    pub fn is_pressed(&self, key: PointerButton) -> bool {
        self.pressed_mask.is_pressed(key)
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
            BackendType::Bitmap { size } => Box::new(OwnedBitmapBackend::new(size)),
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

        // FIXME

        for n in 0..u8::MAX {
            if self.pointer.clicked_this_frame.buttons.get(n as usize) {
                self.pointer.pressed_mask.buttons.clear(n as usize);
                self.pointer.clicked_this_frame.buttons.clear(n as usize);
            }
        }

        // NOTE: During quick clicks pressed and released event may come in one frame
        // thus we keep track of these and release after rendering so the user code
        // can detect the click. This assumes that release event will come after press
        let mut pressed_this_frame = ButtonMask::new();

        for event in self.backend.events()? {
            match event {
                Event::Resize { new_size } => {
                    self.backend.resize(new_size)?;
                    self.resized = true;
                }
                Event::ButtonPress { button } => {
                    pressed_this_frame.set_pressed(button);
                    self.pointer.set_pressed(button);
                }
                Event::ButtonRelease { button } => {
                    if pressed_this_frame.is_pressed(button) {
                        self.pointer.clicked_this_frame.set_pressed(button);
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub enum PointerButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
}

// TODO: Transalte button codes

#[derive(Debug)]
pub(crate) enum Event {
    Resize { new_size: Vector2<u32> },
    ButtonPress { button: PointerButton },
    ButtonRelease { button: PointerButton },
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
    #[inline(always)]
    pub const fn from_components(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self { a, r, g, b }
    }

    /// AARRGGBB
    #[inline(always)]
    pub const fn from_raw(raw: u32) -> Self {
        let [a, r, g, b] = raw.to_be_bytes();
        Self { a, r, g, b }
    }

    #[inline(always)]
    pub fn blend(c1: Self, c2: Self) -> Self {
        let r = cmp::min(
            (c1.r as u32 * (255 - c2.a as u32) + c2.r as u32 * c2.a as u32) / 255,
            255,
        ) as u8;
        let g = cmp::min(
            (c1.g as u32 * (255 - c2.a as u32) + c2.g as u32 * c2.a as u32) / 255,
            255,
        ) as u8;
        let b = cmp::min(
            (c1.b as u32 * (255 - c2.a as u32) + c2.b as u32 * c2.a as u32) / 255,
            255,
        ) as u8;

        Self { r, g, b, a: c1.a }
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

    #[inline(always)]
    pub fn clamp_non_negative(self) -> Self {
        Self {
            x: cmp::max(0, self.x),
            y: cmp::max(0, self.y),
        }
    }

    #[inline(always)]
    pub fn linear_interpolation(p1: Self, p2: Self, t: f32) -> Self {
        let mut x = p1.x as f32 + (p2.x as f32 - p1.x as f32) * t;
        if x < 0.0 {
            x = 0.0;
        }
        let x = x as i32;

        let mut y = p1.y as f32 + (p2.y as f32 - p1.y as f32) * t;
        if y < 0.0 {
            y = 0.0;
        }
        let y = y as i32;

        Vector2 { x, y }
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

impl<T> Add for Vector2<T>
where
    T: Add<Output = T>,
{
    type Output = Vector2<<T as Add>::Output>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Vector2<T>
where
    T: Sub<Output = T>,
{
    type Output = Vector2<<T as Sub>::Output>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
