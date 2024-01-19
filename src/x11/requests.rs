use crate::x11::{
    replies::ReplyType, utils::pad, BeBytes, Drawable, Font, GContext, Pixmap, Rectangle, Window,
    WindowClass, WindowVisual,
};
use std::{
    io::{self, Write},
    ops::BitOr,
};

mod opcodes;

macro_rules! impl_raw_field {
    ($ty:path, $setter:ident, $idx:expr) => {
        pub fn $setter(mut self, new_value: $ty) -> Self {
            self.values.values[$idx] = Some(new_value.into());
            self
        }
    };
}

macro_rules! impl_raw_fields_go {
    ($idx:expr $(,)?) => { };

    ($idx:expr, $setter:ident: $ty:path, $($rest:tt)*) => {
        impl_raw_field!($ty, $setter, $idx);
        impl_raw_fields_go!($idx + 1, $($rest)*);
    };
}

macro_rules! impl_raw_fields {
    ($($rest:tt)*) => {
        impl_raw_fields_go!(0, $($rest)*);
    };
}

pub trait XRequest: BeBytes {
    fn reply_type() -> Option<ReplyType> {
        None
    }
}

macro_rules! write_le_bytes {
    ($w:expr, $content:expr) => {
        $w.write_all(&(($content).to_le_bytes()))?;
    };
}

#[derive(Debug, Clone)]
pub struct InitializeConnection {
    pub major_version: u16,
    pub minor_version: u16,
    pub authorization_protocol_name: Vec<u8>,
    pub authorization_protocol_data: Vec<u8>,
}

impl BeBytes for InitializeConnection {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let authorization_name_len = self.authorization_protocol_name.len();
        let authorization_name_pad = pad(authorization_name_len);
        let authorization_data_len = self.authorization_protocol_data.len();
        let authorization_data_pad = pad(authorization_data_len);

        w.write_all(b"l\0")?;
        write_le_bytes!(w, self.major_version);
        write_le_bytes!(w, self.minor_version);
        write_le_bytes!(w, authorization_name_len as u16);
        write_le_bytes!(w, authorization_data_len as u16);
        w.write_all(&[0u8; 2])?; // unused
        w.write_all(&self.authorization_protocol_name)?;
        w.write_all(&vec![0u8; authorization_name_pad])?; // unused, pad
        w.write_all(&self.authorization_protocol_data)?;
        w.write_all(&vec![0u8; authorization_data_pad])?; // unused, pad

        Ok(())
    }
}

impl XRequest for InitializeConnection {}

#[derive(Debug, Clone)]
pub struct WindowCreationAttributes {
    values: ListOfValues<15>,
}

impl WindowCreationAttributes {
    pub fn new() -> Self {
        Self {
            values: ListOfValues::new(),
        }
    }

    impl_raw_fields! {
      set_background_pixmap: u32,
      set_background_pixel: u32,
      set_border_pixmap: u32,
      set_border_pixel: u32,
      set_bit_gravity: u32,
      set_win_gravity: u32,
      set_backing_store: u32,
      set_backing_planes: u32,
      set_backing_pixel: u32,
      set_override_redirect: u32,
      set_save_under: u32,
      set_event_mask: EventType,
      set_do_not_propagate_mask: u32,
      set_colormap: u32,
      set_cursor: u32,
    }
}

#[derive(Debug, Clone)]
pub struct CreateWindow {
    pub depth: u8,
    pub wid: Window,
    pub parent: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub window_class: WindowClass,
    pub visual: WindowVisual,
    pub attributes: WindowCreationAttributes,
}

impl BeBytes for CreateWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.attributes.values.mask_and_count();

        write_le_bytes!(w, opcodes::CREATE_WINDOW);
        write_le_bytes!(w, self.depth);
        write_le_bytes!(w, 8u16 + n); // length
        write_le_bytes!(w, self.wid.id().value());
        write_le_bytes!(w, self.parent.id().value());
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.border_width);
        write_le_bytes!(w, self.window_class as u16);
        write_le_bytes!(w, self.visual.value());
        write_le_bytes!(w, bitmask);
        self.attributes.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for CreateWindow {}

#[derive(Debug, Clone, Copy)]
pub struct GetWindowAttributes {
    pub window: Window,
}

impl BeBytes for GetWindowAttributes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_WINDOW_ATTRIBUTES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window.id().value());

        Ok(())
    }
}

impl XRequest for GetWindowAttributes {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetWindowAttributes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapWindow {
    pub window: Window,
}

impl BeBytes for MapWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::MAP_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.window.id().value());

        Ok(())
    }
}

impl XRequest for MapWindow {}

pub struct GetGeometry {
    pub drawable: Drawable,
}

impl BeBytes for GetGeometry {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_GEOMETRY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.drawable.value());

        Ok(())
    }
}

impl XRequest for GetGeometry {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetGeometry)
    }
}

#[derive(Debug, Clone)]
pub struct PolyFillRectangle {
    pub drawable: Drawable,
    pub gc: GContext,
    pub rectangles: Vec<Rectangle>,
}

impl BeBytes for PolyFillRectangle {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n: u16 = self.rectangles.len() as u16;

        write_le_bytes!(w, opcodes::POLY_FILL_RECTANGLE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + (2 * n)); // request length
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, self.gc.id().value());
        for rectangle in &self.rectangles {
            rectangle.to_le_bytes(w)?;
        }

        Ok(())
    }
}

