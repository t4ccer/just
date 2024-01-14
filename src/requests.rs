use crate::{
    utils::pad, BeBytes, Drawable, GContext, Rectangle, Window, WindowClass, WindowVisual,
};
use std::io::{self, Write};

mod opcodes;

pub trait XRequest: BeBytes {}

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
        w.write_all(&self.major_version.to_be_bytes())?;
        w.write_all(&self.minor_version.to_be_bytes())?;
        w.write_all(&(n as u16).to_be_bytes())?;
        w.write_all(&(d as u16).to_be_bytes())?;
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
        w.write_all(&opcodes::CREATE_WINDOW.to_be_bytes())?;
        w.write_all(&self.depth.to_be_bytes())?;
        w.write_all(&8u16.to_be_bytes())?; // TODO: values
        w.write_all(&self.wid.0.to_be_bytes())?;
        w.write_all(&self.parent.0.to_be_bytes())?;
        w.write_all(&self.x.to_be_bytes())?;
        w.write_all(&self.y.to_be_bytes())?;
        w.write_all(&self.width.to_be_bytes())?;
        w.write_all(&self.height.to_be_bytes())?;
        w.write_all(&self.border_width.to_be_bytes())?;
        w.write_all(&(self.window_class as u16).to_be_bytes())?;
        w.write_all(&self.visual.value().to_be_bytes())?;
        w.write_all(&0u32.to_be_bytes())?; // TODO: values

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
        w.write_all(&opcodes::MAP_WINDOW.to_be_bytes())?;
        w.write_all(&0u8.to_be_bytes())?; // unused
        w.write_all(&2u16.to_be_bytes())?; // size
        w.write_all(&self.window.0.to_be_bytes())?;

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

        w.write_all(&opcodes::POLY_FILL_RECTANGLE.to_be_bytes())?;
        w.write_all(&0u8.to_be_bytes())?; // unused
        w.write_all(&(3 + 2 * n).to_be_bytes())?; // request length
        w.write_all(&self.drawable.value().to_be_bytes())?;
        w.write_all(&self.gc.0.to_be_bytes())?;
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
}

impl_raw_field!(GcParams, function, set_function, 0);
impl_raw_field!(GcParams, plane_mask, set_plane_mask, 1);
impl_raw_field!(GcParams, foreground, set_foreground, 2);
impl_raw_field!(GcParams, background, set_background, 3);
impl_raw_field!(GcParams, line_width, set_line_width, 4);
impl_raw_field!(GcParams, line_style, set_line_style, 5);
impl_raw_field!(GcParams, cap_style, set_cap_style, 6);
impl_raw_field!(GcParams, join_style, set_join_style, 7);
impl_raw_field!(GcParams, fill_style, set_fill_style, 8);
impl_raw_field!(GcParams, fill_rule, set_fill_rule, 9);
impl_raw_field!(GcParams, tile, set_tile, 10);
impl_raw_field!(GcParams, stipple, set_stipple, 11);
impl_raw_field!(
    GcParams,
    tile_stipple_x_origin,
    set_tile_stipple_x_origin,
    12
);
impl_raw_field!(
    GcParams,
    tile_stipple_y_origin,
    set_tile_stipple_y_origin,
    13
);
impl_raw_field!(GcParams, font, set_font, 14);
impl_raw_field!(GcParams, subwindow_mode, set_subwindow_mode, 15);
impl_raw_field!(GcParams, graphics_exposures, set_graphics_exposures, 16);
impl_raw_field!(GcParams, clip_x_origin, set_clip_x_origin, 19);
impl_raw_field!(GcParams, clip_y_origin, set_clip_y_origin, 18);
impl_raw_field!(GcParams, clip_mask, set_clip_mask, 19);
impl_raw_field!(GcParams, dash_offset, set_dash_offset, 20);
impl_raw_field!(GcParams, dashes, set_dashes, 21);
impl_raw_field!(GcParams, arc_mode, set_arc_mode, 22);

impl GcParams {
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
                w.write_all(&value.to_be_bytes())?;
            }
        }

        Ok(())
    }
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

        w.write_all(&opcodes::CREATE_GC.to_be_bytes())?;
        w.write_all(&0u8.to_be_bytes())?; // unused
        w.write_all(&(4u16 + n).to_be_bytes())?; // length
        w.write_all(&self.cid.0.to_be_bytes())?;
        w.write_all(&self.drawable.value().to_be_bytes())?;
        w.write_all(&bitmask.to_be_bytes())?;
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

        w.write_all(&opcodes::CHANGE_GC.to_be_bytes())?;
        w.write_all(&0u8.to_be_bytes())?; // unused
        w.write_all(&(3 + n).to_be_bytes())?; // length
        w.write_all(&self.gcontext.0.to_be_bytes())?;
        w.write_all(&bitmask.to_be_bytes())?;
        self.values.to_be_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for ChangeGC {}

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl BeBytes for GetInputFocus {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&opcodes::GET_INPUT_FOCUS.to_be_bytes())?;
        w.write_all(&0u8.to_be_bytes())?; // unused
        w.write_all(&1u16.to_be_bytes())?; // length

        Ok(())
    }
}

impl XRequest for GetInputFocus {}
