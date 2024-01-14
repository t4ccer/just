#![allow(dead_code)]

use crate::{
    connection::{XConnection, XConnectionRead},
    error::Error,
    utils::*,
    xauth::XAuth,
    xerror::XError,
};
use std::{fmt::Display, io::Read, mem, num::NonZeroU32};

pub mod connection;
pub mod error;
mod utils;
pub mod xauth;
pub mod xerror;

pub trait XRequest: Sized {
    fn to_be_bytes(&self) -> Vec<u8>;
}

pub trait XResponse: Sized {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error>;
}

#[derive(Debug)]
pub struct InitializeConnectionRequest {
    major_version: u16,
    minor_version: u16,
    authorization_protocol_name: Vec<u8>,
    authorization_protocol_data: Vec<u8>,
}

impl XRequest for InitializeConnectionRequest {
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

#[derive(Debug)]
enum InitializeConnectionResponse {
    Refused(InitializeConnectionResponseRefused),
    Success(InitializeConnectionResponseSuccess),
}

impl XResponse for InitializeConnectionResponse {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let response_code = conn.read_u8()?;
        match response_code {
            0 => Ok(Self::Refused(
                InitializeConnectionResponseRefused::from_be_bytes(conn)?,
            )),
            1 => Ok(Self::Success(
                InitializeConnectionResponseSuccess::from_be_bytes(conn)?,
            )),
            2 => todo!("Authenticate"),
            _ => Err(Error::InvalidResponse),
        }
    }
}

#[derive(Debug)]
pub struct InitializeConnectionResponseRefused {
    protocol_major_version: u16,
    protocol_minor_version: u16,
    reason: Vec<u8>,
}

impl XResponse for InitializeConnectionResponseRefused {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let reason_length = conn.read_u8()?;
        let protocol_major_version = conn.read_u16_be()?;
        let protocol_minor_version = conn.read_u16_be()?;
        let _ = conn.read_u16_be()?;
        let reason = conn.read_n_bytes(reason_length as usize)?;
        let _pad = conn.read_n_bytes(pad(reason_length as usize))?;
        Ok(Self {
            protocol_major_version,
            protocol_minor_version,
            reason,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Window(u32);

impl Window {
    pub fn map(self, display: &mut XDisplay) -> Result<(), Error> {
        let request = MapWindowRequest { window: self };
        display.connection.send_request(&request)?;
        Ok(())
    }
}

#[derive(Debug)]
#[repr(u8)]
enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PseudoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl TryFrom<u8> for VisualClass {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 5 {
            return Err(value);
        }
        Ok(unsafe { mem::transmute(value) })
    }
}

#[derive(Debug)]
pub struct Visual {
    id: u32,
    class: VisualClass,
    bits_per_rgb_value: u8,
    colormap_entries: u16,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

impl Visual {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let id = conn.read_u32_be()?;
        let class = VisualClass::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let bits_per_rgb_value = conn.read_u8()?;
        let colormap_entries = conn.read_u16_be()?;
        let red_mask = conn.read_u32_be()?;
        let green_mask = conn.read_u32_be()?;
        let blue_mask = conn.read_u32_be()?;
        let _unused = conn.read_u32_be()?;
        Ok(Self {
            id,
            class,
            bits_per_rgb_value,
            colormap_entries,
            red_mask,
            green_mask,
            blue_mask,
        })
    }
}

#[derive(Debug)]
pub struct Depth {
    depth: u8,
    visuals: Vec<Visual>,
}

impl Depth {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _unused = conn.read_u8()?;
        let visuals_length = conn.read_u16_be()?;
        let _unused = conn.read_u32_be()?;
        let visuals = conn.read_many(visuals_length as usize, Visual::from_be_bytes)?;
        Ok(Self { depth, visuals })
    }
}

#[derive(Debug)]
#[repr(u8)]
enum BackingStore {
    NotUseful = 0,
    WhenMapped = 1,
    Always = 2,
}

impl TryFrom<u8> for BackingStore {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 2 {
            return Err(value);
        }

        Ok(unsafe { mem::transmute(value) })
    }
}