impl XRequest for PolyFillRectangle {}

#[derive(Debug, Clone)]
pub struct ListOfValues<const N: usize> {
    values: [Option<u32>; N],
}

impl<const N: usize> ListOfValues<N> {
    pub fn new() -> Self {
        Self { values: [None; N] }
    }

    pub fn mask_and_count(&self) -> (u32, u16) {
        let mut bitmask: u32 = 0;
        let mut n: u16 = 0;

        for value in self.values.iter().rev() {
            bitmask <<= 1;
            bitmask |= value.is_some() as u32;
            n += value.is_some() as u16;
        }

        (bitmask, n)
    }

    pub fn to_le_bytes_if_set(&self, w: &mut impl Write) -> io::Result<()> {
        for value in self.values {
            if let Some(value) = value {
                write_le_bytes!(w, value);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GContextSettings {
    values: ListOfValues<23>,
}

impl GContextSettings {
    pub fn new() -> Self {
        Self {
            values: ListOfValues::new(),
        }
    }

    impl_raw_fields! {
      set_function: u32, // TODO: type
      set_plane_mask: u32,
      set_foreground: u32,
      set_background: u32,
      set_line_width: u16,
      set_line_style: u32, // TODO: type
      set_cap_style: u32, // TODO: type
      set_join_style: u32, // TODO: type
      set_fill_style: u32, // TODO: type
      set_fill_rule: u32, // TODO: type
      set_tile: Pixmap,
      set_stipple: Pixmap,
      set_tile_stipple_x_origin: u16,
      set_tile_stipple_y_origin: u16,
      set_font: Font,
      set_subwindow_mode: u32,
      set_graphics_exposures: bool,
      set_clip_x_origin: u16,
      set_clip_y_origin: u16,
      set_clip_mask: Pixmap, // TODO: or None
      set_dash_offset: u16,
      set_dashes: u8,
      set_arc_mode: u32,
    }
}

#[derive(Debug, Clone)]
pub struct CreateGC {
    pub cid: GContext,
    pub drawable: Drawable,
    pub values: GContextSettings,
}

impl BeBytes for CreateGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_le_bytes!(w, opcodes::CREATE_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16 + n); // length
        write_le_bytes!(w, self.cid.id().value());
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, bitmask);
        self.values.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for CreateGC {}

#[derive(Debug, Clone)]
pub struct ChangeGC {
    pub gcontext: GContext,
    pub values: GContextSettings,
}

impl BeBytes for ChangeGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_le_bytes!(w, opcodes::CHANGE_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + n); // length
        write_le_bytes!(w, self.gcontext.id().value());
        write_le_bytes!(w, bitmask);
        self.values.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for ChangeGC {}

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl BeBytes for GetInputFocus {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, &opcodes::GET_INPUT_FOCUS);
        write_le_bytes!(w, &0u8); // unused
        write_le_bytes!(w, &1u16); // length

        Ok(())
    }
}

// FIXME: Add response
impl XRequest for GetInputFocus {}

pub struct EventType {
    value: u32,
}

// TODO: Macro for these
impl EventType {
    pub const KEY_PRESS: Self = Self { value: 0x00000001 };
    pub const KEY_RELEASE: Self = Self { value: 0x00000002 };
    pub const BUTTON_PRESS: Self = Self { value: 0x00000004 };
    pub const BUTTON_RELEASE: Self = Self { value: 0x00000008 };
    pub const ENTER_WINDOW: Self = Self { value: 0x00000010 };
    pub const LEAVE_WINDOW: Self = Self { value: 0x00000020 };
    pub const POINTER_MOTION: Self = Self { value: 0x00000040 };
    pub const POINTER_MOTION_HINT: Self = Self { value: 0x00000080 };
    pub const BUTTON1_MOTION: Self = Self { value: 0x00000100 };
    pub const BUTTON2_MOTION: Self = Self { value: 0x00000200 };
    pub const BUTTON3_MOTION: Self = Self { value: 0x00000400 };
    pub const BUTTON4_MOTION: Self = Self { value: 0x00000800 };
    pub const BUTTON5_MOTION: Self = Self { value: 0x00001000 };
    pub const BUTTON_MOTION: Self = Self { value: 0x00002000 };
    pub const KEYMAP_STATE: Self = Self { value: 0x00004000 };
    pub const EXPOSURE: Self = Self { value: 0x00008000 };
    pub const VISIBILITY_CHANGE: Self = Self { value: 0x00010000 };
    pub const STRUCTURE_NOTIFY: Self = Self { value: 0x00020000 };
    pub const RESIZE_REDIRECT: Self = Self { value: 0x00040000 };
    pub const SUBSTRUCTURE_NOTIFY: Self = Self { value: 0x00080000 };
    pub const SUBSTRUCTURE_REDIRECT: Self = Self { value: 0x00100000 };
    pub const FOCUS_CHANGE: Self = Self { value: 0x00200000 };
    pub const PROPERTY_CHANGE: Self = Self { value: 0x00400000 };
    pub const COLORMAP_CHANGE: Self = Self { value: 0x00800000 };
    pub const OWNER_GRAB_BUTTON: Self = Self { value: 0x01000000 };

    fn contains(self, other: Self) -> bool {
        (self.value & other.value) != 0
    }
}

impl BitOr for EventType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}

impl Into<u32> for EventType {
    fn into(self) -> u32 {
        self.value
    }
}
