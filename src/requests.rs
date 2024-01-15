use crate::{
    utils::pad, BeBytes, Drawable, Font, GContext, Pixmap, Rectangle, ReplyType, Window,
    WindowClass, WindowVisual,
};
use std::io::{self, Write};

mod opcodes;

pub trait XRequest: BeBytes {
    fn reply_type() -> Option<ReplyType> {
        None
    }
}

macro_rules! write_be_bytes {
    ($w:expr, $content:expr) => {
        $w.write_all(&(($content).to_be_bytes()))?;
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
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let authorization_name_len = self.authorization_protocol_name.len();
        let authorization_name_pad = pad(authorization_name_len);
        let authorization_data_len = self.authorization_protocol_data.len();
        let authorization_data_pad = pad(authorization_data_len);

        w.write_all(b"B\0")?;
        write_be_bytes!(w, self.major_version);
        write_be_bytes!(w, self.minor_version);
        write_be_bytes!(w, authorization_name_len as u16);
        write_be_bytes!(w, authorization_data_len as u16);
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
    // TODO: values
}

impl BeBytes for CreateWindow {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_be_bytes!(w, opcodes::CREATE_WINDOW);
        write_be_bytes!(w, self.depth);
        write_be_bytes!(w, 8u16); // length, TODO: values
        write_be_bytes!(w, self.wid.id().value());
        write_be_bytes!(w, self.parent.id().value());
        write_be_bytes!(w, self.x);
        write_be_bytes!(w, self.y);
        write_be_bytes!(w, self.width);
        write_be_bytes!(w, self.height);
        write_be_bytes!(w, self.border_width);
        write_be_bytes!(w, self.window_class as u16);
        write_be_bytes!(w, self.visual.value());
        write_be_bytes!(w, 0u32); // bitmask

        // TODO: values

        Ok(())
    }
}

impl XRequest for CreateWindow {}

#[derive(Debug, Clone, Copy)]
pub struct GetWindowAttributes {
    pub window: Window,
}

impl BeBytes for GetWindowAttributes {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_be_bytes!(w, opcodes::GET_WINDOW_ATTRIBUTES);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 2u16); // length
        write_be_bytes!(w, self.window.id().value());

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
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_be_bytes!(w, opcodes::MAP_WINDOW);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 2u16); // size
        write_be_bytes!(w, self.window.id().value());

        Ok(())
    }
}

impl XRequest for MapWindow {}

#[derive(Debug, Clone)]
pub struct PolyFillRectangle {
    pub drawable: Drawable,
    pub gc: GContext,
    pub rectangles: Vec<Rectangle>,
}

impl BeBytes for PolyFillRectangle {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n: u16 = self.rectangles.len() as u16;

        write_be_bytes!(w, opcodes::POLY_FILL_RECTANGLE);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 3 + (2 * n)); // request length
        write_be_bytes!(w, self.drawable.value());
        write_be_bytes!(w, self.gc.id().value());
        for rectangle in &self.rectangles {
            rectangle.to_be_bytes(w)?;
        }

        Ok(())
    }
}

impl XRequest for PolyFillRectangle {}

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

    pub fn to_be_bytes_if_set(&self, w: &mut impl Write) -> io::Result<()> {
        for value in self.values {
            if let Some(value) = value {
                write_be_bytes!(w, value);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GcParams {
    values: ListOfValues<23>,
}

impl GcParams {
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
    pub values: GcParams,
}

impl BeBytes for CreateGC {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_be_bytes!(w, opcodes::CREATE_GC);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 4u16 + n); // length
        write_be_bytes!(w, self.cid.id().value());
        write_be_bytes!(w, self.drawable.value());
        write_be_bytes!(w, bitmask);
        self.values.values.to_be_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for CreateGC {}

#[derive(Debug, Clone)]
pub struct ChangeGC {
    pub gcontext: GContext,
    pub values: GcParams,
}

impl BeBytes for ChangeGC {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_be_bytes!(w, opcodes::CHANGE_GC);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 3 + n); // length
        write_be_bytes!(w, self.gcontext.id().value());
        write_be_bytes!(w, bitmask);
        self.values.values.to_be_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for ChangeGC {}

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl BeBytes for GetInputFocus {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_be_bytes!(w, &opcodes::GET_INPUT_FOCUS);
        write_be_bytes!(w, &0u8); // unused
        write_be_bytes!(w, &1u16); // length

        Ok(())
    }
}

impl XRequest for GetInputFocus {}