#[derive(Debug)]
pub struct Screen {
    root: Window,
    default_colormat: u32,
    white_pixel: u32,
    black_pixel: u32,
    current_input_masks: u32,
    width_in_pixels: u16,
    height_in_pixels: u16,
    width_in_millimeters: u16,
    height_in_millimeters: u16,
    min_installed_maps: u16,
    max_installed_maps: u16,
    root_visual: u32,
    backing_stores: BackingStore,
    save_unders: bool,
    root_depth: u8,
    allowed_depths: Vec<Depth>,
}

impl Screen {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let root = Window(conn.read_u32_be()?);
        let default_colormat = conn.read_u32_be()?;
        let white_pixel = conn.read_u32_be()?;
        let black_pixel = conn.read_u32_be()?;
        let current_input_masks = conn.read_u32_be()?;
        let width_in_pixels = conn.read_u16_be()?;
        let height_in_pixels = conn.read_u16_be()?;
        let width_in_millimeters = conn.read_u16_be()?;
        let height_in_millimeters = conn.read_u16_be()?;
        let min_installed_maps = conn.read_u16_be()?;
        let max_installed_maps = conn.read_u16_be()?;
        let root_visual = conn.read_u32_be()?;
        let backing_stores =
            BackingStore::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let save_unders = conn.read_u8()? == 1;
        let root_depth = conn.read_u8()?;
        let allowed_depths_lenght = conn.read_u8()?;
        let allowed_depths =
            conn.read_many(allowed_depths_lenght as usize, Depth::from_be_bytes)?;

        Ok(Self {
            root,
            default_colormat,
            white_pixel,
            black_pixel,
            current_input_masks,
            width_in_pixels,
            height_in_pixels,
            width_in_millimeters,
            height_in_millimeters,
            min_installed_maps,
            max_installed_maps,
            root_visual,
            backing_stores,
            save_unders,
            root_depth,
            allowed_depths,
        })
    }
}

#[derive(Debug)]
pub struct Format {
    depth: u8,
    bits_per_pixel: u8,
    scanline_pad: u8,
}

impl Format {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let mut format = [0u8; 8];
        conn.read_exact(&mut format)?;
        Ok(Format {
            depth: format[0],
            bits_per_pixel: format[1],
            scanline_pad: format[2],
        })
    }
}

#[derive(Debug)]
pub struct InitializeConnectionResponseSuccess {
    protocol_major_version: u16,
    protocol_minor_version: u16,
    release_number: u32,
    resource_id_base: u32,
    resource_id_mask: u32,
    motion_buffer_size: u32,
    maximum_request_length: u16,
    image_byte_order: u8,
    bitmap_format_byte_order: u8,
    bitmap_format_scanline_unit: u8,
    bitmap_format_scanline_pad: u8,
    min_keycode: u8,
    max_keycode: u8,
    vendor: Vec<u8>,
    pixmap_formats: Vec<Format>,
    screens: Vec<Screen>,
}

impl XResponse for InitializeConnectionResponseSuccess {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let protocol_major_version = conn.read_u16_be()?;
        let protocol_minor_version = conn.read_u16_be()?;
        let _ = conn.read_u16_be()?;
        let release_number = conn.read_u32_be()?;
        let resource_id_base = conn.read_u32_be()?;
        let resource_id_mask = conn.read_u32_be()?;
        let motion_buffer_size = conn.read_u32_be()?;
        let vendor_length = conn.read_u16_be()?;
        let maximum_request_length = conn.read_u16_be()?;
        let screens_length = conn.read_u8()?;
        let formats_length = conn.read_u8()?;
        let image_byte_order = conn.read_u8()?;
        let bitmap_format_byte_order = conn.read_u8()?;
        let bitmap_format_scanline_unit = conn.read_u8()?;
        let bitmap_format_scanline_pad = conn.read_u8()?;
        let min_keycode = conn.read_u8()?;
        let max_keycode = conn.read_u8()?;
        let _unused = conn.read_u32_be()?;
        let vendor = conn.read_n_bytes(vendor_length as usize)?;
        let _pad = conn.read_n_bytes(pad(vendor_length as usize))?;
        let pixmap_formats = conn.read_many(formats_length as usize, Format::from_be_bytes)?;
        let screens = conn.read_many(screens_length as usize, Screen::from_be_bytes)?;

