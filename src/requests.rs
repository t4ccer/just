use crate::{
    utils::pad, BeBytes, Drawable, GContext, Rectangle, Window, WindowClass, WindowVisual,
};
use std::io::{self, Write};

mod opcodes;

pub trait XRequest: BeBytes {}

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
        let n = self.authorization_protocol_name.len();
        let p = pad(n);
        let d = self.authorization_protocol_data.len();
        let q = pad(d);

        w.write_all(b"B\0")?;
        write_be_bytes!(w, self.major_version);
        write_be_bytes!(w, self.minor_version);
        write_be_bytes!(w, n as u16);
        write_be_bytes!(w, d as u16);
        w.write_all(&[0u8; 2])?; // unused
        w.write_all(&self.authorization_protocol_name)?;
        w.write_all(&vec![0u8; p])?; // unused, pad
        w.write_all(&self.authorization_protocol_data)?;
        w.write_all(&vec![0u8; q])?; // unused, pad

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
        write_be_bytes!(w, self.wid.0);
        write_be_bytes!(w, self.parent.0);
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
pub struct MapWindow {
    pub window: Window,
}

impl BeBytes for MapWindow {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_be_bytes!(w, opcodes::MAP_WINDOW);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 2u16); // size
        write_be_bytes!(w, self.window.0);

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
        write_be_bytes!(w, self.gc.0);
        for rectangle in &self.rectangles {
            rectangle.to_be_bytes(w)?;
        }

        Ok(())
    }
}

impl XRequest for PolyFillRectangle {}

macro_rules! impl_raw_field {
    ($struct:path, $getter:ident, $setter:ident, $idx:expr) => {
        #[allow(dead_code)]
        impl $struct {
            pub fn $getter(&self) -> Option<u32> {
                self.raw_values[$idx]
            }

            pub fn $setter(mut self, new_value: u32) -> Self {
                self.raw_values[$idx] = Some(new_value);
                self
            }
        }
    };
}

macro_rules! impl_raw_fields_go {
    ($struct:path, $idx:expr, $getter:ident, $setter:ident $(,)?) => {
        impl_raw_field!($struct, $getter, $setter, $idx);
    };

    ($struct:path, $idx:expr, $getter:ident, $setter:ident, $($rest:tt)*) => {
        impl_raw_field!($struct, $getter, $setter, $idx);
        impl_raw_fields_go!($struct, $idx + 1, $($rest)*);
    };
}

macro_rules! impl_raw_fields {
    ($struct:path, $($rest:tt)*) => {
        impl_raw_fields_go!($struct, 0, $($rest)*);
    };
}

#[derive(Debug, Clone)]
pub struct GcParams {
    raw_values: [Option<u32>; 23],
}

impl GcParams {
    pub fn new() -> Self {
        Self {
            raw_values: [None; 23],
        }
    }

    pub fn mask_and_count(&self) -> (u32, u16) {
        let mut bitmask: u32 = 0;
        let mut n: u16 = 0;

        for value in self.raw_values.iter().rev() {
            bitmask <<= 1;
            bitmask |= value.is_some() as u32;
            n += value.is_some() as u16;
        }

        (bitmask, n)
    }

    pub fn to_be_bytes_if_set(&self, w: &mut impl Write) -> io::Result<()> {
        for value in self.raw_values {
            if let Some(value) = value {
                write_be_bytes!(w, value);
            }
        }

        Ok(())
    }
}

impl_raw_fields! {GcParams,
  function, set_function,
  plane_mask, set_plane_mask,
  foreground, set_foreground,
  background, set_background,
  line_width, set_line_width,
  line_style, set_line_style,
  cap_style, set_cap_style,
  join_style, set_join_style,
  fill_style, set_fill_style,
  fill_rule, set_fill_rule,
  tile, set_tile,
  stipple, set_stipple,
  tile_stipple_x_origin, set_tile_stipple_x_origin,
  tile_stipple_y_origin, set_tile_stipple_y_origin,
  font, set_font,
  subwindow_mode, set_subwindow_mode,
  graphics_exposures, set_graphics_exposures,
  clip_x_origin, set_clip_x_origin,
  clip_y_origin, set_clip_y_origin,
  clip_mask, set_clip_mask,
  dash_offset, set_dash_offset,
  dashes, set_dashes,
  arc_mode, set_arc_mode,
}

#[derive(Debug, Clone)]
pub struct CreateGc {
    pub cid: GContext,
    pub drawable: Drawable,
    pub values: GcParams,
}

impl BeBytes for CreateGc {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.mask_and_count();

        write_be_bytes!(w, opcodes::CREATE_GC);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 4u16 + n); // length
        write_be_bytes!(w, self.cid.0);
        write_be_bytes!(w, self.drawable.value());
        write_be_bytes!(w, bitmask);
        self.values.to_be_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for CreateGc {}

#[derive(Debug, Clone)]
pub struct ChangeGC {
    pub gcontext: GContext,
    pub values: GcParams,
}

impl BeBytes for ChangeGC {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.mask_and_count();

        write_be_bytes!(w, opcodes::CHANGE_GC);
        write_be_bytes!(w, 0u8); // unused
        write_be_bytes!(w, 3 + n); // length
        write_be_bytes!(w, self.gcontext.0);
        write_be_bytes!(w, bitmask);
        self.values.to_be_bytes_if_set(w)?;

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
