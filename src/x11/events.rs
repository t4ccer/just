use crate::x11::{Atom, Colormap, OrNone, Window};
use std::mem;

use super::ResourceId;

fn invalid_bool(value: u8) -> bool {
    value != 0 && value != 1
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct KeyPressRelease {
    _event_code: u8,
    pub detail: u8,
    pub time: u32,
    pub root: u32,
    pub event: u32,
    pub child: u32,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16,
    pub same_screen: bool,
    _pad: [u8; 1],
}

impl KeyPressRelease {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x1e]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum MotionNotifyDetail {
    Normal = 0,
    Hint = 1,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MotionNotify {
    _event_code: u8,
    pub detail: MotionNotifyDetail,
    _sequence_number: u16,
    pub time: u32,
    pub root: Window,
    pub event: Window,
    pub child: OrNone<Window>,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16, // TODO: SETofKEYBUTMASK
    pub same_screen: bool,
    _pad: [u8; 1],
}

impl MotionNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x01] > 1 {
            return None;
        }

        if invalid_bool(raw[0x1e]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum EnterLeaveNotifyMode {
    Normal = 0,
    Grab = 1,
    Ungrab = 2,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum EnterLeaveNotifyDetail {
    Ancestor = 0,
    Virtual = 1,
    Inferior = 2,
    Nonlinear = 3,
    NonlinearVirtual = 4,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct EnterLeaveNotify {
    _event_code: u8,
    pub detail: EnterLeaveNotifyDetail,
    _sequence_number: u16,
    pub time: u32,
    pub root: Window,
    pub event: Window,
    pub child: OrNone<Window>,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: u16, // TODO: SETofKEYBUTMASK
    pub mode: EnterLeaveNotifyMode,
    pub same_screen_focus: u8, // TODO: Type
}

impl EnterLeaveNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x1e] > 2 {
            return None;
        }

        if raw[0x01] > 4 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FocusInOutDetail {
    Ancestor = 0,
    Virtual = 1,
    Inferior = 2,
    Nonlinear = 3,
    NonlinearVirtual = 4,
    Pointer = 5,
    PointerRoot = 6,
    None = 7,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum FocusInOutMode {
    Normal = 0,
    Grab = 1,
    Ungrab = 2,
    WhileGrabbed = 3,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct FocusInOut {
    _event_code: u8,
    pub detail: FocusInOutDetail,
    _sequence_number: u16,
    pub event: Window,
    pub mode: FocusInOutMode,
    _pad: [u8; 23],
}

impl FocusInOut {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x01] > 7 {
            return None;
        }

        if raw[0x08] > 3 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct KeymapNotify {
    _event_code: u8,
    pub keys: [u8; 31],
}

impl KeymapNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Expose {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub window: Window,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub count: u16,
    _pad: [u8; 14],
}

impl Expose {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct GraphicsExposure {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub drawable: ResourceId,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub minor_opcode: u16,
    pub count: u16,
    pub major_opcode: u8,
    _pad: [u8; 11],
}

impl GraphicsExposure {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct NoExposure {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub drawable: ResourceId,
    pub minor_opcode: u16,
    pub major_opcode: u8,
    _pad: [u8; 21],
}

impl NoExposure {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum VisibilityNotifyState {
    Unobscured = 0,
    PartiallyObscured = 1,
    FullyObscured = 2,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct VisibilityNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub window: Window,
    pub state: VisibilityNotifyState,
    _pad: [u8; 23],
}

impl VisibilityNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x08] > 2 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CreateNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub parent: Window,
    pub window: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: bool,
    _pad: [u8; 9],
}

impl CreateNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x16]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct DestroyNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    _pad: [u8; 20],
}

impl DestroyNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct UnmapNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    pub from_configure: bool,
    _pad: [u8; 19],
}

impl UnmapNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x0c]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MapNotify {
    _event_code: u8,
    _unused: u8,
    pub event: Window,
    pub window: Window,
    pub override_redirect: bool,
    _pad: [u8; 19],
}

impl MapNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x0c]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MapRequest {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub parent: Window,
    pub window: Window,
    _pad: [u8; 20],
}

impl MapRequest {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ReparentNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    event: Window,
    window: Window,
    parent: Window,
    x: i16,
    y: i16,
    override_redirect: bool,
    _pad: [u8; 11],
}

impl ReparentNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x14]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ConfigureNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    pub above_sibling: OrNone<Window>,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: bool,
    _pad: [u8; 5],
}

