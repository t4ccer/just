use requests::ChangeGC;

use crate::{
    connection::{XConnection, XConnectionRead},
    error::Error,
    requests::{
        CreateGc, CreateWindow, GcParams, InitializeConnection, MapWindow, PolyFillRectangle,
    },
    utils::*,
    xauth::XAuth,
};
use std::{
    fmt::Display,
    io::{self, Write},
    mem,
    num::NonZeroU32,
};

pub mod connection;
pub mod error;
pub mod requests;
mod utils;
pub mod xauth;
pub mod xerror;

pub trait BeBytes: Sized {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()>;
}

pub trait XResponse: Sized {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error>;
}

#[derive(Debug)]
pub enum InitializeConnectionResponse {
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
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub reason: Vec<u8>,
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
        let request = MapWindow { window: self };
        display.connection.send_request(&request)?;
        Ok(())
    }

    pub fn create_gc(self, display: &mut XDisplay, values: GcParams) -> Result<GContext, Error> {
        let cid = GContext(display.allocate_new_id());

        let request = CreateGc {
            cid,
            drawable: Drawable::Window(self),
            values,
        };
        display.connection.send_request(&request)?;

        Ok(cid)
    }

    pub fn draw_rectangle(
        self,
        display: &mut XDisplay,
        gc: GContext,
        rectangle: Rectangle,
    ) -> Result<(), Error> {
        let request = PolyFillRectangle {
            drawable: Drawable::Window(self),
            gc,
            rectangles: vec![rectangle],
        };
        display.connection.send_request(&request)?;
        Ok(())
    }
}

#[derive(Debug)]
#[repr(u8)]
pub enum VisualClass {
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
    pub id: u32,
    pub class: VisualClass,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
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
    pub depth: u8,
    pub visuals: Vec<Visual>,
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
pub enum BackingStore {
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
    pub root: Window,
    pub default_colormat: u32,
    pub white_pixel: u32,
    pub black_pixel: u32,
    pub current_input_masks: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
    pub min_installed_maps: u16,
    pub max_installed_maps: u16,
    pub root_visual: u32,
    pub backing_stores: BackingStore,
    pub save_unders: bool,
    pub root_depth: u8,
    pub allowed_depths: Vec<Depth>,
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
    pub depth: u8,
    pub bits_per_pixel: u8,
    pub scanline_pad: u8,
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
    pub protocol_major_version: u16,
    pub protocol_minor_version: u16,
    pub release_number: u32,
    pub resource_id_base: u32,
    pub resource_id_mask: u32,
    pub motion_buffer_size: u32,
    pub maximum_request_length: u16,
    pub image_byte_order: u8,
    pub bitmap_format_byte_order: u8,
    pub bitmap_format_scanline_unit: u8,
    pub bitmap_format_scanline_pad: u8,
    pub min_keycode: u8,
    pub max_keycode: u8,
    pub vendor: Vec<u8>,
    pub pixmap_formats: Vec<Format>,
    pub screens: Vec<Screen>,
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
pub enum WindowClass {
    CopyFromParent = 0,
    InputOutput = 1,
    InputOnly = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct VisualId(NonZeroU32);

#[derive(Debug, Clone, Copy)]
pub enum WindowVisual {
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

#[derive(Debug, Clone, Copy)]
pub enum Drawable {
    Window(Window),
    Pixmap(u32), // TODO: Pixmap type
}

impl Drawable {
    fn value(self) -> u32 {
        match self {
            Drawable::Window(window) => window.0,
            Drawable::Pixmap(pixmap) => pixmap,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl BeBytes for Rectangle {
    fn to_be_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        w.write_all(&self.x.to_be_bytes())?;
        w.write_all(&self.y.to_be_bytes())?;
        w.write_all(&self.width.to_be_bytes())?;
        w.write_all(&self.height.to_be_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GContext(u32);

impl GContext {
    pub fn change(self, display: &mut XDisplay, settings: GcParams) -> Result<(), Error> {
        let request = ChangeGC {
            gcontext: self,
            values: settings,
        };
        display.connection.send_request(&request)?;
        Ok(())
    }
}

pub struct XDisplay {
    pub response: InitializeConnectionResponseSuccess,
    pub connection: XConnection,
    pub next_id: u32,
}

impl XDisplay {
    pub fn open() -> Result<Self, Error> {
        let mut connection = XConnection::from_env()?;
        let auth = XAuth::from_env()?;

        let init = InitializeConnection {
            major_version: 11,
            minor_version: 0,
            authorization_protocol_name: auth.name,
            authorization_protocol_data: auth.data,
        };
        connection.send_request(&init)?;
        connection.flush()?;

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
            next_id: 1,
        })
    }

    fn allocate_new_id(&mut self) -> u32 {
        let new_part = self.response.resource_id_mask & self.next_id;
        self.next_id += 1;

        // FIXME: Robust id generation
        assert!(new_part != self.response.resource_id_mask);

        self.response.resource_id_base | new_part
    }

    pub fn create_window(&mut self) -> Result<Window, Error> {
        let wid = Window(self.allocate_new_id());
        let create_window = CreateWindow {
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
