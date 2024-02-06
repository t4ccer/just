use crate::x11::{
    connection::XConnection,
    error::Error,
    requests::Timestamp,
    requests::{opcodes, KeyCode},
    utils::pad,
    AtomId, ColormapId, ListOfStr, OrNone, ResourceId, VisualId, WindowId,
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

/*
     1                         family
           0         Internet
           1         DECnet
           2         Chaos
           5         ServerInterpreted
           6         InternetV6
     1                         unused
     2      n                  length of address
     n      LISTofBYTE         address
     p                         unused, p=pad(n)

*/
#[derive(Debug, Clone)]
pub enum HostFamily {
    Internet = 0,
    DECnet = 1,
    Chaos = 2,
    ServerImplemented = 5,
    InternetV6 = 6,
}

impl HostFamily {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        match conn.read_u8()? {
            0 => Ok(Self::Internet),
            1 => Ok(Self::DECnet),
            2 => Ok(Self::Chaos),
            5 => Ok(Self::ServerImplemented),
            6 => Ok(Self::InternetV6),
            _ => Err(Error::InvalidResponse),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Host {
    pub family: HostFamily,
    pub address: Vec<u8>,
}

impl Host {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let family = HostFamily::from_le_bytes(conn)?;
        let _unused = conn.read_u8()?;
        let address_length = conn.read_le_u16()?;
        let address = conn.read_n_bytes(address_length as usize)?;
        drop(conn.drain(pad(address_length as usize))?);
        Ok(Self { family, address })
    }
}

/*
GetWindowAttributes
▶
     1     1                               Reply
     1                                     backing-store
          0     NotUseful
          1     WhenMapped
          2     Always
     2     CARD16                          sequence number
     4     3                               reply length
     4     VISUALID                        visual
     2                                     class
          1     InputOutput
          2     InputOnly
     1     BITGRAVITY                      bit-gravity
     1     WINGRAVITY                      win-gravity
     4     CARD32                          backing-planes
     4     CARD32                          backing-pixel
     1     BOOL                            save-under
     1     BOOL                            map-is-installed
     1                                     map-state
          0     Unmapped
          1     Unviewable
          2     Viewable
     1     BOOL                            override-redirect
     4     COLORMAP                        colormap
          0     None
     4     SETofEVENT                      all-event-masks
     4     SETofEVENT                      your-event-mask
     2     SETofDEVICEEVENT                do-not-propagate-mask
     2                                     unused
*/

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

impl_xreply!(GetWindowAttributes);

/*
GetGeometry
▶
     1     1                               Reply
     1     CARD8                           depth
     2     CARD16                          sequence number
     4     0                               reply length
     4     WINDOW                          root
     2     INT16                           x
     2     INT16                           y
     2     CARD16                          width
     2     CARD16                          height
     2     CARD16                          border-width
     10                                    unused
*/
#[derive(Debug, Clone)]
pub struct GetGeometry {
    pub depth: u8,
    pub root: WindowId,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

impl GetGeometry {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = WindowId(ResourceId {
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

impl_xreply!(GetGeometry);

/*
QueryTree
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     n                               reply length
     4     WINDOW                          root
     4     WINDOW                          parent
          0     None
     2     n                               number of WINDOWs in children
     14                                    unused
     4n     LISTofWINDOW                   children
*/

#[derive(Debug, Clone)]
pub struct QueryTree {
    pub root: WindowId,
    pub parent: OrNone<WindowId>,
    pub children: Vec<WindowId>,
}

impl QueryTree {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = WindowId(ResourceId {
            value: conn.read_le_u32()?,
        });
        let parent = OrNone(WindowId(ResourceId {
            value: conn.read_le_u32()?,
        }));
        let children_count = conn.read_le_u16()?;
        drop(conn.drain(14)?);
        let mut children = Vec::with_capacity(children_count as usize);
        for _ in 0..children_count {
            let child = WindowId(ResourceId {
                value: conn.read_le_u32()?,
            });
            children.push(child);
        }

        Ok(Self {
            root,
            parent,
            children,
        })
    }
}

impl_xreply!(QueryTree);

/*
InternAtom
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     4     ATOM                            atom
           0     None
     20                                    unused
*/

#[derive(Debug, Clone)]
pub struct InternAtom {
    pub atom: AtomId,
}

impl InternAtom {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let atom = AtomId::from(conn.read_le_u32()?);
        drop(conn.drain(20)?);

        Ok(Self { atom })
    }
}

impl_xreply!(InternAtom);

/*
GetAtomName
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     2     n                               length of name
     22                                    unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct GetAtomName {
    pub name: Vec<u8>,
}

impl GetAtomName {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let name_length = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);
        let name = conn.read_n_bytes(name_length)?;
        drop(conn.drain(pad(name_length))?);

        Ok(Self { name })
    }
}

impl_xreply!(GetAtomName);

/*
GetProperty
▶
     1     1                               Reply
     1     CARD8                           format
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     4     ATOM                            type
          0     None
     4     CARD32                          bytes-after
     4     CARD32                          length of value in format units
                    (= 0 for format = 0)
                    (= n for format = 8)
                    (= n/2 for format = 16)
                    (= n/4 for format = 32)
     12                                    unused
     n     LISTofBYTE                      value
                    (n is zero for format = 0)
                    (n is a multiple of 2 for format = 16)
                    (n is a multiple of 4 for format = 32)
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct GetProperty {
    pub format: u8,
    pub type_: AtomId,
    pub bytes_after: u32,
    pub length_of_value: u32,
    pub value: Vec<u8>,
}

impl GetProperty {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let format = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let type_ = AtomId::from(conn.read_le_u32()?);
        let bytes_after = conn.read_le_u32()?;
        let length_of_value = conn.read_le_u32()?;
        drop(conn.drain(12)?);
        let value_length = length_of_value as usize;
        let value = conn.read_n_bytes(value_length)?;
        drop(conn.drain(pad(value_length))?);

        Ok(Self {
            format,
            type_,
            bytes_after,
            length_of_value,
            value,
        })
    }
}

impl_xreply!(GetProperty);

/*
ListProperties
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     n                               reply length
     2     n                               number of ATOMs in atoms
     22                                    unused
     4n     LISTofATOM                     atoms
*/

#[derive(Debug, Clone)]
pub struct ListProperties {
    pub atoms: Vec<AtomId>,
}

impl ListProperties {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let atom_count = conn.read_le_u16()?;
        drop(conn.drain(22)?);
        let mut atoms = Vec::with_capacity(atom_count as usize);
        for _ in 0..atom_count {
            let atom = AtomId::from(conn.read_le_u32()?);
            atoms.push(atom);
        }

        Ok(Self { atoms })
    }
}

impl_xreply!(ListProperties);

/*
GetSelectionOwner
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     4     WINDOW                          owner
          0     None
     20                                    unused
*/

#[derive(Debug, Clone)]
pub struct GetSelectionOwner {
    pub owner: WindowId,
}

impl GetSelectionOwner {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let owner = WindowId(ResourceId {
            value: conn.read_le_u32()?,
        });
        drop(conn.drain(20)?);

        Ok(Self { owner })
    }
}

impl_xreply!(GetSelectionOwner);

/*
GrabPointer
▶
     1     1                               Reply
     1                                     status
          0     Success
          1     AlreadyGrabbed
          2     InvalidTime
          3     NotViewable
          4     Frozen
     2     CARD16                          sequence number
     4     0                               reply length
     24                                    unused
*/

#[derive(Debug, Clone)]
pub enum GrabPointerStatus {
    Success,
    AlreadyGrabbed,
    InvalidTime,
    NotViewable,
    Frozen,
}

#[derive(Debug, Clone)]
pub struct GrabPointer {
    pub status: GrabPointerStatus,
}

impl GrabPointer {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status_code = conn.read_u8()?;
        let status = match status_code {
            0 => GrabPointerStatus::Success,
            1 => GrabPointerStatus::AlreadyGrabbed,
            2 => GrabPointerStatus::InvalidTime,
            3 => GrabPointerStatus::NotViewable,
            4 => GrabPointerStatus::Frozen,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        drop(conn.drain(4 + 24)?);

        Ok(Self { status })
    }
}

impl_xreply!(GrabPointer);

/*
GrabKeyboard
▶
     1     1                               Reply
     1                                     status
          0     Success
          1     AlreadyGrabbed
          2     InvalidTime
          3     NotViewable
          4     Frozen
     2     CARD16                          sequence number
     4     0                               reply length
     24                                    unused
*/

#[derive(Debug, Clone)]
pub enum GrabKeyboardStatus {
    Success,
    AlreadyGrabbed,
    InvalidTime,
    NotViewable,
    Frozen,
}

#[derive(Debug, Clone)]
pub struct GrabKeyboard {
    pub status: GrabKeyboardStatus,
}

impl GrabKeyboard {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status_code = conn.read_u8()?;
        let status = match status_code {
            0 => GrabKeyboardStatus::Success,
            1 => GrabKeyboardStatus::AlreadyGrabbed,
            2 => GrabKeyboardStatus::InvalidTime,
            3 => GrabKeyboardStatus::NotViewable,
            4 => GrabKeyboardStatus::Frozen,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        drop(conn.drain(4 + 24)?);

        Ok(Self { status })
    }
}

impl_xreply!(GrabKeyboard);

/*
QueryPointer
▶
     1     1                               Reply
     1     BOOL                            same-screen
     2     CARD16                          sequence number
     4     0                               reply length
     4     WINDOW                          root
     4     WINDOW                          child
          0     None
     2     INT16                           root-x
     2     INT16                           root-y
     2     INT16                           win-x
     2     INT16                           win-y
     2     SETofKEYBUTMASK                 mask
     6                                     unused
*/

#[derive(Debug, Clone)]
pub struct QueryPointer {
    pub same_screen: bool,
    pub root: WindowId,
    pub child: OrNone<WindowId>,
    pub root_x: i16,
    pub root_y: i16,
    pub win_x: i16,
    pub win_y: i16,
    pub mask: u16,
}

impl QueryPointer {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let same_screen = conn.read_bool()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = WindowId(ResourceId {
            value: conn.read_le_u32()?,
        });
        let child = OrNone(WindowId(ResourceId {
            value: conn.read_le_u32()?,
        }));
        let root_x = conn.read_le_i16()?;
        let root_y = conn.read_le_i16()?;
        let win_x = conn.read_le_i16()?;
        let win_y = conn.read_le_i16()?;
        let mask = conn.read_le_u16()?;
        drop(conn.drain(6)?);

        Ok(Self {
            same_screen,
            root,
            child,
            root_x,
            root_y,
            win_x,
            win_y,
            mask,
        })
    }
}

impl_xreply!(QueryPointer);

/*
GetMotionEvents
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     2n                              reply length
     4     n                               number of TIMECOORDs in events
     20                                    unused
     8n     LISTofTIMECOORD                events

  TIMECOORD
     4     TIMESTAMP                       time
     2     INT16                           x
     2     INT16                           y
*/

#[derive(Debug, Clone)]
pub struct TimeCoord {
    pub time: Timestamp,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone)]
