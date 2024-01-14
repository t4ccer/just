use crate::{utils::pad, Drawable, GContext, Rectangle, Window, WindowClass, WindowVisual};

mod opcodes;

pub trait XRequest: Sized {
    fn to_be_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, Clone)]
pub struct InitializeConnection {
    pub major_version: u16,
    pub minor_version: u16,
    pub authorization_protocol_name: Vec<u8>,
    pub authorization_protocol_data: Vec<u8>,
}

impl XRequest for InitializeConnection {
    fn to_be_bytes(&self) -> Vec<u8> {
        let n = self.authorization_protocol_name.len();
        let p = pad(n);
        let d = self.authorization_protocol_data.len();
        let q = pad(d);
        let mut bytes = Vec::<u8>::with_capacity(10 + n + p + d + q);

        bytes.extend(b"B\0");
        bytes.extend(self.major_version.to_be_bytes());
        bytes.extend(self.minor_version.to_be_bytes());
        bytes.extend((n as u16).to_be_bytes());
        bytes.extend((d as u16).to_be_bytes());
        bytes.extend([0u8; 2]); // unused
        bytes.extend(&self.authorization_protocol_name);
        bytes.extend(vec![0u8; p]); // unused, pad
        bytes.extend(&self.authorization_protocol_data);
        bytes.extend(vec![0u8; q]); // unused, pad

        bytes
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
    // TODO: values
}

impl XRequest for CreateWindow {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::new();

        request.extend(opcodes::CREATE_WINDOW.to_be_bytes());
        request.extend(self.depth.to_be_bytes());
        request.extend(8u16.to_be_bytes()); // TODO: values
        request.extend(self.wid.0.to_be_bytes());
        request.extend(self.parent.0.to_be_bytes());
        request.extend(self.x.to_be_bytes());
        request.extend(self.y.to_be_bytes());
        request.extend(self.width.to_be_bytes());
        request.extend(self.height.to_be_bytes());
        request.extend(self.border_width.to_be_bytes());
        request.extend((self.window_class as u16).to_be_bytes());
        request.extend(self.visual.value().to_be_bytes());
        request.extend(0u32.to_be_bytes()); // TODO: values

        request
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapWindow {
    pub window: Window,
}

impl XRequest for MapWindow {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::with_capacity(8);

        request.extend(opcodes::MAP_WINDOW.to_be_bytes());
        request.extend(0u8.to_be_bytes()); // unused
        request.extend(2u16.to_be_bytes()); // size
        request.extend(self.window.0.to_be_bytes());

        request
    }
}

#[derive(Debug, Clone)]
pub struct PolyFillRectangle {
    pub drawable: Drawable,
    pub gc: GContext,
    pub rectangles: Vec<Rectangle>,
}

impl XRequest for PolyFillRectangle {
    fn to_be_bytes(&self) -> Vec<u8> {
        let n: u16 = self.rectangles.len() as u16;

        let mut request = Vec::with_capacity(n as usize);

        request.extend(opcodes::POLY_FILL_RECTANGLE.to_be_bytes());
        request.extend(0u8.to_be_bytes()); // unused
        request.extend((3 + 2 * n).to_be_bytes()); // request length
        request.extend(self.drawable.value().to_be_bytes());
        request.extend(self.gc.0.to_be_bytes());
        self.rectangles
            .iter()
            .for_each(|rectangle| request.extend(rectangle.to_be_bytes()));

        request
    }
}

macro_rules! impl_raw_field {
    ($struct:path, $getter:ident, $setter:ident, $idx:expr) => {
        #[allow(dead_code)]
        impl $struct {
            pub fn $getter(&self) -> Option<u32> {
                self.raw_values[$idx]
            }

            pub fn $setter(&mut self, new_value: u32) {
                self.raw_values[$idx] = Some(new_value);
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

#[derive(Debug, Clone)]
pub struct CreateGc {
    pub cid: GContext,
    pub drawable: Drawable,
    pub values: GcParams,
}

impl XRequest for CreateGc {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::new();

        let mut n: u16 = 0;
        let mut bitmask: u32 = 0;
        for value in self.values.raw_values.iter().rev() {
            bitmask <<= 1;
            bitmask |= value.is_some() as u32;
            n += value.is_some() as u16;
        }

        request.extend(opcodes::CREATE_GC.to_be_bytes());
        request.extend(0u8.to_be_bytes()); // unused
        request.extend((4u16 + n).to_be_bytes()); // length
        request.extend(self.cid.0.to_be_bytes());
        request.extend(self.drawable.value().to_be_bytes());
        request.extend(bitmask.to_be_bytes());

        for value in self.values.raw_values {
            if let Some(value) = value {
                request.extend(value.to_be_bytes());
            }
        }

        request
    }
}

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl XRequest for GetInputFocus {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::new();

        request.extend(opcodes::GET_INPUT_FOCUS.to_be_bytes());
        request.extend(0u8.to_be_bytes()); // unused
        request.extend(1u16.to_be_bytes()); // length

        request
    }
}
