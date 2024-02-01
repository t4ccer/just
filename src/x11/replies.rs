use crate::x11::{
    connection::XConnection, error::Error, requests::opcodes, ListOfStr, ResourceId, Window,
};

pub trait XReply: Sized {
    fn from_reply(reply: SomeReply) -> Option<Self>;
}

macro_rules! impl_xreply_go {
    ($inner:ty, $wrapper:path) => {
        impl XReply for $inner {
            fn from_reply(reply: SomeReply) -> Option<Self> {
                match reply {
                    $wrapper(r) => Some(r),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! impl_xreply {
    ($t:tt) => {
        impl_xreply_go!($t, SomeReply::$t);
    };
}

#[derive(Debug, Clone)]
pub struct GetWindowAttributes {
    backing_store: u8,
    visual_id: u32,
    class: u16,
    bit_gravity: u8,
    win_gravity: u8,
    backing_planes: u32,
    backing_pixel: u32,
    save_under: bool,
    map_is_installed: bool,
    map_state: u8,
    override_redirect: bool,
    colormap: u32,
    all_even_masks: u32,
    your_even_masks: u32,
    do_not_propagate_mask: u16,
}

impl_xreply!(GetWindowAttributes);

impl GetWindowAttributes {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let backing_store = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let visual_id = conn.read_le_u32()?;
        let class = conn.read_le_u16()?;
        let bit_gravity = conn.read_u8()?;
        let win_gravity = conn.read_u8()?;
        let backing_planes = conn.read_le_u32()?;
        let backing_pixel = conn.read_le_u32()?;
        let save_under = conn.read_bool()?;
        let map_is_installed = conn.read_bool()?;
        let map_state = conn.read_u8()?;
        let override_redirect = conn.read_bool()?;
        let colormap = conn.read_le_u32()?;
        let all_even_masks = conn.read_le_u32()?;
        let your_even_masks = conn.read_le_u32()?;
        let do_not_propagate_mask = conn.read_le_u16()?;
        let _unused = conn.read_le_u16()?;

        Ok(Self {
            backing_store,
            visual_id,
            class,
            bit_gravity,
            win_gravity,
            backing_planes,
            backing_pixel,
            save_under,
            map_is_installed,
            map_state,
            override_redirect,
            colormap,
            all_even_masks,
            your_even_masks,
            do_not_propagate_mask,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetGeometry {
    pub depth: u8,
    pub root: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

impl_xreply!(GetGeometry);

impl GetGeometry {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = Window(ResourceId {
            value: conn.read_le_u32()?,
        });
        let x = conn.read_le_i16()?;
        let y = conn.read_le_i16()?;
        let width = conn.read_le_u16()?;
        let height = conn.read_le_u16()?;
        let border_width = conn.read_le_u16()?;
        conn.drain(10)?;

        Ok(Self {
            depth,
            root,
            x,
            y,
            width,
            height,
            border_width,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GetFontPath {
    pub paths: ListOfStr,
}

impl GetFontPath {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let no_of_strs_in_path = dbg!(conn.read_le_u16())?;
        drop(conn.drain(22)?); // unused
        let paths = ListOfStr::from_le_bytes(no_of_strs_in_path as usize, conn)?;

        Ok(Self { paths })
    }
}

impl_xreply!(GetFontPath);

#[derive(Debug, Clone)]
pub enum SomeReply {
    GetWindowAttributes(GetWindowAttributes),
    GetGeometry(GetGeometry),
    GetFontPath(GetFontPath),
}

#[derive(Debug, Clone)]
pub(crate) enum AwaitingReply {
    /// Response not received yet, but when received will be inserted to the map of responses
    NotReceived(ReplyType),

    /// Response not received yet, but when received will be discared
    Discarded(ReplyType),

    /// Response already received and waiting to be used or discarded
    Received(SomeReply),
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ReplyType {
    GetWindowAttributes = opcodes::GET_WINDOW_ATTRIBUTES,
    GetGeometry = opcodes::GET_GEOMETRY,
    QueryTree = opcodes::QUERY_TREE,
    InternAtom = opcodes::INTERN_ATOM,
    GetAtomName = opcodes::GET_ATOM_NAME,
    GetProperty = opcodes::GET_PROPERTY,
    ListProperties = opcodes::LIST_PROPERTIES,
    GetSelectionOwner = opcodes::GET_SELECTION_OWNER,
    GrabPointer = opcodes::GRAB_POINTER,
    GrabKeyboard = opcodes::GRAB_KEYBOARD,
    QueryPointer = opcodes::QUERY_POINTER,
    GetMotionEvents = opcodes::GET_MOTION_EVENTS,
    TranslateCoordinates = opcodes::TRANSLATE_COORDINATES,
    GetInputFocus = opcodes::GET_INPUT_FOCUS,
    QueryKeymap = opcodes::QUERY_KEYMAP,
    QueryFont = opcodes::QUERY_FONT,
    QueryTextExtents = opcodes::QUERY_TEXT_EXTENTS,
    ListFonts = opcodes::LIST_FONTS,
    ListFontsWithInfo = opcodes::LIST_FONTS_WITH_INFO,
    GetFontPath = opcodes::GET_FONT_PATH,
    GetImage = opcodes::GET_IMAGE,
    ListInstalledColormaps = opcodes::LIST_INSTALLED_COLORMAPS,
    AllocColor = opcodes::ALLOC_COLOR,
    AllocNamedColor = opcodes::ALLOC_NAMED_COLOR,
    AllocColorCells = opcodes::ALLOC_COLOR_CELLS,
    AllocColorPlanes = opcodes::ALLOC_COLOR_PLANES,
    QueryColors = opcodes::QUERY_COLORS,
    LookupColor = opcodes::LOOKUP_COLOR,
    QueryBestSize = opcodes::QUERY_BEST_SIZE,
    QueryExtension = opcodes::QUERY_EXTENSION,
    ListExtensions = opcodes::LIST_EXTENSIONS,
    GetKeyboardMapping = opcodes::GET_KEYBOARD_MAPPING,
    GetKeyboardControl = opcodes::GET_KEYBOARD_CONTROL,
    GetPointerControl = opcodes::GET_POINTER_CONTROL,
    GetScreenSaver = opcodes::GET_SCREEN_SAVER,
    ListHosts = opcodes::LIST_HOSTS,
    SetPointerMapping = opcodes::SET_POINTER_MAPPING,
    GetPointerMapping = opcodes::GET_POINTER_MAPPING,
    SetModifierMapping = opcodes::SET_MODIFIER_MAPPING,
    GetModifierMapping = opcodes::GET_MODIFIER_MAPPING,
}