pub struct GetMotionEvents {
    pub events: Vec<TimeCoord>,
}

impl GetMotionEvents {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let event_count = conn.read_le_u32()? as usize;
        drop(conn.drain(20)?);

        let mut events = Vec::with_capacity(event_count);
        for _ in 0..event_count {
            let time = Timestamp(conn.read_le_u32()?);
            let x = conn.read_le_i16()?;
            let y = conn.read_le_i16()?;
            events.push(TimeCoord { time, x, y });
        }

        Ok(Self { events })
    }
}

impl_xreply!(GetMotionEvents);

/*
TranslateCoordinates
▶
     1     1                               Reply
     1     BOOL                            same-screen
     2     CARD16                          sequence number
     4     0                               reply length
     4     WINDOW                          child
          0     None
     2     INT16                           dst-x
     2     INT16                           dst-y
     16                                    unused
*/

#[derive(Debug, Clone)]
pub struct TranslateCoordinates {
    pub same_screen: bool,
    pub child: WindowId,
    pub dst_x: i16,
    pub dst_y: i16,
}

impl TranslateCoordinates {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let same_screen = conn.read_bool()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let child = WindowId(ResourceId {
            value: conn.read_le_u32()?,
        });
        let dst_x = conn.read_le_i16()?;
        let dst_y = conn.read_le_i16()?;
        drop(conn.drain(16)?);

