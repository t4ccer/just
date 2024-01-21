#![allow(dead_code)]

use crate::x11::{
    connection::{ConnectionKind, XConnection},
    error::Error,
    events::Event,
    replies::{AwaitingReply, Geometry, Reply, ReplyType, WindowAttributes},
    requests::{
        ChangeGC, CreateGC, CreateWindow, GContextSettings, GetGeometry, GetWindowAttributes,
        InitializeConnection, MapWindow, PolyFillRectangle, WindowCreationAttributes, XRequest,
    },
    utils::*,
    xauth::XAuth,
    xerror::XError,
};
use std::{
    collections::{vec_deque::Drain, HashMap, VecDeque},
    fmt::Display,
    io::{self, Write},
    mem,
};

pub mod connection;
pub mod error;
pub mod events;
pub mod replies;
pub mod requests;
mod utils;
pub mod xauth;
pub mod xerror;

pub trait LeBytes: Sized {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()>;
}

pub trait XResponse: Sized {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ResourceId {
    value: u32,
}

impl ResourceId {
    pub fn value(self) -> u32 {
        self.value
    }
}

macro_rules! impl_resource_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name(ResourceId);

        impl $name {
            pub fn id(self) -> ResourceId {
                self.0
            }

            pub fn to_le_bytes(self) -> [u8; 4] {
                let raw: u32 = self.into();
                raw.to_le_bytes()
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> u32 {
                value.0.value()
            }
        }
    };
}

impl_resource_id!(Pixmap);
impl_resource_id!(VisualId);
impl_resource_id!(Font);
impl_resource_id!(Atom);
impl_resource_id!(Colormap);
impl_resource_id!(Cursor);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct OrNone<T>(T);

