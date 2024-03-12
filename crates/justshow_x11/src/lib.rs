#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::new_without_default, // `Default` is a bad idea.
          clippy::unnecessary_cast, // Better be safe in encodings
    )
)]

use requests::{XExtensionRequest, XRequestBase};

use crate::{
    connection::{ConnectionKind, XConnection},
    error::Error,
    events::SomeEvent,
    extensions::randr,
    replies::{AwaitingReply, ReceivedReply, ReplyType, SomeReply, XReply},
    requests::{InitializeConnection, XProtocolVersion, XRequest},
    utils::*,
    xauth::XAuth,
    xerror::SomeError,
};
use std::{
    collections::{vec_deque::Drain, HashMap, VecDeque},
    fmt::Display,
    io::{self, Write},
    marker::PhantomData,
    mem,
};

pub mod atoms;
pub mod connection;
pub mod error;
pub mod events;
pub mod extensions;
pub mod keysym;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ResourceId {
    value: u32,
}

impl ResourceId {
    pub fn value(self) -> u32 {
        self.value
    }
}

impl From<u32> for ResourceId {
    fn from(value: u32) -> Self {
        ResourceId { value }
    }
}

impl From<ResourceId> for u32 {
    fn from(value: ResourceId) -> Self {
        value.value
    }
}