impl ConfigureNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x1a]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ConfigureRequestStackMode {
    Above = 0,
    Below = 1,
    TopIf = 2,
    BottomIf = 3,
    Opposite = 4,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ConfigureRequest {
    _event_code: u8,
    pub stack_mode: ConfigureRequestStackMode,
    _sequence_number: u16,
    pub parent: Window,
    pub window: Window,
    pub sibling: OrNone<Window>,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub value_mask: u16,
    _pad: [u8; 4],
}

impl ConfigureRequest {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x01] > 4 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct GravityNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    x: i16,
    y: i16,
    _pad: [u8; 16],
}

impl GravityNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ResizeRequest {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub window: Window,
    pub width: u16,
    pub height: u16,
    _pad: [u8; 20],
}

impl ResizeRequest {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CirculateNotifyPlace {
    Top = 0,
    Bottom = 1,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CirculateNotify {
    _event_code: u8,
    _unused1: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    _unused2: Window, // correct
    pub place: CirculateNotifyPlace,
    _pad: [u8; 15],
}

impl CirculateNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x0d] > 1 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct CirculateRequest {
    _event_code: u8,
    _unused1: u8,
    _sequence_number: u16,
    pub event: Window,
    pub window: Window,
    _unused2: u32,
    pub place: CirculateNotifyPlace,
    _pad: [u8; 15],
}

impl CirculateRequest {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x10] > 1 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum PropertyNotifyState {
    NewValue,
    Deleted,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct PropertyNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub window: Window,
    pub atom: Atom,
    pub time: u32,
    pub state: PropertyNotifyState,
    _pad: [u8; 15],
}

impl PropertyNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x10] > 1 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SelectionClear {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub time: u32,
    pub owner: Window,
    pub selection: Atom,
    _pad: [u8; 16],
}

impl SelectionClear {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SelectionRequest {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub time: u32, // 0 for current
    pub owner: Window,
    pub requestor: Window,
    pub selection: Atom,
    pub target: Atom,
    pub property: OrNone<Atom>,
    _pad: [u8; 4],
}

impl SelectionRequest {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct SelectionNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub time: u32, // 0 for current
    pub requestor: Window,
    pub selection: Atom,
    pub target: Atom,
    pub property: OrNone<Atom>,
    _pad: [u8; 8],
}

impl SelectionNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ColormapNotifyState {
    Uninstalled = 0,
    Installed = 1,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ColormapNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub window: Window,
    pub colormap: OrNone<Colormap>,
    pub new: bool,
    pub state: ColormapNotifyState,
    _pad: [u8; 18],
}

impl ColormapNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if invalid_bool(raw[0x0c]) {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ClientMessage {
    _event_code: u8,
    pub format: u8,
    _sequence_number: u16,
    pub window: Window,
    pub type_message: Atom, // 'type' is a keyword
    pub data: [u8; 20],
}

impl ClientMessage {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum MappingNotifyRequest {
    Modifier = 0,
    Keyboard = 1,
    Pointer = 2,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct MappingNotify {
    _event_code: u8,
    _unused: u8,
    _sequence_number: u16,
    pub request: MappingNotifyRequest,
    pub first_keycode: u8,
    count: u8,
    _pad: [u8; 25],
}

impl MappingNotify {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        if raw[0x04] > 2 {
            return None;
        }