        Ok(Self {
            same_screen,
            child,
            dst_x,
            dst_y,
        })
    }
}

impl_xreply!(TranslateCoordinates);

/*
GetInputFocus
▶
     1     1                               Reply
     1                                     revert-to
          0     None
          1     PointerRoot
          2     Parent
     2     CARD16                          sequence number
     4     0                               reply length
     4     WINDOW                          focus
          0     None
          1     PointerRoot
     20                                    unused
*/

#[derive(Debug, Clone)]
pub enum Focus {
    None,
    PointerRoot,
    Window(WindowId),
}

#[derive(Debug, Clone)]
pub enum RevertTo {
    None,
    PointerRoot,
    Parent,
}

#[derive(Debug, Clone)]
pub struct GetInputFocus {
    pub revert_to: RevertTo,
    pub focus: Focus,
}

impl GetInputFocus {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let revert_to_code = conn.read_u8()?;
        let revert_to = match revert_to_code {
            0 => RevertTo::None,
            1 => RevertTo::PointerRoot,
            2 => RevertTo::Parent,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let focus_value = conn.read_le_u32()?;
        let focus = match focus_value {
            0 => Focus::None,
            1 => Focus::PointerRoot,
            wid => Focus::Window(WindowId::from(wid)),
        };
        drop(conn.drain(20)?);

        Ok(Self { revert_to, focus })
    }
}

impl_xreply!(GetInputFocus);

/*
QueryKeymap
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     2                               reply length
     32     LISTofCARD8                    keys
*/

#[derive(Debug, Clone)]
pub struct QueryKeymap {
    pub keys: [u8; 32],
}

impl QueryKeymap {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let mut keys = [0; 32];
        conn.read_exact(&mut keys)?;

        Ok(Self { keys })
    }
}

impl_xreply!(QueryKeymap);

/*
QueryFont
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     7+2n+3m                         reply length
     12     CHARINFO                       min-bounds
     4                                     unused
     12     CHARINFO                       max-bounds
     4                                     unused
     2     CARD16                          min-char-or-byte2
     2     CARD16                          max-char-or-byte2
     2     CARD16                          default-char
     2     n                               number of FONTPROPs in properties
     1                                     draw-direction
          0     LeftToRight
          1     RightToLeft
     1     CARD8                           min-byte1
     1     CARD8                           max-byte1
     1     BOOL                            all-chars-exist
     2     INT16                           font-ascent
     2     INT16                           font-descent
     4     m                               number of CHARINFOs in char-infos
     8n     LISTofFONTPROP                 properties
     12m     LISTofCHARINFO                char-infos

  FONTPROP
     4     ATOM                            name
     4     <32-bits>                       value

  CHARINFO
     2     INT16                           left-side-bearing
     2     INT16                           right-side-bearing
     2     INT16                           character-width
     2     INT16                           ascent
     2     INT16                           descent
     2     CARD16                          attributes
*/

#[derive(Debug, Clone)]
pub enum DrawDirection {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone)]
pub struct FontProp {
    pub name: AtomId,
    pub value: u32,
}

impl FontProp {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let name = AtomId::from(conn.read_le_u32()?);
        let value = conn.read_le_u32()?;
        Ok(Self { name, value })
    }
}

#[derive(Debug, Clone)]
pub struct CharInfo {
    left_side_bearing: i16,
    right_side_bearing: i16,
    character_width: i16,
    ascent: i16,
    descent: i16,
    attributes: u16,
}

impl CharInfo {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let left_side_bearing = conn.read_le_i16()?;
        let right_side_bearing = conn.read_le_i16()?;
        let character_width = conn.read_le_i16()?;
        let ascent = conn.read_le_i16()?;
        let descent = conn.read_le_i16()?;
        let attributes = conn.read_le_u16()?;