impl_resource_id!(PixmapId);
impl_resource_id!(VisualId);
impl_resource_id!(FontId);
impl_resource_id!(ColormapId);
impl_resource_id!(CursorId);
impl_resource_id!(WindowId);
impl_resource_id!(GContextId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct OrNone<T>(T);

impl<T> OrNone<T>
where
    T: Into<u32> + From<u32> + Copy,
{
    pub fn new(inner: T) -> Self {
        Self(inner)
    }

    pub fn none() -> Self {
        Self(0u32.into())
    }

    pub fn value(self) -> Option<T> {
        if self.0.into() == 0u32 {
            None
        } else {
            Some(self.0)
        }
    }
}

impl<T> From<OrNone<T>> for u32
where
    T: Into<u32>,
{
    fn from(value: OrNone<T>) -> Self {
        value.0.into()
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
        // id_mask has at least 18 continuous ones so we shift next_id to align with these
        let new_part = self.id_mask & (self.next_id << self.id_mask.trailing_zeros());
        self.next_id += 1;

        // Unlikey as they are  at least 2^18 ids to allocate
        assert_ne!(
            new_part, 0,
            "Invalid resource ID allocated, probably run out of resource IDs"
        );

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
            2 => todo!("InitializeConnectionResponseAuthenticate"),
            _ => Err(Error::InvalidResponse(stringify!(
                InitializeConnectionResponse
            ))),
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
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
        let class = VisualClass::try_from(conn.read_u8()?)
            .map_err(|_| Error::InvalidResponse(stringify!(VisualClass)))?;
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Screen {
    pub root: WindowId,
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
        let root = WindowId(ResourceId {
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
        let backing_stores = BackingStore::try_from(conn.read_u8()?)
            .map_err(|_| Error::InvalidResponse("BackingStore"))?;
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
    Window(WindowId),
    Pixmap(PixmapId),
}

impl Drawable {
    fn value(self) -> u32 {
        match self {
            Drawable::Window(window) => window.into(),
            Drawable::Pixmap(pixmap) => pixmap.into(),
        }
    }

    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
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

pub struct XDisplay {
    id_allocator: IdAllocator,
    screens: Vec<Screen>,
    connection: XConnection,
    awaiting_replies: HashMap<SequenceNumber, AwaitingReply>,
    next_sequence_number: SequenceNumber,
    event_queue: VecDeque<SomeEvent>,
    error_queue: VecDeque<SomeError>,
    maximum_request_length: u16,
    pub min_keycode: u8,
    pub max_keycode: u8,
}

impl XDisplay {
    pub fn open() -> Result<Self, Error> {
        let connection = XConnection::open()?;
        Self::with_connection(connection)
    }

    pub fn with_connection(mut connection: XConnection) -> Result<Self, Error> {
        let (authorization_protocol_name, authorization_protocol_data) = match connection.kind() {
            ConnectionKind::UnixStream => {
                let auth = XAuth::from_env()?;
                (auth.name, auth.data)
            }
        };

        let init = InitializeConnection::new(
            XProtocolVersion::V11_0,
            authorization_protocol_name,
            authorization_protocol_data,
        );
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
            next_sequence_number: SequenceNumber { value: 1 }, // InitializeConnection request was 0
            event_queue: VecDeque::new(),
            error_queue: VecDeque::new(),
            maximum_request_length: response.maximum_request_length,
            max_keycode: response.max_keycode,
            min_keycode: response.min_keycode,
        })
    }

    pub fn id_allocator(&mut self) -> &mut IdAllocator {
        &mut self.id_allocator
    }

    pub fn maximum_request_length(&self) -> u16 {
        self.maximum_request_length
    }

    pub fn screens(&self) -> &[Screen] {
        &self.screens
    }

    fn next_sequence_number(&mut self) -> Result<SequenceNumber, Error> {
        let this_sequence_number = self.next_sequence_number.value;
        self.next_sequence_number = SequenceNumber {
            value: self.next_sequence_number.value.wrapping_add(1),
        };

        Ok(SequenceNumber {
            value: this_sequence_number,
        })
    }

    fn wrap_reply<Request: XRequestBase>(
        &mut self,
        sequence_number: SequenceNumber,
    ) -> Result<PendingReply<Request::Reply>, Error> {
        if let Some(reply_type) = Request::reply_type() {
            let sequence_number_is_fresh = self
                .awaiting_replies
                .insert(sequence_number, AwaitingReply::NotReceived(reply_type))
                .is_none();
            debug_assert!(sequence_number_is_fresh);
        }

        Ok(PendingReply {
            sequence_number,
            reply_type: PhantomData,
        })
    }

    /// Send a request to X11 server and return a handler to a pending response. Note that the
    /// request may be buffered until [`Self::flush`] is used.
    pub fn send_request<Request: XRequest>(
        &mut self,
        request: &Request,
    ) -> Result<PendingReply<Request::Reply>, Error> {
        self.connection.send_request(request)?;
        let sequence_number = self.next_sequence_number()?;
        self.wrap_reply::<Request>(sequence_number)
    }

    pub fn send_extension_request<Request: XExtensionRequest>(
        &mut self,
        request: &Request,
        major_opcode: u8,
    ) -> Result<PendingReply<Request::Reply>, Error> {
        self.connection
            .send_extension_request(request, major_opcode)?;
        let sequence_number = self.next_sequence_number()?;
        self.wrap_reply::<Request>(sequence_number)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.connection.flush()?;
        Ok(())
    }

    /// Get reply to previously sent request. Block until reply arrives
    pub fn await_pending_reply<Reply>(
        &mut self,
        mut pending: PendingReply<Reply>,
    ) -> Result<Reply, Error>
    where
        Reply: XReply,
    {
        loop {
            match self.try_get_pending_reply(pending)? {
                Ok(reply) => return Ok(reply),
                Err(returned_pending) => {
                    pending = returned_pending;
                    self.flush()?;
                    self.decode_response_blocking()?;
                }
            }
        }
    }

    /// Try to get reply to previously sent request. If reply didn't arrive yet return pending
    /// reply ID and don't block.
    pub fn try_get_pending_reply<Reply>(
        &mut self,
        pending: PendingReply<Reply>,
    ) -> Result<Result<Reply, PendingReply<Reply>>, Error>
    where
        Reply: XReply,
    {
        let (awaited, entry) = self
            .awaiting_replies
            .remove_entry(&pending.sequence_number)
            .expect("Reponse be tracked in map");

        match entry {
            reply @ AwaitingReply::NotReceived(_) => {
                self.awaiting_replies.insert(awaited, reply);
                Ok(Err(pending))
            }
            AwaitingReply::Discarded(_) => unreachable!("Tried to get discarded reply"),
            AwaitingReply::Received(reply) if reply.done_receiving => {
                Reply::from_reply(reply.reply)
                    .ok_or(Error::UnexpectedReply)
                    .map(Ok)
            }
            reply @ AwaitingReply::Received(_) => {
                self.awaiting_replies.insert(awaited, reply);
                Ok(Err(pending))
            }
        }
    }

    pub fn discard_reply<Reply>(&mut self, to_discard: PendingReply<Reply>) -> Result<(), Error>
    where
        Reply: XReply,
    {
        let entry = self
            .awaiting_replies
            .get(&to_discard.sequence_number)
            .expect("Sequence number must be known");

        match entry {
            &AwaitingReply::NotReceived(ty) => {
                self.awaiting_replies
                    .insert(to_discard.sequence_number, AwaitingReply::Discarded(ty));
            }
            AwaitingReply::Discarded(_) => unreachable!("Discarded sequence number twice"),
            AwaitingReply::Received(received) => {
                if received.done_receiving {
                    self.awaiting_replies.remove(&to_discard.sequence_number);
                } else {
                    self.awaiting_replies.insert(
                        to_discard.sequence_number,
                        AwaitingReply::Discarded(received.reply_type),
                    );
                }
            }
        };

        Ok(())
    }

    fn decode_response_blocking(&mut self) -> Result<(), Error> {
        let code: u8 = self.connection.read_u8()?;
        match code {
            0 => {
                let error_code: u8 = self.connection.read_u8()?;
                let error = SomeError::from_le_bytes(&mut self.connection, error_code)?;
                self.error_queue.push_back(error);
            }
            1 => {
                self.handle_reply_blocking()?;
            }
            event_code => {
                let event = self.decode_event_blocking(event_code)?;
                self.event_queue.push_back(event);
            }
        }

        Ok(())
    }

    fn handle_reply_blocking(&mut self) -> Result<(), Error> {
        // TODO: Try to avoid using peek
        let sequence_number: SequenceNumber = SequenceNumber {
            value: ((self.connection.peek(2)? as u16) << 8) + self.connection.peek(1)? as u16,
        };

        let (_, awaiting_reply) = self
            .awaiting_replies
            .remove_entry(&sequence_number)
            .expect("Sequence number must be known");

        let reply_type = awaiting_reply.reply_type();
        let reply = self.decode_reply_blocking(reply_type)?;

        match awaiting_reply {
            AwaitingReply::NotReceived(_) => {
                let received = match reply {
                    reply @ SomeReply::ListFontsWithInfoPartial(_) => {
                        let mut received = ReceivedReply {
                            reply: SomeReply::ListFontsWithInfo(replies::ListFontsWithInfo {
                                replies: vec![],
                            }),
                            reply_type,
                            done_receiving: false,
                        };
                        debug_assert!(
                            received.append_reply(reply),
                            "Could not merge with empty reply"
                        );
                        received
                    }
                    reply => ReceivedReply {
                        reply,
                        reply_type,
                        done_receiving: true,
                    },
                };

                self.awaiting_replies
                    .insert(sequence_number, AwaitingReply::Received(received));
            }
            discarded @ AwaitingReply::Discarded(_) => {
                if let SomeReply::ListFontsWithInfoPartial(
                    replies::ListFontsWithInfoPartial::ListFontsWithInfoPiece(_),
                ) = reply
                {
                    {
                        // We cannot remove it from tracking map yet as this is a partial response
                        // and more will come with the same sequence number, so it must be saved
                        // to lookup the response type.
                        self.awaiting_replies.insert(sequence_number, discarded);
                    }
                }
            }
            AwaitingReply::Received(mut old_reply) => {
                if old_reply.append_reply(reply) {
                    self.awaiting_replies
                        .insert(sequence_number, AwaitingReply::Received(old_reply));
                } else {
                    todo!("Unmatched response types with pending data not handled yet")
                }
            }
        };

        Ok(())
    }

    fn decode_reply_blocking(&mut self, reply_type: ReplyType) -> Result<SomeReply, Error> {
        macro_rules! handle_reply {
            ($t:tt) => {{
                let reply = replies::$t::from_le_bytes(&mut self.connection)?;
                Ok(SomeReply::$t(reply))
            }};
        }

        match reply_type {
            ReplyType::GetWindowAttributes => handle_reply!(GetWindowAttributes),
            ReplyType::GetGeometry => handle_reply!(GetGeometry),
            ReplyType::QueryTree => handle_reply!(QueryTree),
            ReplyType::InternAtom => handle_reply!(InternAtom),
            ReplyType::GetAtomName => handle_reply!(GetAtomName),
            ReplyType::GetProperty => handle_reply!(GetProperty),
            ReplyType::ListProperties => handle_reply!(ListProperties),
            ReplyType::GetSelectionOwner => handle_reply!(GetSelectionOwner),
            ReplyType::GrabPointer => handle_reply!(GrabPointer),
            ReplyType::GrabKeyboard => handle_reply!(GrabKeyboard),
            ReplyType::QueryPointer => handle_reply!(QueryPointer),
            ReplyType::GetMotionEvents => handle_reply!(GetMotionEvents),
            ReplyType::TranslateCoordinates => handle_reply!(TranslateCoordinates),
            ReplyType::GetInputFocus => handle_reply!(GetInputFocus),
            ReplyType::QueryKeymap => handle_reply!(QueryKeymap),
            ReplyType::QueryFont => handle_reply!(QueryFont),
            ReplyType::QueryTextExtents => handle_reply!(QueryTextExtents),
            ReplyType::ListFonts => handle_reply!(ListFonts),
            ReplyType::ListFontsWithInfo => {
                // ListFontsWithInfo request may result in multiple replies so we need to handle it
                // specially here. We cannot use `handle_reply!` here as reply type is
                // `ListFontsWithInfo` because it is what client wants to receive at the end.
                let reply = replies::ListFontsWithInfoPartial::from_le_bytes(&mut self.connection)?;
                Ok(SomeReply::ListFontsWithInfoPartial(reply))
            }
            ReplyType::GetFontPath => handle_reply!(GetFontPath),
            ReplyType::GetImage => handle_reply!(GetImage),
            ReplyType::ListInstalledColormaps => handle_reply!(ListInstalledColormaps),
            ReplyType::AllocColor => handle_reply!(AllocColor),
            ReplyType::AllocNamedColor => handle_reply!(AllocNamedColor),
            ReplyType::AllocColorCells => handle_reply!(AllocColorCells),
            ReplyType::AllocColorPlanes => handle_reply!(AllocColorPlanes),
            ReplyType::QueryColors => handle_reply!(QueryColors),
            ReplyType::LookupColor => handle_reply!(LookupColor),
            ReplyType::QueryBestSize => handle_reply!(QueryBestSize),
            ReplyType::QueryExtension => handle_reply!(QueryExtension),
            ReplyType::ListExtensions => handle_reply!(ListExtensions),
            ReplyType::GetKeyboardMapping => handle_reply!(GetKeyboardMapping),
            ReplyType::GetKeyboardControl => handle_reply!(GetKeyboardControl),
            ReplyType::GetPointerControl => handle_reply!(GetPointerControl),
            ReplyType::GetScreenSaver => handle_reply!(GetScreenSaver),
            ReplyType::ListHosts => handle_reply!(ListHosts),
            ReplyType::SetPointerMapping => handle_reply!(SetPointerMapping),
            ReplyType::GetPointerMapping => handle_reply!(GetPointerMapping),
            ReplyType::SetModifierMapping => handle_reply!(SetModifierMapping),
            ReplyType::GetModifierMapping => handle_reply!(GetModifierMapping),
            ReplyType::ExtensionRandr(randr_reply) => {
                macro_rules! handle_randr_reply {
                    ($t:tt) => {{
                        let reply = randr::replies::$t::from_le_bytes(&mut self.connection)?;
                        Ok(SomeReply::ExtensionRandr(randr::replies::SomeReply::$t(
                            reply,
                        )))
                    }};
                }

                match randr_reply {
                    randr::replies::ReplyType::GetMonitors => handle_randr_reply!(GetMonitors),
                    randr::replies::ReplyType::GetCrtcInfo => handle_randr_reply!(GetCrtcInfo),
                }
            }
        }
    }

    fn decode_event_blocking(&mut self, event_code: u8) -> Result<SomeEvent, Error> {
        let mut raw = [0u8; 32];
        raw[0] = event_code;
        self.connection.read_exact(&mut raw[1..])?;
        SomeEvent::from_le_bytes(raw).ok_or(Error::InvalidResponse(stringify!(SomeEvent)))
    }

    fn has_pending_events(&mut self) -> Result<bool, Error> {
        Ok(self.connection.has_unconsumed_data() || self.connection.fill_buf_nonblocking()?)
    }

    pub fn next_event(&mut self) -> Result<Option<SomeEvent>, Error> {
        while self.has_pending_events()? {
            self.decode_response_blocking()?;
        }

        Ok(self.event_queue.pop_front())
    }

    /// Drain all events
    pub fn events(&mut self) -> Result<Drain<'_, SomeEvent>, Error> {
        while self.has_pending_events()? {
            self.decode_response_blocking()?;
        }

        Ok(self.event_queue.drain(..))
    }

    /// Drain all errors from queue
    pub fn errors(&mut self) -> Drain<'_, SomeError> {
        self.error_queue.drain(..)
    }
}

// i.e. you cannot disacrd reply twice, etc.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SequenceNumber {
    value: u16,
}

// NOTE: Don't derive Clone and Copy, it's not implemented on purpose to emulate linearity.
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct PendingReply<Reply> {
    sequence_number: SequenceNumber,
    reply_type: PhantomData<Reply>,
}

// Rust's constraints solver isn't really smart and enforced Debug on R for no reason
impl<Reply> std::fmt::Debug for PendingReply<Reply> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingReply")
            .field("sequence_number", &self.sequence_number)
            .field("reply_type", &self.reply_type)
            .finish()
    }
}

impl<Reply> PendingReply<Reply> {
    pub fn sequence_number(&self) -> SequenceNumber {
        self.sequence_number
    }
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