        Some(unsafe { mem::transmute(raw) })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Event {
    KeyPress(KeyPressRelease) = 2,
    KeyRelease(KeyPressRelease) = 3,
    ButtonPress(KeyPressRelease) = 4,
    ButtonRelease(KeyPressRelease) = 5,
    MotionNotify(MotionNotify) = 6,
    EnterNotify(EnterLeaveNotify) = 7,
    LeaveNotify(EnterLeaveNotify) = 8,
    FocusIn(FocusInOut) = 9,
    FocusOut(FocusInOut) = 10,
    KeymapNotify(KeymapNotify) = 11,
    Expose(Expose) = 12,
    GraphicsExposure(GraphicsExposure) = 13,
    NoExposure(NoExposure) = 14,
    VisibilityNotify(VisibilityNotify) = 15,
    CreateNotify(CreateNotify) = 16,
    DestroyNotify(DestroyNotify) = 17,
    UnmapNotify(UnmapNotify) = 18,
    MapNotify(MapNotify) = 19,
    MapRequest(MapRequest) = 20,
    ReparentNotify(ReparentNotify) = 21,
    ConfigureNotify(ConfigureNotify) = 22,
    ConfigureRequest(ConfigureRequest) = 23,
    GravityNotify(GravityNotify) = 24,
    ResizeRequest(ResizeRequest) = 25,
    CirculateNotify(CirculateNotify) = 26,
    CirculateRequest(CirculateRequest) = 27,
    PropertyNotify(PropertyNotify) = 28,
    SelectionClear(SelectionClear) = 29,
    SelectionRequest(SelectionRequest) = 30,
    SelectionNotify(SelectionNotify) = 31,
    ColormapNotify(ColormapNotify) = 32,
    ClientMessage(ClientMessage) = 33,
    MappingNotify(MappingNotify) = 34,
}

impl Event {
    pub(crate) fn from_le_bytes(raw: [u8; 32]) -> Option<Self> {
        // TODO: Detect high upper bit set for extension events
        let event_code = raw[0];

        match event_code {
            2 => Some(Self::KeyPress(KeyPressRelease::from_le_bytes(raw)?)),
            3 => Some(Self::KeyRelease(KeyPressRelease::from_le_bytes(raw)?)),
            4 => Some(Self::ButtonPress(KeyPressRelease::from_le_bytes(raw)?)),
            5 => Some(Self::ButtonRelease(KeyPressRelease::from_le_bytes(raw)?)),
            6 => Some(Self::MotionNotify(MotionNotify::from_le_bytes(raw)?)),
            7 => Some(Self::EnterNotify(EnterLeaveNotify::from_le_bytes(raw)?)),
            8 => Some(Self::LeaveNotify(EnterLeaveNotify::from_le_bytes(raw)?)),
            9 => Some(Self::FocusIn(FocusInOut::from_le_bytes(raw)?)),
            10 => Some(Self::FocusOut(FocusInOut::from_le_bytes(raw)?)),
            11 => Some(Self::KeymapNotify(KeymapNotify::from_le_bytes(raw)?)),
            12 => Some(Self::Expose(Expose::from_le_bytes(raw)?)),
            13 => Some(Self::GraphicsExposure(GraphicsExposure::from_le_bytes(
                raw,
            )?)),
            14 => Some(Self::NoExposure(NoExposure::from_le_bytes(raw)?)),
            15 => Some(Self::VisibilityNotify(VisibilityNotify::from_le_bytes(
                raw,
            )?)),
            16 => Some(Self::CreateNotify(CreateNotify::from_le_bytes(raw)?)),
            17 => Some(Self::DestroyNotify(DestroyNotify::from_le_bytes(raw)?)),
            18 => Some(Self::UnmapNotify(UnmapNotify::from_le_bytes(raw)?)),
            19 => Some(Self::MapNotify(MapNotify::from_le_bytes(raw)?)),
            20 => Some(Self::MapRequest(MapRequest::from_le_bytes(raw)?)),
            21 => Some(Self::ReparentNotify(ReparentNotify::from_le_bytes(raw)?)),
            22 => Some(Event::ConfigureNotify(ConfigureNotify::from_le_bytes(raw)?)),
            23 => Some(Event::ConfigureRequest(ConfigureRequest::from_le_bytes(
                raw,
            )?)),
            24 => Some(Event::GravityNotify(GravityNotify::from_le_bytes(raw)?)),
            25 => Some(Event::ResizeRequest(ResizeRequest::from_le_bytes(raw)?)),
            26 => Some(Event::CirculateNotify(CirculateNotify::from_le_bytes(raw)?)),
            27 => Some(Event::CirculateRequest(CirculateRequest::from_le_bytes(
                raw,
            )?)),
            28 => Some(Event::PropertyNotify(PropertyNotify::from_le_bytes(raw)?)),
            29 => Some(Event::SelectionClear(SelectionClear::from_le_bytes(raw)?)),
            30 => Some(Event::SelectionRequest(SelectionRequest::from_le_bytes(
                raw,
            )?)),
            31 => Some(Event::SelectionNotify(SelectionNotify::from_le_bytes(raw)?)),
            32 => Some(Event::ColormapNotify(ColormapNotify::from_le_bytes(raw)?)),
            33 => Some(Event::ClientMessage(ClientMessage::from_le_bytes(raw)?)),
            34 => Some(Event::MappingNotify(MappingNotify::from_le_bytes(raw)?)),
            _ => None,
        }
    }
}
