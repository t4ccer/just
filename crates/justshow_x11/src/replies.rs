use std::{fmt::Display, ops::Deref, str::FromStr};

use crate::{
    atoms::AtomId,
    connection::XConnection,
    error::Error,
    keysym::KeySym,
    requests::KeyCode,
    requests::Timestamp,
    utils::{impl_enum, pad},
    xerror::SomeError,
    ColormapId, FromLeBytes, ListOfStr, OrNone, ResourceId, VisualId, WindowId,
};

pub trait XReply: Sized {
    type Error;

    fn from_reply(reply: SomeReply) -> Option<Self>;
    fn from_error(error: SomeError) -> Option<Self::Error>;
}

macro_rules! impl_xreply {
    ($t:tt) => {
        impl XReply for $t {
            type Error = $crate::xerror::SomeError;

            #[inline(always)]
            fn from_reply(reply: SomeReply) -> Option<Self> {
                match reply {
                    SomeReply::$t(r) => Some(r),
                    _ => None,
                }
            }

            #[inline(always)]
            fn from_error(error: SomeError) -> Option<Self::Error> {
                Some(error)
            }
        }
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

impl_enum! {
    #[repr(u8)]
    enum HostFamily {
        Internet = 0,
        DECnet = 1,
        Chaos = 2,
        ServerImplemented = 5,
        InternetV6 = 6,
    }
}

#[derive(Debug, Clone)]
pub struct Host {
    pub family: HostFamily,
    pub address: Vec<u8>,
}

impl FromLeBytes for Host {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
    pub backing_store: u8,
    pub visual_id: u32,
    pub class: u16,
    pub bit_gravity: u8,
    pub win_gravity: u8,
    pub backing_planes: u32,
    pub backing_pixel: u32,
    pub save_under: bool,
    pub map_is_installed: bool,
    pub map_state: u8,
    pub override_redirect: bool,
    pub colormap: u32,
    pub all_even_masks: u32,
    pub your_even_masks: u32,
    pub do_not_propagate_mask: u16,
}

impl FromLeBytes for GetWindowAttributes {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for GetGeometry {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

// TODO: Move somewhere else
macro_rules! read_vec {
    ($count:expr, $generator:expr) => {{
        let mut res = Vec::with_capacity($count as usize);
        for _ in 0..$count {
            res.push($generator);
        }
        res
    }};
}
pub(crate) use read_vec;

impl FromLeBytes for QueryTree {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
        let children = read_vec!(
            children_count,
            WindowId(ResourceId {
                value: conn.read_le_u32()?,
            })
        );

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
    pub atom: AtomId, // TODO: Or none
}

impl FromLeBytes for InternAtom {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let atom = AtomId::unchecked_from(conn.read_le_u32()?);
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

// TODO: Correct 'String Equivalence' from spec
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String8(String);

impl Display for String8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl String8 {
    #[inline(always)]
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        // TODO: Proper ISO Latin-1 decoding
        String::from_utf8(bytes).map_or(None, |s| Some(Self(s)))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl FromStr for String8 {
    type Err = std::convert::Infallible;

    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from(s)))
    }
}

impl Deref for String8 {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone)]
pub struct GetAtomName {
    pub name: String8,
}

impl FromLeBytes for GetAtomName {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let name_length = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);
        let name = conn.read_n_bytes(name_length)?;
        let name = String8::from_bytes(name).ok_or(Error::InvalidResponse("String8"))?;
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

impl FromLeBytes for GetProperty {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let format = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let type_ = AtomId::unchecked_from(conn.read_le_u32()?);
        let bytes_after = conn.read_le_u32()?;
        let length_of_value = conn.read_le_u32()?;
        drop(conn.drain(12)?);
        let value_length = length_of_value as usize * (format as usize / 8);
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

impl FromLeBytes for ListProperties {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_code = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let atom_count = conn.read_le_u16()?;
        drop(conn.drain(22)?);
        let atoms = read_vec!(atom_count, AtomId::unchecked_from(conn.read_le_u32()?));

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

impl FromLeBytes for GetSelectionOwner {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl_enum! {
    #[repr(u8)]
    enum GrabPointerStatus {
        Success = 0,
        AlreadyGrabbed = 1,
        InvalidTime = 2,
        NotViewable = 3,
        Frozen = 4,
    }
}

#[derive(Debug, Clone)]
pub struct GrabPointer {
    pub status: GrabPointerStatus,
}

impl FromLeBytes for GrabPointer {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = GrabPointerStatus::from_le_bytes(conn)?;
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

impl_enum! {
    #[repr(u8)]
    enum GrabKeyboardStatus {
        Success = 0,
        AlreadyGrabbed = 1,
        InvalidTime = 2,
        NotViewable = 3,
        Frozen = 4,
    }
}

#[derive(Debug, Clone)]
pub struct GrabKeyboard {
    pub status: GrabKeyboardStatus,
}

impl FromLeBytes for GrabKeyboard {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = GrabKeyboardStatus::from_le_bytes(conn)?;
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

impl FromLeBytes for QueryPointer {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let same_screen = conn.read_bool()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = WindowId(ResourceId {
            value: conn.read_le_u32()?,
        });
        let child = OrNone::new(WindowId::unchecked_from(conn.read_le_u32()?));
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

impl FromLeBytes for GetMotionEvents {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let event_count = conn.read_le_u32()? as usize;
        drop(conn.drain(20)?);

        let mut events = Vec::with_capacity(event_count);
        for _ in 0..event_count {
            let time = Timestamp::from(conn.read_le_u32()?);
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

impl FromLeBytes for TranslateCoordinates {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl_enum! {
    #[repr(u8)]
    enum RevertTo {
        None = 0,
        PointerRoot = 1,
        Parent = 2,
    }
}

#[derive(Debug, Clone)]
pub struct GetInputFocus {
    pub revert_to: RevertTo,
    pub focus: Focus,
}

impl FromLeBytes for GetInputFocus {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let revert_to = RevertTo::from_le_bytes(conn)?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let focus_value = conn.read_le_u32()?;
        let focus = match focus_value {
            0 => Focus::None,
            1 => Focus::PointerRoot,
            wid => Focus::Window(WindowId::unchecked_from(wid)),
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

impl FromLeBytes for QueryKeymap {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl_enum! {
    #[repr(u8)]
    enum DrawDirection {
        LeftToRight = 0,
        RightToLeft = 1,
    }
}

#[derive(Debug, Clone)]
pub struct FontProp {
    pub name: AtomId,
    pub value: u32,
}

impl FromLeBytes for FontProp {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let name = AtomId::unchecked_from(conn.read_le_u32()?);
        let value = conn.read_le_u32()?;
        Ok(Self { name, value })
    }
}

#[derive(Debug, Clone)]
pub struct CharInfo {
    pub left_side_bearing: i16,
    pub right_side_bearing: i16,
    pub character_width: i16,
    pub ascent: i16,
    pub descent: i16,
    pub attributes: u16,
}

impl FromLeBytes for CharInfo {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for QueryFont {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
        let draw_direction = DrawDirection::from_le_bytes(conn)?;
        let min_byte1 = conn.read_u8()?;
        let max_byte1 = conn.read_u8()?;
        let all_chars_exist = conn.read_bool()?;
        let font_ascent = conn.read_le_i16()?;
        let font_descent = conn.read_le_i16()?;
        let char_infos_count = conn.read_le_u32()?;

        let properties = read_vec!(properties_count, FontProp::from_le_bytes(conn)?);
        let char_infos = read_vec!(char_infos_count, CharInfo::from_le_bytes(conn)?);

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

impl FromLeBytes for QueryTextExtents {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let draw_direction = DrawDirection::from_le_bytes(conn)?;
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

impl FromLeBytes for ListFonts {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for ListFontsWithInfoPartial {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
            let draw_direction = DrawDirection::from_le_bytes(conn)?;
            let min_byte1 = conn.read_u8()?;
            let max_byte1 = conn.read_u8()?;
            let all_chars_exist = conn.read_bool()?;
            let font_ascent = conn.read_le_i16()?;
            let font_descent = conn.read_le_i16()?;
            drop(conn.drain(4)?);

            let properties = read_vec!(properties_count, FontProp::from_le_bytes(conn)?);

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

impl FromLeBytes for GetFontPath {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for GetImage {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let reply_length = conn.read_le_u32()?;
        let visual = OrNone::new(VisualId::unchecked_from(conn.read_le_u32()?));
        drop(conn.drain(20)?);
        let data_length = reply_length * 4; // NOTE: Check this math
        let data = conn.read_n_bytes(data_length as usize)?;
        drop(conn.drain(pad(data_length as usize))?);

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

impl FromLeBytes for ListInstalledColormaps {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let cmaps_count = conn.read_le_u16()?;
        drop(conn.drain(22)?);
        let cmaps = read_vec!(cmaps_count, ColormapId::unchecked_from(conn.read_le_u32()?));

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

impl FromLeBytes for AllocColor {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for AllocNamedColor {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for AllocColorCells {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let pixels_count = conn.read_le_u16()?;
        let masks_count = conn.read_le_u16()?;
        drop(conn.drain(20)?);
        let pixels = read_vec!(pixels_count, conn.read_le_u32()?);
        let masks = read_vec!(masks_count, conn.read_le_u32()?);

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

impl FromLeBytes for AllocColorPlanes {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let pixels_count = conn.read_le_u16()?;
        let _unused = conn.read_le_u16()?;
        let red_mask = conn.read_le_u32()?;
        let green_mask = conn.read_le_u32()?;
        let blue_mask = conn.read_le_u32()?;
        drop(conn.drain(8)?);
        let pixels = read_vec!(pixels_count, conn.read_le_u32()?);

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

impl FromLeBytes for Rgb {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for QueryColors {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let colors_count = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);
        let colors = read_vec!(colors_count, Rgb::from_le_bytes(conn)?);

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

impl FromLeBytes for LookupColor {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for QueryBestSize {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for QueryExtension {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for ListExtensions {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

#[derive(Debug, Clone)]
pub struct GetKeyboardMapping {
    pub keysyms_per_keycode: u8,
    pub keysyms: Vec<KeySym>,
}

impl FromLeBytes for GetKeyboardMapping {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let n = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let reply_length = conn.read_le_u32()?;
        drop(conn.drain(24)?);

        let m = reply_length as usize / n as usize;
        let keysyms = read_vec!(n as usize * m, KeySym::from_le_bytes(conn)?);

        Ok(Self {
            keysyms_per_keycode: n,
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

impl FromLeBytes for GetKeyboardControl {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for GetPointerControl {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl FromLeBytes for GetScreenSaver {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let timeout = conn.read_le_u16()?;
        let interval = conn.read_le_u16()?;
        let prefer_blanking_byte = conn.read_u8()?;
        let prefer_blanking = prefer_blanking_byte != 0;
        let allow_exposures_byte = conn.read_u8()?;
        let allow_exposures = allow_exposures_byte != 0;
        drop(conn.drain(18)?);

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

impl_enum! {
    #[repr(u8)]
    enum HostMode {
        Disabled = 0,
        Enabled = 1,
    }
}

#[derive(Debug, Clone)]
pub struct ListHosts {
    pub mode: HostMode,
    pub hosts: Vec<Host>,
}

impl FromLeBytes for ListHosts {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let mode = HostMode::from_le_bytes(conn)?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()? as usize;
        let num_hosts = conn.read_le_u16()? as usize;
        drop(conn.drain(22)?);
        let hosts = read_vec!(num_hosts, Host::from_le_bytes(conn)?);

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

impl_enum! {
    #[repr(u8)]
    enum SetPointerMappingStatus {
        Success = 0,
        Busy = 1,
    }
}

#[derive(Debug, Clone)]
pub struct SetPointerMapping {
    pub status: SetPointerMappingStatus,
}

impl FromLeBytes for SetPointerMapping {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = SetPointerMappingStatus::from_le_bytes(conn)?;
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

impl FromLeBytes for GetPointerMapping {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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

impl_enum! {
    #[repr(u8)]
    enum SetModifierMappingStatus {
        Success = 0,
        Busy = 1,
        Failed = 2,
    }
}

#[derive(Debug, Clone)]
pub struct SetModifierMapping {
    pub status: SetModifierMappingStatus,
}

impl FromLeBytes for SetModifierMapping {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = SetModifierMappingStatus::from_le_bytes(conn)?;
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

impl FromLeBytes for GetModifierMapping {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
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
    ExtensionRandr(crate::extensions::randr::replies::SomeReply),
}

#[derive(Debug, Clone, Copy)]
pub enum ReplyType {
    GetWindowAttributes,
    GetGeometry,
    QueryTree,
    InternAtom,
    GetAtomName,
    GetProperty,
    ListProperties,
    GetSelectionOwner,
    GrabPointer,
    GrabKeyboard,
    QueryPointer,
    GetMotionEvents,
    TranslateCoordinates,
    GetInputFocus,
    QueryKeymap,
    QueryFont,
    QueryTextExtents,
    ListFonts,
    ListFontsWithInfo,
    GetFontPath,
    GetImage,
    ListInstalledColormaps,
    AllocColor,
    AllocNamedColor,
    AllocColorCells,
    AllocColorPlanes,
    QueryColors,
    LookupColor,
    QueryBestSize,
    QueryExtension,
    ListExtensions,
    GetKeyboardMapping,
    GetKeyboardControl,
    GetPointerControl,
    GetScreenSaver,
    ListHosts,
    SetPointerMapping,
    GetPointerMapping,
    SetModifierMapping,
    GetModifierMapping,
    ExtensionRandr(crate::extensions::randr::replies::ReplyType),
}

#[derive(Debug, Clone)]
pub(crate) struct ReceivedReply {
    pub(crate) reply_type: ReplyType,
    pub(crate) reply: Result<SomeReply, ()>,
    pub(crate) done_receiving: bool,
}

impl ReceivedReply {
    pub(crate) fn append_reply(&mut self, reply: SomeReply) -> bool {
        if self.done_receiving {
            return false;
        }

        match &mut self.reply {
            Ok(SomeReply::ListFontsWithInfo(list_fonts)) => match reply {
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