        Ok(Self {
            left_side_bearing,
            right_side_bearing,
            character_width,
            ascent,
            descent,
            attributes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct QueryFont {
    pub min_bounds: CharInfo,
    pub max_bounds: CharInfo,
    pub min_char_or_byte2: u16,
    pub max_char_or_byte2: u16,
    pub default_char: u16,
    pub properties: Vec<FontProp>,
    pub draw_direction: DrawDirection,
    pub min_byte1: u8,
    pub max_byte1: u8,
    pub all_chars_exist: bool,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub char_infos: Vec<CharInfo>,
}

impl QueryFont {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let min_bounds = CharInfo::from_le_bytes(conn)?;
        drop(conn.drain(4)?);
        let max_bounds = CharInfo::from_le_bytes(conn)?;
        drop(conn.drain(4)?);
        let min_char_or_byte2 = conn.read_le_u16()?;
        let max_char_or_byte2 = conn.read_le_u16()?;
        let default_char = conn.read_le_u16()?;
        let properties_count = conn.read_le_u16()?;
        let draw_direction_code = conn.read_u8()?;
        let draw_direction = match draw_direction_code {
            0 => DrawDirection::LeftToRight,
            1 => DrawDirection::RightToLeft,
            _ => return Err(Error::InvalidResponse),
        };
        let min_byte1 = conn.read_u8()?;
        let max_byte1 = conn.read_u8()?;
        let all_chars_exist = conn.read_bool()?;
        let font_ascent = conn.read_le_i16()?;
        let font_descent = conn.read_le_i16()?;
        let char_infos_count = conn.read_le_u32()?;

        let mut properties = Vec::with_capacity(properties_count as usize);
        for _ in 0..properties_count {
            let property = FontProp::from_le_bytes(conn)?;
            properties.push(property);
        }

        let mut char_infos = Vec::with_capacity(char_infos_count as usize);
        for _ in 0..char_infos_count {
            let char_info = CharInfo::from_le_bytes(conn)?;
            char_infos.push(char_info);
        }

        Ok(Self {
            min_bounds,
            max_bounds,
            min_char_or_byte2,
            max_char_or_byte2,
            default_char,
            properties,
            draw_direction,
            min_byte1,
            max_byte1,
            all_chars_exist,
            font_ascent,
            font_descent,
            char_infos,
        })
    }
}

impl_xreply!(QueryFont);

/*
QueryTextExtents
▶
     1     1                               Reply
     1                                     draw-direction
          0     LeftToRight
          1     RightToLeft
     2     CARD16                          sequence number
     4     0                               reply length
     2     INT16                           font-ascent
     2     INT16                           font-descent
     2     INT16                           overall-ascent
     2     INT16                           overall-descent
     4     INT32                           overall-width
     4     INT32                           overall-left
     4     INT32                           overall-right
     4                                     unused
*/

#[derive(Debug, Clone)]
pub struct QueryTextExtents {
    pub draw_direction: DrawDirection,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub overall_ascent: i16,
    pub overall_descent: i16,
    pub overall_width: i32,
    pub overall_left: i32,
    pub overall_right: i32,
}

impl QueryTextExtents {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let draw_direction_code = conn.read_u8()?;
        let draw_direction = match draw_direction_code {
            0 => DrawDirection::LeftToRight,
            1 => DrawDirection::RightToLeft,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let font_ascent = conn.read_le_i16()?;
        let font_descent = conn.read_le_i16()?;
        let overall_ascent = conn.read_le_i16()?;
        let overall_descent = conn.read_le_i16()?;
        let overall_width = conn.read_le_i32()?;
        let overall_left = conn.read_le_i32()?;
        let overall_right = conn.read_le_i32()?;
        drop(conn.drain(4)?);

        Ok(Self {
            draw_direction,
            font_ascent,
            font_descent,
            overall_ascent,
            overall_descent,
            overall_width,
            overall_left,
            overall_right,
        })
    }
}

impl_xreply!(QueryTextExtents);

/*
ListFonts
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     2     CARD16                          number of STRs in names
     22                                    unused
     n     LISTofSTR                       names
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct ListFonts {
    pub names: ListOfStr,
}

impl ListFonts {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let names_count = conn.read_le_u16()?;
        drop(conn.drain(22)?);
        let names = ListOfStr::from_le_bytes(names_count as usize, conn)?;
        let n = names.encoded_len();
        drop(conn.drain(pad(n))?);

        Ok(Self { names })
    }
}

impl_xreply!(ListFonts);

/*
ListFontsWithInfo
▶ (except for last in series)
     1     1                               Reply
     1     n                               length of name in bytes
     2     CARD16                          sequence number
     4     7+2m+(n+p)/4                    reply length
     12     CHARINFO                       min-bounds
     4                                     unused
     12     CHARINFO                       max-bounds
     4                                     unused
     2     CARD16                          min-char-or-byte2
     2     CARD16                          max-char-or-byte2
     2     CARD16                          default-char
     2     m                               number of FONTPROPs in properties
     1                                     draw-direction
          0     LeftToRight
          1     RightToLeft
     1     CARD8                           min-byte1
     1     CARD8                           max-byte1
     1     BOOL                            all-chars-exist
     2     INT16                           font-ascent
     2     INT16                           font-descent
     4     CARD32                          replies-hint
     8m     LISTofFONTPROP                 properties
     n     STRING8                         name
     p                                     unused, p=pad(n)

  FONTPROP
     encodings are the same as for QueryFont

  CHARINFO
     encodings are the same as for QueryFont

▶ (last in series)
     1     1                               Reply
     1     0                               last-reply indicator
     2     CARD16                          sequence number
     4     7                               reply length
     52                                    unused

*/

#[derive(Debug, Clone)]
pub struct ListFontsWithInfoPiece {
    pub min_bounds: CharInfo,
    pub max_bounds: CharInfo,
    pub min_char_or_byte2: u16,
    pub max_char_or_byte2: u16,
    pub default_char: u16,
    pub properties: Vec<FontProp>,
    pub draw_direction: DrawDirection,
    pub min_byte1: u8,
    pub max_byte1: u8,
    pub all_chars_exist: bool,
    pub font_ascent: i16,
    pub font_descent: i16,
    pub name: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum ListFontsWithInfoPartial {
    ListFontsWithInfoPiece(ListFontsWithInfoPiece),
    ListFontsWithInfoEnd,
}

impl ListFontsWithInfoPartial {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let name_length = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;

        if name_length == 0 {
            drop(conn.drain(52)?);
            Ok(Self::ListFontsWithInfoEnd)
        } else {
            let min_bounds = CharInfo::from_le_bytes(conn)?;
            drop(conn.drain(4)?);
            let max_bounds = CharInfo::from_le_bytes(conn)?;
            drop(conn.drain(4)?);
            let min_char_or_byte2 = conn.read_le_u16()?;
            let max_char_or_byte2 = conn.read_le_u16()?;
            let default_char = conn.read_le_u16()?;
            let properties_count = conn.read_le_u16()?;
            let draw_direction_code = conn.read_u8()?;
            let draw_direction = match draw_direction_code {
                0 => DrawDirection::LeftToRight,
                1 => DrawDirection::RightToLeft,
                _ => return Err(Error::InvalidResponse),
            };
            let min_byte1 = conn.read_u8()?;
            let max_byte1 = conn.read_u8()?;
            let all_chars_exist = conn.read_bool()?;
            let font_ascent = conn.read_le_i16()?;
            let font_descent = conn.read_le_i16()?;
            drop(conn.drain(4)?);

            let mut properties = Vec::with_capacity(properties_count as usize);
            for _ in 0..properties_count {
                properties.push(FontProp::from_le_bytes(conn)?);
            }

            let name = conn.read_n_bytes(name_length as usize)?;
            drop(conn.drain(pad(name_length as usize))?);

            Ok(Self::ListFontsWithInfoPiece(ListFontsWithInfoPiece {
                min_bounds,
                max_bounds,
                min_char_or_byte2,
                max_char_or_byte2,
                default_char,
                properties,
                draw_direction,
                min_byte1,
                max_byte1,
                all_chars_exist,
                font_ascent,
                font_descent,
                name,
            }))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ListFontsWithInfo {
    pub replies: Vec<ListFontsWithInfoPiece>,
}

impl_xreply!(ListFontsWithInfo);

/*
GetFontPath
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     2     CARD16                          number of STRs in path
     22                                    unused
     n     LISTofSTR                       path
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct GetFontPath {
    pub paths: ListOfStr,
}

impl GetFontPath {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let no_of_strs_in_path = conn.read_le_u16()?;
        drop(conn.drain(22)?); // unused
        let paths = ListOfStr::from_le_bytes(no_of_strs_in_path as usize, conn)?;

        Ok(Self { paths })
    }
}

impl_xreply!(GetFontPath);

/*
GetImage
▶
     1     1                               Reply
     1     CARD8                           depth
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     4     VISUALID                        visual
          0     None
     20                                    unused
     n     LISTofBYTE                      data
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct GetImage {
    pub depth: u8,
    pub visual: OrNone<VisualId>,
    pub data: Vec<u8>,
}

impl GetImage {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let reply_length = conn.read_le_u32()?;
        let visual = OrNone::new(VisualId::from(conn.read_le_u32()?));
        drop(conn.drain(20)?);
        let data_length = reply_length * 4; // NOTE: Check this math
        let data = conn.read_n_bytes(data_length as usize)?;
        let _ = conn.drain(pad(data_length as usize))?;

        Ok(Self {
            depth,
            visual,
            data,
        })
    }
}

impl_xreply!(GetImage);

/*
ListInstalledColormaps
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     n                               reply length
     2     n                               number of COLORMAPs in cmaps
     22                                    unused
     4n     LISTofCOLORMAP                 cmaps
*/

#[derive(Debug, Clone)]
pub struct ListInstalledColormaps {
    pub cmaps: Vec<ColormapId>,
}

impl ListInstalledColormaps {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let cmaps_count = conn.read_le_u16()?;
        drop(conn.drain(22)?);

        let mut cmaps = Vec::with_capacity(cmaps_count as usize);
        for _ in 0..cmaps_count {
            let colormap = ColormapId::from(conn.read_le_u32()?);
            cmaps.push(colormap);
        }

        Ok(Self { cmaps })
    }
}

impl_xreply!(ListInstalledColormaps);

/*
AllocColor
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     2     CARD16                          red
     2     CARD16                          green
     2     CARD16                          blue
     2                                     unused
     4     CARD32                          pixel
     12                                    unused
*/

#[derive(Debug, Clone)]
pub struct AllocColor {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub pixel: u32,
}

impl AllocColor {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let red = conn.read_le_u16()?;
        let green = conn.read_le_u16()?;
        let blue = conn.read_le_u16()?;
        drop(conn.drain(2)?);
        let pixel = conn.read_le_u32()?;
        drop(conn.drain(12)?);

        Ok(Self {
            red,
            green,
            blue,
            pixel,
        })
    }
}

impl_xreply!(AllocColor);

/*
AllocNamedColor
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     4     CARD32                          pixel
     2     CARD16                          exact-red
     2     CARD16                          exact-green
     2     CARD16                          exact-blue
     2     CARD16                          visual-red
     2     CARD16                          visual-green
     2     CARD16                          visual-blue
     8                                     unused
*/

#[derive(Debug, Clone)]
pub struct AllocNamedColor {
    pub pixel: u32,
    pub exact_red: u16,
    pub exact_green: u16,
    pub exact_blue: u16,
    pub visual_red: u16,
    pub visual_green: u16,
    pub visual_blue: u16,
}

impl AllocNamedColor {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let pixel = conn.read_le_u32()?;
        let exact_red = conn.read_le_u16()?;
        let exact_green = conn.read_le_u16()?;
        let exact_blue = conn.read_le_u16()?;
        let visual_red = conn.read_le_u16()?;
        let visual_green = conn.read_le_u16()?;
        let visual_blue = conn.read_le_u16()?;
        drop(conn.drain(8)?);

        Ok(Self {
            pixel,
            exact_red,
            exact_green,
            exact_blue,
            visual_red,
            visual_green,
            visual_blue,
        })
    }
}

impl_xreply!(AllocNamedColor);

/*
AllocColorCells
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     n+m                             reply length
     2     n                               number of CARD32s in pixels
     2     m                               number of CARD32s in masks
     20                                    unused
     4n     LISTofCARD32                   pixels
     4m     LISTofCARD32                   masks
*/

#[derive(Debug, Clone)]
pub struct AllocColorCells {
    pub pixels: Vec<u32>,
    pub masks: Vec<u32>,
}

impl AllocColorCells {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let pixels_count = conn.read_le_u16()?;
        let masks_count = conn.read_le_u16()?;
        drop(conn.drain(20)?);

        let mut pixels = Vec::with_capacity(pixels_count as usize);
        for _ in 0..pixels_count {
            let pixel = conn.read_le_u32()?;
            pixels.push(pixel);
        }

        let mut masks = Vec::with_capacity(masks_count as usize);
        for _ in 0..masks_count {
            let mask = conn.read_le_u32()?;
            masks.push(mask);
        }

        Ok(Self { pixels, masks })
    }
}

impl_xreply!(AllocColorCells);

/*
AllocColorPlanes
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     n                               reply length
     2     n                               number of CARD32s in pixels
     2                                     unused
     4     CARD32                          red-mask
     4     CARD32                          green-mask
     4     CARD32                          blue-mask
     8                                     unused
     4n     LISTofCARD32                   pixels
*/

#[derive(Debug, Clone)]
pub struct AllocColorPlanes {
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pixels: Vec<u32>,
}

impl AllocColorPlanes {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let pixels_count = conn.read_le_u16()?;
        let _unused = conn.read_le_u16()?;
        let red_mask = conn.read_le_u32()?;
        let green_mask = conn.read_le_u32()?;
        let blue_mask = conn.read_le_u32()?;
        drop(conn.drain(8)?);