impl<T> OrNone<T>
where
    T: Into<u32> + Copy,
{
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn value(self) -> Option<T> {
        if self.0.into() == 0u32 {
            None
        } else {
            Some(self.0)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IdAllocator {
    id_base: u32,
    id_mask: u32,
    next_id: u32,
}

impl IdAllocator {
    pub fn new(id_base: u32, id_mask: u32) -> Self {
        Self {
            id_base,
            id_mask,
            next_id: 1,
        }
    }

    pub fn allocate_id(&mut self) -> ResourceId {
        let new_part = self.id_mask & (self.next_id << self.id_mask.trailing_zeros());
        self.next_id += 1;

        assert_ne!(new_part, 0, "Invalid ID allocated");

        ResourceId {
            value: self.id_base | new_part,
        }
    }
}

#[derive(Debug)]
pub enum InitializeConnectionResponse {
    Refused(InitializeConnectionResponseRefused),
    Success(InitializeConnectionResponseSuccess),
}

impl XResponse for InitializeConnectionResponse {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let response_code = conn.read_u8()?;
        match response_code {
            0 => Ok(Self::Refused(
                InitializeConnectionResponseRefused::from_le_bytes(conn)?,
            )),
            1 => Ok(Self::Success(
                InitializeConnectionResponseSuccess::from_le_bytes(conn)?,
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
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let reason_length = conn.read_u8()?;
        let protocol_major_version = conn.read_le_u16()?;
        let protocol_minor_version = conn.read_le_u16()?;
        let _ = conn.read_le_u16()?;
        let reason = conn.read_n_bytes(reason_length as usize)?;
        let _pad = conn.read_n_bytes(pad(reason_length as usize))?;
        Ok(Self {
            protocol_major_version,
            protocol_minor_version,
            reason,
        })
    }
}

impl_resource_id!(Window);

impl Window {
    pub fn map(self, display: &mut XDisplay) -> Result<(), Error> {
        let request = MapWindow { window: self };
        display.send_request(&request)?;
        Ok(())
    }

    pub fn create_gc(
        self,
        display: &mut XDisplay,
        values: GContextSettings,
    ) -> Result<GContext, Error> {
        let cid = GContext(display.id_allocator.allocate_id());

        let request = CreateGC {
            cid,
            drawable: Drawable::Window(self),
            values,
        };
        display.send_request(&request)?;

        Ok(cid)
    }

    pub fn draw_rectangle(
        self,
        display: &mut XDisplay,
        gc: GContext,
        rectangle: Rectangle,
    ) -> Result<(), Error> {
        self.draw_rectangles(display, gc, vec![rectangle])
    }

    pub fn draw_rectangles(
        self,
        display: &mut XDisplay,
        gc: GContext,
        rectangles: Vec<Rectangle>,
    ) -> Result<(), Error> {
        let request = PolyFillRectangle {
            drawable: Drawable::Window(self),
            gc,
            rectangles,
        };
        display.send_request(&request)?;
        Ok(())
    }

    pub fn get_attributes(self, display: &mut XDisplay) -> Result<WindowAttributes, Error> {
        let request = GetWindowAttributes { window: self };
        let sequence_number = display.send_request(&request)?;
        display.connection.flush()?;

        let reply = display.await_reply(sequence_number)?;

        if let Reply::GetWindowAttributes(reply) = reply {
            Ok(reply)
        } else {
            panic!("Unexpected reply type");
        }
    }

    pub fn get_geometry(self, display: &mut XDisplay) -> Result<Geometry, Error> {
        let request = GetGeometry {
            drawable: Drawable::Window(self),
        };

        let sequence_number = display.send_request(&request)?;
        display.connection.flush()?;

        let reply = display.await_reply(sequence_number)?;

        if let Reply::GetGeometry(reply) = reply {
            Ok(reply)
        } else {
            panic!("Unexpected reply type");
        }
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
    pub id: VisualId,
    pub class: VisualClass,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
}

impl Visual {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let id = conn.read_le_u32()?;
        let class = VisualClass::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let bits_per_rgb_value = conn.read_u8()?;
        let colormap_entries = conn.read_le_u16()?;
        let red_mask = conn.read_le_u32()?;
        let green_mask = conn.read_le_u32()?;
        let blue_mask = conn.read_le_u32()?;
        let _unused = conn.read_le_u32()?;
        Ok(Self {
            id: VisualId(ResourceId { value: id }),
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
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _unused = conn.read_u8()?;
        let visuals_length = conn.read_le_u16()?;
        let _unused = conn.read_le_u32()?;
        let visuals = conn.read_many(visuals_length as usize, Visual::from_le_bytes)?;
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
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let root = Window(ResourceId {
            value: conn.read_le_u32()?,
        });
        let default_colormat = conn.read_le_u32()?;
        let white_pixel = conn.read_le_u32()?;
        let black_pixel = conn.read_le_u32()?;
        let current_input_masks = conn.read_le_u32()?;
        let width_in_pixels = conn.read_le_u16()?;
        let height_in_pixels = conn.read_le_u16()?;
        let width_in_millimeters = conn.read_le_u16()?;
        let height_in_millimeters = conn.read_le_u16()?;
        let min_installed_maps = conn.read_le_u16()?;
        let max_installed_maps = conn.read_le_u16()?;
        let root_visual = conn.read_le_u32()?;
        let backing_stores =
            BackingStore::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let save_unders = conn.read_u8()? == 1;
        let root_depth = conn.read_u8()?;
        let allowed_depths_lenght = conn.read_u8()?;
        let allowed_depths =
            conn.read_many(allowed_depths_lenght as usize, Depth::from_le_bytes)?;

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
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let protocol_major_version = conn.read_le_u16()?;
        let protocol_minor_version = conn.read_le_u16()?;
        let _ = conn.read_le_u16()?;
        let release_number = conn.read_le_u32()?;
        let resource_id_base = conn.read_le_u32()?;
        let resource_id_mask = conn.read_le_u32()?;
        let motion_buffer_size = conn.read_le_u32()?;
        let vendor_length = conn.read_le_u16()?;
        let maximum_request_length = conn.read_le_u16()?;
        let screens_length = conn.read_u8()?;
        let formats_length = conn.read_u8()?;
        let image_byte_order = conn.read_u8()?;
        let bitmap_format_byte_order = conn.read_u8()?;
        let bitmap_format_scanline_unit = conn.read_u8()?;
        let bitmap_format_scanline_pad = conn.read_u8()?;
        let min_keycode = conn.read_u8()?;
        let max_keycode = conn.read_u8()?;
        let _unused = conn.read_le_u32()?;
        let vendor = conn.read_n_bytes(vendor_length as usize)?;
        let _pad = conn.read_n_bytes(pad(vendor_length as usize))?;
        let pixmap_formats = conn.read_many(formats_length as usize, Format::from_le_bytes)?;
        let screens = conn.read_many(screens_length as usize, Screen::from_le_bytes)?;

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
pub enum WindowVisual {
    CopyFromParent,
    Id(VisualId),
}

impl WindowVisual {
    pub fn value(self) -> u32 {
        match self {
            WindowVisual::CopyFromParent => 0,
            WindowVisual::Id(vid) => vid.id().value(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Drawable {
    Window(Window),
    Pixmap(Pixmap),
}

impl Drawable {
    fn value(self) -> u32 {
        match self {
            Drawable::Window(window) => window.into(),
            Drawable::Pixmap(pixmap) => pixmap.into(),
        }
    }

    pub(crate) fn to_le_bytes(&self) -> [u8; 4] {
        match self {
            Drawable::Window(window) => window.to_le_bytes(),
            Drawable::Pixmap(pixmap) => pixmap.to_le_bytes(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
        unsafe { mem::transmute(self) }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Rectangle {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Rectangle {
    fn to_le_bytes(self) -> [u8; 8] {
        unsafe { mem::transmute(self) }
    }
}

impl_resource_id!(GContext);

impl GContext {
    pub fn change(self, display: &mut XDisplay, settings: GContextSettings) -> Result<(), Error> {
        let request = ChangeGC {
            gcontext: self,
            values: settings,
        };
        display.send_request(&request)?;
        Ok(())
    }
}

pub struct XDisplay {
    pub id_allocator: IdAllocator,
    pub screens: Vec<Screen>,
    pub connection: XConnection,
    awaiting_replies: HashMap<SequenceNumber, AwaitingReply>,
    next_sequence_number: SequenceNumber,
    event_queue: VecDeque<Event>,
    error_queue: VecDeque<XError>,
    pub maximum_request_length: u16,
}

impl XDisplay {
    pub fn open() -> Result<Self, Error> {
        let mut connection = XConnection::from_env()?;
        let (authorization_protocol_name, authorization_protocol_data) = match connection.kind() {
            ConnectionKind::UnixStream => {
                let auth = XAuth::from_env()?;
                (auth.name, auth.data)
            }
        };

        let init = InitializeConnection {
            major_version: 11,
            minor_version: 0,
            authorization_protocol_name,
            authorization_protocol_data,
        };
        connection.send_request(&init)?;
        connection.flush()?;

        let response = InitializeConnectionResponse::from_le_bytes(&mut connection)?;
        let response = match response {
            InitializeConnectionResponse::Refused(response) => {
                return Err(Error::CouldNotOpenDisplay(response));
            }
            InitializeConnectionResponse::Success(response) => response,
        };

        let id_allocator = IdAllocator::new(response.resource_id_base, response.resource_id_mask);

        Ok(Self {
            id_allocator,
            screens: response.screens,
            connection,
            awaiting_replies: HashMap::new(),
            next_sequence_number: SequenceNumber { inner: 1 },
            event_queue: VecDeque::new(),
            error_queue: VecDeque::new(),
            maximum_request_length: response.maximum_request_length,
        })
    }

    pub fn create_simple_window(
        &mut self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
        border_width: u16,
        attributes: WindowCreationAttributes,
    ) -> Result<Window, Error> {
        let new_window_id = Window(self.id_allocator.allocate_id());
        let create_window = CreateWindow {
            depth: self.screens[0].allowed_depths[0].depth,
            wid: new_window_id,
            parent: self.screens[0].root,
            x,
            y,
            width,
            height,
            border_width,
            window_class: WindowClass::CopyFromParent,
            visual: WindowVisual::CopyFromParent,
            attributes,
        };
        self.send_request(&create_window)?;

        Ok(new_window_id)
    }

    pub fn send_request<R: XRequest>(&mut self, request: &R) -> Result<SequenceNumber, Error> {
        let this_sequence_number = self.next_sequence_number.inner;
        self.next_sequence_number = SequenceNumber {
            inner: self.next_sequence_number.inner.wrapping_add(1),
        };

        self.connection.send_request(request)?;

        if let Some(reply_type) = R::reply_type() {
            let sequence_number_exists = self
                .awaiting_replies
                .insert(
                    SequenceNumber {
                        inner: this_sequence_number,
                    },
                    AwaitingReply::NotReceived(reply_type),
                )
                .is_none();
            assert!(sequence_number_exists);
        }

        Ok(SequenceNumber {
            inner: this_sequence_number,
        })
    }

    pub fn await_reply(&mut self, awaited: SequenceNumber) -> Result<Reply, Error> {
        loop {
            if let Some((awaited, entry)) = self.awaiting_replies.remove_entry(&awaited) {
                match entry {
                    AwaitingReply::Received(reply) => {
                        return Ok(reply);
                    }
                    reply => {
                        self.awaiting_replies.insert(awaited, reply);
                    }
                }
            }
            self.decode_response_blocking()?;
        }
    }

    pub fn discard_reply(&mut self, to_discard: SequenceNumber) -> Result<(), Error> {
        let entry = self
            .awaiting_replies
            .get(&to_discard)
            .expect("Sequence number must be known");

        match entry {
            &AwaitingReply::NotReceived(ty) => self
                .awaiting_replies
                .insert(to_discard, AwaitingReply::Discarded(ty)),
            AwaitingReply::Discarded(_) => unreachable!("Discarded sequence number twice"),
            AwaitingReply::Received(_) => self.awaiting_replies.remove(&to_discard),
        };

        Ok(())
    }

    fn decode_response_blocking(&mut self) -> Result<(), Error> {
        let code: u8 = self.connection.read_u8()?;
        match code {
            0 => {
                let error_code: u8 = self.connection.read_u8()?;
                let error = XError::from_le_bytes(&mut self.connection, error_code)?;
                self.error_queue.push_back(error);
            }
            1 => {
                // TODO: Try to avoid using peek
                let sequence_number: SequenceNumber = SequenceNumber {
                    inner: ((self.connection.peek(2)? as u16) << 8)
                        + self.connection.peek(1)? as u16,
                };

                match self
                    .awaiting_replies
                    .get(&sequence_number)
                    .expect("Sequence number must be known")
                {
                    &AwaitingReply::NotReceived(reply_type) => {
                        let reply = self.decode_reply_blocking(reply_type)?;
                        self.awaiting_replies
                            .insert(sequence_number, AwaitingReply::Received(reply));
                    }
                    &AwaitingReply::Discarded(reply_type) => {
                        let _discarded_reply = self.decode_reply_blocking(reply_type)?;
                    }
                    AwaitingReply::Received(_) => {
                        unreachable!("Received response twice for the same sequence number")
                    }
                };
            }
            event_code => {
                let event = self.decode_event_blocking(event_code)?;
                self.event_queue.push_back(event);
            }
        }

        Ok(())
    }

    fn decode_reply_blocking(&mut self, reply_type: ReplyType) -> Result<Reply, Error> {
        match reply_type {
            ReplyType::GetWindowAttributes => {
                let reply = replies::WindowAttributes::from_le_bytes(&mut self.connection)?;
                Ok(Reply::GetWindowAttributes(reply))
            }
            ReplyType::GetGeometry => {
                let reply = replies::Geometry::from_le_bytes(&mut self.connection)?;
                Ok(Reply::GetGeometry(reply))
            }
            ReplyType::GetFontPath => {
                let reply = replies::GetFontPath::from_le_bytes(&mut self.connection)?;
                Ok(Reply::GetFontPath(reply))
            }
            _ => todo!(),
        }
    }

    fn decode_event_blocking(&mut self, event_code: u8) -> Result<Event, Error> {
        let mut raw = [0u8; 32];
        raw[0] = event_code;
        self.connection.read_exact(&mut raw[1..])?;
        Event::from_le_bytes(raw).ok_or(Error::InvalidResponse)
    }

    fn has_pending_events(&mut self) -> Result<bool, Error> {
        Ok(!self.connection.read_buf.is_empty() || self.connection.fill_buf_nonblocking()?)
    }

    /// Drain all events
    pub fn events(&mut self) -> Result<Drain<'_, Event>, Error> {
        while self.has_pending_events()? {
            self.decode_response_blocking()?;
        }

        Ok(self.event_queue.drain(..))
    }

    /// Drain all errors from queue
    pub fn errors(&mut self) -> Drain<'_, XError> {
        self.error_queue.drain(..)
    }
}

// NOTE: Should we use `#[must_use]` here?
// NOTE: Don't derive Clone and Copy, it's not implemented on purpose to emulate linearity.
// i.e. you cannot disacrd reply twice, etc.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SequenceNumber {
    inner: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ListOfStr {
    pub strings: Vec<Vec<u8>>,
}

impl ListOfStr {
    pub(crate) fn encoded_len(&self) -> usize {
        self.strings.iter().map(|s| s.len() + 1).sum()
    }

    pub(crate) fn from_le_bytes(
        strings_count: usize,
        conn: &mut XConnection,
    ) -> Result<Self, Error> {
        let mut strings = Vec::with_capacity(strings_count);
        for _ in 0..strings_count {
            let string_len = conn.read_u8()?;
            let s = conn.read_n_bytes(string_len as usize)?;
            strings.push(s);
        }
        Ok(Self { strings })
    }
}

impl LeBytes for ListOfStr {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        for string in &self.strings {
            let string_len = string.len();
            assert!(string_len <= u8::MAX as usize, "String too long");

            w.write_all(&[string_len as u8])?;
            w.write_all(string)?;
        }

        Ok(())
    }
}

#[test]
fn list_of_str_roundtrip() {
    let raw_data = b"\x0e/file/path/abc\x12/file/path/abcdefg";
    let mut conn = XConnection::dummy(VecDeque::from(raw_data.to_vec()));

    let decoded = ListOfStr::from_le_bytes(2, &mut conn).unwrap();
    let expected = ListOfStr {
        strings: vec![
            vec![
                47, 102, 105, 108, 101, 47, 112, 97, 116, 104, 47, 97, 98, 99,
            ],
            vec![
                47, 102, 105, 108, 101, 47, 112, 97, 116, 104, 47, 97, 98, 99, 100, 101, 102, 103,
            ],
        ],
    };
    assert_eq!(decoded, expected);

    let encoded = {
        let mut buf = Vec::new();
        decoded.to_le_bytes(&mut buf).unwrap();
        buf
    };
    assert_eq!(encoded, raw_data.to_vec());
}