        Ok(Self {
            protocol_major_version,
            protocol_minor_version,
            release_number,
            resource_id_base,
            resource_id_mask,
            motion_buffer_size,
            maximum_request_length,
            image_byte_order,
            bitmap_format_byte_order,
            bitmap_format_scanline_unit,
            bitmap_format_scanline_pad,
            min_keycode,
            max_keycode,
            vendor,
            pixmap_formats,
            screens,
        })
    }
}

impl Display for InitializeConnectionResponseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Initialize Connection Response Refused: Protocol: {}.{}. Reason: {}",
            self.protocol_major_version,
            self.protocol_minor_version,
            display_maybe_utf8(&self.reason)
        )
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct VisualId(NonZeroU32);

#[derive(Debug, Clone, Copy)]
enum WindowVisual {
    CopyFromParent,
    Id(VisualId),
}

impl WindowVisual {
    pub fn value(self) -> u32 {
        match self {
            WindowVisual::CopyFromParent => 0,
            WindowVisual::Id(vid) => vid.0.get(),
        }
    }
}

#[derive(Debug)]
pub struct CreateWindowRequest {
    depth: u8,
    wid: Window,
    parent: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    border_width: u16,
    window_class: WindowClass,
    visual: WindowVisual,
    // TODO: values
}

impl XRequest for CreateWindowRequest {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::new();

        request.extend(1u8.to_be_bytes());
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
pub struct MapWindowRequest {
    window: Window,
}

impl XRequest for MapWindowRequest {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::with_capacity(8);

        request.extend(8u8.to_be_bytes());
        request.extend(0u8.to_be_bytes());
        request.extend(2u16.to_be_bytes());
        request.extend(self.window.0.to_be_bytes());

        request
    }
}

pub struct XDisplay {
    response: InitializeConnectionResponseSuccess,
    connection: XConnection,
    next_id: u32,
}

impl XDisplay {
    pub fn open() -> Result<Self, Error> {
        let mut connection = XConnection::from_env()?;
        let auth = XAuth::from_env()?;

        let init = InitializeConnectionRequest {
            major_version: 11,
            minor_version: 0,
            authorization_protocol_name: auth.name,
            authorization_protocol_data: auth.data,
        };
        connection.send_request(&init)?;

        let response = connection.read_response::<InitializeConnectionResponse>()?;
        let response = match response {
            InitializeConnectionResponse::Refused(response) => {
                return Err(Error::CouldNotOpenDisplay(response));
            }
            InitializeConnectionResponse::Success(response) => response,
        };

        Ok(Self {
            response,
            connection,
            next_id: 0,
        })
    }

    fn allocate_new_id(&mut self) -> u32 {
        let ret = self.response.resource_id_base | (self.response.resource_id_mask & self.next_id);
        self.next_id += 1;
        ret
    }

    pub fn create_window(&mut self) -> Result<Window, Error> {
        let wid = Window(self.allocate_new_id());
        let create_window = CreateWindowRequest {
            depth: self.response.screens[0].allowed_depths[0].depth,
            wid,
            parent: self.response.screens[0].root,
            x: 0,
            y: 0,
            width: 800,
            height: 600,
            border_width: 0,
            window_class: WindowClass::CopyFromParent,
            visual: WindowVisual::CopyFromParent,
        };
        self.connection.send_request(&create_window)?;

        Ok(wid)
    }
}

struct GetInputFocusRequest;

impl XRequest for GetInputFocusRequest {
    fn to_be_bytes(&self) -> Vec<u8> {
        let mut request = Vec::new();

        request.extend(43u8.to_be_bytes());
        request.extend(0u8.to_be_bytes());
        request.extend(1u16.to_be_bytes());

        request
    }
}

pub fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;
    let window = display.create_window()?;
    window.map(&mut display)?;

    // display.connection.send_request(&GetInputFocusRequest)?;

    // let err = display.connection.read_response::<XError>()?;
    // dbg!(&err);

    // let mut buf = [0; 0x100];
    // display.connection.read_end.inner.read(&mut buf)?;
    // dbg!(&buf);

    std::thread::sleep(std::time::Duration::from_secs(5));

    Ok(())
}