        // TODO: Optimize it
        let mut pixels = Vec::with_capacity(pixels_count as usize);
        for _ in 0..pixels_count {
            let pixel = conn.read_le_u32()?;
            pixels.push(pixel);
        }

        Ok(Self {
            red_mask,
            green_mask,
            blue_mask,
            pixels,
        })
    }
}

impl_xreply!(AllocColorPlanes);

/*
QueryColors
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     2n                              reply length
     2     n                               number of RGBs in colors
     22                                    unused
     8n     LISTofRGB                      colors

  RGB
     2     CARD16                          red
     2     CARD16                          green
     2     CARD16                          blue
     2                                     unused
*/

#[derive(Debug, Clone)]
pub struct Rgb {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl Rgb {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let red = conn.read_le_u16()?;
        let green = conn.read_le_u16()?;
        let blue = conn.read_le_u16()?;
        let _unused = conn.read_le_u16()?;
        Ok(Rgb { red, green, blue })
    }
}

#[derive(Debug, Clone)]
pub struct QueryColors {
    pub colors: Vec<Rgb>,
}

impl QueryColors {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let colors_count = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);

        let mut colors = Vec::with_capacity(colors_count);
        for _ in 0..colors_count {
            colors.push(Rgb::from_le_bytes(conn)?);
        }

        Ok(Self { colors })
    }
}

impl_xreply!(QueryColors);

/*
LookupColor
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     2     CARD16                          exact-red
     2     CARD16                          exact-green
     2     CARD16                          exact-blue
     2     CARD16                          visual-red
     2     CARD16                          visual-green
     2     CARD16                          visual-blue
     12                                    unused
*/

#[derive(Debug, Clone)]
pub struct LookupColor {
    pub exact_red: u16,
    pub exact_green: u16,
    pub exact_blue: u16,
    pub visual_red: u16,
    pub visual_green: u16,
    pub visual_blue: u16,
}

impl LookupColor {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let exact_red = conn.read_le_u16()?;
        let exact_green = conn.read_le_u16()?;
        let exact_blue = conn.read_le_u16()?;
        let visual_red = conn.read_le_u16()?;
        let visual_green = conn.read_le_u16()?;
        let visual_blue = conn.read_le_u16()?;
        drop(conn.drain(12)?);

        Ok(Self {
            exact_red,
            exact_green,
            exact_blue,
            visual_red,
            visual_green,
            visual_blue,
        })
    }
}

impl_xreply!(LookupColor);

/*
QueryBestSize
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     2     CARD16                          width
     2     CARD16                          height
     20                                    unused
*/

#[derive(Debug, Clone)]
pub struct QueryBestSize {
    pub width: u16,
    pub height: u16,
}

impl QueryBestSize {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let width = conn.read_le_u16()?;
        let height = conn.read_le_u16()?;
        drop(conn.drain(20)?);

        Ok(Self { width, height })
    }
}

impl_xreply!(QueryBestSize);

/*
QueryExtension
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     1     BOOL                            present
     1     CARD8                           major-opcode
     1     CARD8                           first-event
     1     CARD8                           first-error
     20                                    unused
*/

#[derive(Debug, Clone)]
pub struct QueryExtension {
    pub present: bool,
    pub major_opcode: u8,
    pub first_event: u8,
    pub first_error: u8,
}

impl QueryExtension {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let present = conn.read_bool()?;
        let major_opcode = conn.read_u8()?;
        let first_event = conn.read_u8()?;
        let first_error = conn.read_u8()?;
        drop(conn.drain(20)?);

        Ok(Self {
            present,
            major_opcode,
            first_event,
            first_error,
        })
    }
}

impl_xreply!(QueryExtension);

/*
ListExtensions
▶
     1     1                               Reply
     1     CARD8                           number of STRs in names
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     24                                    unused
     n     LISTofSTR                       names
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct ListExtensions {
    pub names: ListOfStr,
}

impl ListExtensions {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let number_of_names = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        drop(conn.drain(24)?);
        let names = ListOfStr::from_le_bytes(number_of_names as usize, conn)?;
        drop(conn.drain(pad(names.encoded_len()))?);

        Ok(Self { names })
    }
}

impl_xreply!(ListExtensions);

/*
GetKeyboardMapping
▶
     1     1                               Reply
     1     n                               keysyms-per-keycode
     2     CARD16                          sequence number
     4     nm                              reply length (m = count field
                                           from the request)
     24                                    unused
     4nm     LISTofKEYSYM                  keysyms
*/

// TODO:  Read KeySym decoding section
#[derive(Debug, Clone)]
pub struct KeySym {
    inner: u32,
}

impl KeySym {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let inner = conn.read_le_u32()?;
        Ok(Self { inner })
    }
}

#[derive(Debug, Clone)]
pub struct GetKeyboardMapping {
    pub keysyms_per_keycode: u8,
    pub keysyms: Vec<KeySym>,
}

impl GetKeyboardMapping {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let keysyms_per_keycode = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let reply_length = conn.read_le_u32()?;
        drop(conn.drain(24)?);

        let m = reply_length as usize / 4;
        let mut keysyms = Vec::with_capacity(m);
        for _ in 0..m {
            let keysym = KeySym::from_le_bytes(conn)?;
            keysyms.push(keysym);
        }

        Ok(Self {
            keysyms_per_keycode,
            keysyms,
        })
    }
}

impl_xreply!(GetKeyboardMapping);

/*
GetKeyboardControl
▶
     1     1                               Reply
     1                                     global-auto-repeat
          0     Off
          1     On
     2     CARD16                          sequence number
     4     5                               reply length
     4     CARD32                          led-mask
     1     CARD8                           key-click-percent
     1     CARD8                           bell-percent
     2     CARD16                          bell-pitch
     2     CARD16                          bell-duration
     2                                     unused
     32     LISTofCARD8                    auto-repeats
*/

#[derive(Debug, Clone)]
pub struct GetKeyboardControl {
    pub global_auto_repeat: bool,
    pub led_mask: u32,
    pub key_click_percent: u8,
    pub bell_percent: u8,
    pub bell_pitch: u16,
    pub bell_duration: u16,
    pub auto_repeats: Vec<u8>,
}

impl GetKeyboardControl {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let global_auto_repeat_byte = conn.read_u8()?;
        let global_auto_repeat = global_auto_repeat_byte != 0;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let led_mask = conn.read_le_u32()?;
        let key_click_percent = conn.read_u8()?;
        let bell_percent = conn.read_u8()?;
        let bell_pitch = conn.read_le_u16()?;
        let bell_duration = conn.read_le_u16()?;
        drop(conn.drain(2)?);
        let auto_repeats = conn.read_n_bytes(32)?;

        Ok(Self {
            global_auto_repeat,
            led_mask,
            key_click_percent,
            bell_percent,
            bell_pitch,
            bell_duration,
            auto_repeats,
        })
    }
}

impl_xreply!(GetKeyboardControl);

/*
GetPointerControl
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     2     CARD16                          acceleration-numerator
     2     CARD16                          acceleration-denominator
     2     CARD16                          threshold
     18                                    unused
*/

#[derive(Debug, Clone)]
pub struct GetPointerControl {
    pub acceleration_numerator: u16,
    pub acceleration_denominator: u16,
    pub threshold: u16,
}

impl GetPointerControl {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let acceleration_numerator = conn.read_le_u16()?;
        let acceleration_denominator = conn.read_le_u16()?;
        let threshold = conn.read_le_u16()?;
        drop(conn.drain(18)?);

        Ok(Self {
            acceleration_numerator,
            acceleration_denominator,
            threshold,
        })
    }
}

impl_xreply!(GetPointerControl);

/*
GetScreenSaver
▶
     1     1                               Reply
     1                                     unused
     2     CARD16                          sequence number
     4     0                               reply length
     2     CARD16                          timeout
     2     CARD16                          interval
     1                                     prefer-blanking
          0     No
          1     Yes
     1                                     allow-exposures
          0     No
          1     Yes
     18                                    unused
*/

#[derive(Debug, Clone)]
pub struct GetScreenSaver {
    pub timeout: u16,
    pub interval: u16,
    pub prefer_blanking: bool,
    pub allow_exposures: bool,
}

impl GetScreenSaver {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let timeout = conn.read_le_u16()?;
        let interval = conn.read_le_u16()?;
        let prefer_blanking_byte = conn.read_u8()?;
        let prefer_blanking = prefer_blanking_byte != 0;
        let allow_exposures_byte = conn.read_u8()?;
        let allow_exposures = allow_exposures_byte != 0;
        let _unused = conn.drain(18)?;

        Ok(Self {
            timeout,
            interval,
            prefer_blanking,
            allow_exposures,
        })
    }
}

impl_xreply!(GetScreenSaver);

/*
ListHosts
▶
     1     1                               Reply
     1                                     mode
          0     Disabled
          1     Enabled
     2     CARD16                          sequence number
     4     n/4                             reply length
     2     CARD16                          number of HOSTs in hosts
     22                                    unused
     n     LISTofHOST                      hosts (n always a multiple of 4)
*/

#[derive(Debug, Clone)]
pub enum HostMode {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone)]
pub struct ListHosts {
    pub mode: HostMode,
    pub hosts: Vec<Host>,
}

impl ListHosts {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let mode_byte = conn.read_u8()?;
        let mode = match mode_byte {
            0 => HostMode::Disabled,
            1 => HostMode::Enabled,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let num_hosts = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);

        // Read the hosts
        let mut hosts = Vec::with_capacity(num_hosts);
        for _ in 0..num_hosts {
            hosts.push(Host::from_le_bytes(conn)?);
        }

        Ok(Self { mode, hosts })
    }
}

impl_xreply!(ListHosts);

/*
SetPointerMapping
▶
     1     1                               Reply
     1                                     status
          0     Success
          1     Busy
     2     CARD16                          sequence number
     4     0                               reply length
     24                                    unused
*/

#[derive(Debug, Clone)]
pub enum SetPointerMappingStatus {
    Success,
    Busy,
}

#[derive(Debug, Clone)]
pub struct SetPointerMapping {
    pub status: SetPointerMappingStatus,
}

impl SetPointerMapping {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status_byte = conn.read_u8()?;
        let status = match status_byte {
            0 => SetPointerMappingStatus::Success,
            1 => SetPointerMappingStatus::Busy,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        drop(conn.drain(24)?);

        Ok(Self { status })
    }
}

impl_xreply!(SetPointerMapping);

/*
GetPointerMapping
▶
     1     1                               Reply
     1     n                               length of map
     2     CARD16                          sequence number
     4     (n+p)/4                         reply length
     24                                    unused
     n     LISTofCARD8                     map
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct GetPointerMapping {
    pub map: Vec<u8>,
}

impl GetPointerMapping {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let length_of_map = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        drop(conn.drain(24)?);
        let map = conn.read_n_bytes(length_of_map as usize)?;
        drop(conn.drain(pad(length_of_map as usize))?);

        Ok(Self { map })
    }
}

impl_xreply!(GetPointerMapping);

/*
SetModifierMapping
▶
     1     1                               Reply
     1                                     status
          0     Success
          1     Busy
          2     Failed
     2     CARD16                          sequence number
     4     0                               reply length
     24                                    unused
*/

#[derive(Debug, Clone)]
pub enum SetModifierMappingStatus {
    Success,
    Busy,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SetModifierMapping {
    pub status: SetModifierMappingStatus,
}

impl SetModifierMapping {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status_byte = conn.read_u8()?;
        let status = match status_byte {
            0 => SetModifierMappingStatus::Success,
            1 => SetModifierMappingStatus::Busy,
            2 => SetModifierMappingStatus::Failed,
            _ => return Err(Error::InvalidResponse),
        };
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        drop(conn.drain(24)?);

        Ok(Self { status })
    }
}

impl_xreply!(SetModifierMapping);

/*
GetModifierMapping
▶
     1     1                               Reply
     1     n                               keycodes-per-modifier
     2     CARD16                          sequence number
     4     2n                              reply length
     24                                    unused
     8n     LISTofKEYCODE                  keycodes
*/

#[derive(Debug, Clone)]
pub struct GetModifierMapping {
    pub keycodes_per_modifier: u8,
    pub keycodes: Vec<[KeyCode; 8]>,
}

impl GetModifierMapping {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let keycodes_per_modifier = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        drop(conn.drain(24)?);

        let num_keycodes = keycodes_per_modifier as usize;
        let mut keycodes = Vec::with_capacity(num_keycodes);
        for _ in 0..num_keycodes {
            let mut key_code = [0u8; 8];
            conn.read_exact(&mut key_code)?;
            keycodes.push(key_code.map(KeyCode::from));
        }

        Ok(Self {
            keycodes_per_modifier,
            keycodes,
        })
    }
}

impl_xreply!(GetModifierMapping);

// Wrappers

#[derive(Debug, Clone)]
pub enum SomeReply {
    GetWindowAttributes(GetWindowAttributes),
    GetGeometry(GetGeometry),
    QueryTree(QueryTree),
    InternAtom(InternAtom),
    GetAtomName(GetAtomName),
    GetProperty(GetProperty),
    ListProperties(ListProperties),
    GetSelectionOwner(GetSelectionOwner),
    GrabPointer(GrabPointer),
    GrabKeyboard(GrabKeyboard),
    QueryPointer(QueryPointer),
    GetMotionEvents(GetMotionEvents),
    TranslateCoordinates(TranslateCoordinates),
    GetInputFocus(GetInputFocus),
    QueryKeymap(QueryKeymap),
    QueryFont(QueryFont),
    QueryTextExtents(QueryTextExtents),
    ListFonts(ListFonts),
    ListFontsWithInfo(ListFontsWithInfo),
    // NOTE: Fake reply type because `ListFontsWithInfo` comes in multiple replies for each font
    ListFontsWithInfoPartial(ListFontsWithInfoPartial),
    GetFontPath(GetFontPath),
    GetImage(GetImage),
    ListInstalledColormaps(ListInstalledColormaps),
    AllocColor(AllocColor),
    AllocNamedColor(AllocNamedColor),
    AllocColorCells(AllocColorCells),
    AllocColorPlanes(AllocColorPlanes),
    QueryColors(QueryColors),
    LookupColor(LookupColor),
    QueryBestSize(QueryBestSize),
    QueryExtension(QueryExtension),
    ListExtensions(ListExtensions),
    GetKeyboardMapping(GetKeyboardMapping),
    GetKeyboardControl(GetKeyboardControl),
    GetPointerControl(GetPointerControl),
    GetScreenSaver(GetScreenSaver),
    ListHosts(ListHosts),
    SetPointerMapping(SetPointerMapping),
    GetPointerMapping(GetPointerMapping),
    SetModifierMapping(SetModifierMapping),
    GetModifierMapping(GetModifierMapping),
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

#[derive(Debug, Clone)]
pub(crate) struct ReceivedReply {
    pub(crate) reply_type: ReplyType,
    pub(crate) reply: SomeReply,
    pub(crate) done_receiving: bool,
}

impl ReceivedReply {
    pub(crate) fn append_reply(&mut self, reply: SomeReply) -> bool {
        if self.done_receiving {
            return false;
        }

        match &mut self.reply {
            SomeReply::ListFontsWithInfo(list_fonts) => match reply {
                SomeReply::ListFontsWithInfoPartial(
                    ListFontsWithInfoPartial::ListFontsWithInfoEnd,
                ) => {
                    self.done_receiving = true;
                }
                SomeReply::ListFontsWithInfoPartial(
                    ListFontsWithInfoPartial::ListFontsWithInfoPiece(piece),
                ) => {
                    list_fonts.replies.push(piece);
                }
                _ => return false,
            },
            _ => return false,
        }

        true
    }
}

#[derive(Debug, Clone)]
pub(crate) enum AwaitingReply {
    /// Response not received yet, but when received will be inserted to the map of responses
    NotReceived(ReplyType),

    /// Response not received yet, but when received will be discared
    Discarded(ReplyType),

    /// Response already received and waiting to be used or discarded or for more information to finalize
    Received(ReceivedReply),
}

impl AwaitingReply {
    pub(crate) fn reply_type(&self) -> ReplyType {
        match self {
            &AwaitingReply::NotReceived(reply_type) | &AwaitingReply::Discarded(reply_type) => {
                reply_type
            }
            AwaitingReply::Received(received) => received.reply_type,
        }
    }
}
