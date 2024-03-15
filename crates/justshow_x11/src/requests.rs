use crate::{
    atoms::AtomId,
    events::{self, EventType, StackMode},
    keysym::KeySym,
    replies::{ReplyType, String8},
    utils::{bitmask, impl_enum, pad},
    ColormapId, CursorId, Drawable, FontId, FromLeBytes, GContextId, ListOfStr, OrNone, PixmapId,
    Point, Rectangle, ToLeBytes, VisualId, WindowClass, WindowId, WindowVisual,
};
use std::{
    fmt,
    io::{self, Write},
    mem,
};

pub(crate) mod opcodes;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timestamp {
    CurrentTime,
    Timestamp(u32),
}

impl From<u32> for Timestamp {
    fn from(value: u32) -> Self {
        if value == 0 {
            Self::CurrentTime
        } else {
            Self::Timestamp(value)
        }
    }
}

impl From<Timestamp> for u32 {
    fn from(value: Timestamp) -> Self {
        match value {
            Timestamp::CurrentTime => 0,
            Timestamp::Timestamp(t) => t,
        }
    }
}

impl FromLeBytes for Timestamp {
    fn from_le_bytes(
        conn: &mut crate::connection::XConnection,
    ) -> Result<Self, crate::error::Error> {
        let inner: u32 = FromLeBytes::from_le_bytes(conn)?;
        Ok(Self::from(inner))
    }
}

impl Timestamp {
    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
        let raw: u32 = self.into();
        raw.to_le_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KeyCode(u8);

impl KeyCode {
    #[inline(always)]
    fn to_le_bytes(self) -> [u8; 1] {
        self.0.to_le_bytes()
    }

    #[inline(always)]
    pub fn raw(self) -> u8 {
        self.0
    }
}

impl From<u8> for KeyCode {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<u32> for KeyCode {
    fn from(value: u32) -> Self {
        Self(value as u8)
    }
}

impl From<KeyCode> for u32 {
    fn from(value: KeyCode) -> Self {
        value.0 as u32
    }
}

macro_rules! impl_value {
    ($ty:ident as) => {
        #[automatically_derived]
        impl $crate::requests::Value for $ty {
            fn to_raw_value(self) -> u32 {
                self as u32
            }
        }
    };

    ($ty:ident into) => {
        #[automatically_derived]
        impl $crate::requests::Value for $ty {
            fn to_raw_value(self) -> u32 {
                self.into()
            }
        }
    };
}
pub(crate) use impl_value;

pub(crate) trait Value: Sized {
    fn to_raw_value(self) -> u32;
}

impl_value!(u8 as);
impl_value!(i8 as);
impl_value!(u16 as);
impl_value!(i16 as);
impl_value!(u32 as);
impl_value!(i32 as);
impl_value!(bool as);

impl_value!(EventType into);
impl_value!(KeyCode into);
impl_value!(StackMode into);

macro_rules! impl_raw_fields_go {
    ($idx:expr $(,)?) => { };

    ($idx:expr, $(#[$field_attr:meta])* $setter:ident: $ty:path, $($rest:tt)*) => {
        $(#[$field_attr])*
        pub fn $setter(mut self, new_value: $ty) -> Self {
            self.values.values[$idx] = Some(new_value.to_raw_value());
            self
        }

        impl_raw_fields_go!($idx + 1, $($rest)*);
    };
}

macro_rules! impl_raw_fields_debug {
    ($d:expr, $self:expr, $idx:expr, $(,)?) => { };

    ($d:expr, $self:expr, $idx:expr, $(#[$field_attr:meta])* $setter:ident : $ty:path, $($rest:tt)*) => {
        // strip `set_` prfix
        $d.field(&stringify!($setter)[4..], &$self.values.values[$idx]);
        impl_raw_fields_debug!($d, $self, $idx + 1, $($rest)*);
    };
}

macro_rules! impl_raw_fields {
    ($(#[$name_attr:meta])* $name:ident [$size:literal] { $($rest:tt)* }) => {
        #[derive(Clone)]
        $(#[$name_attr])*
        pub struct $name {
            values: ListOfValues<$size>,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    values: ListOfValues::new(),
                }
            }

            impl_raw_fields_go!(0, $($rest)*);
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut d = f.debug_struct(stringify!($name));
                impl_raw_fields_debug!(d, self, 0, $($rest)*);
                d.finish()
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct NoReply;

pub trait XRequestBase: ToLeBytes {
    type Reply;

    fn reply_type() -> Option<ReplyType>;
}

pub trait XRequest: XRequestBase {}

pub trait XExtensionRequest: XRequestBase {}

macro_rules! impl_xrequest_with_response {
    ($r:tt) => {
        impl $crate::requests::XRequestBase for $r {
            type Reply = $crate::replies::$r;

            #[inline(always)]
            fn reply_type() -> Option<$crate::replies::ReplyType> {
                Some($crate::replies::ReplyType::$r)
            }
        }

        impl $crate::requests::XRequest for $r {}
    };
}

macro_rules! impl_xrequest_without_response {
    ($r:tt) => {
        impl $crate::requests::XRequestBase for $r {
            type Reply = $crate::requests::NoReply;

            #[inline(always)]
            fn reply_type() -> Option<$crate::replies::ReplyType> {
                None
            }
        }

        impl $crate::requests::XRequest for $r {}
    };
}

macro_rules! write_le_bytes {
    ($w:expr, $content:expr) => {
        $w.write_all(&(($content).to_le_bytes()))?;
    };
}
pub(crate) use write_le_bytes;

#[derive(Debug, Clone)]
pub struct ListOfValues<const N: usize> {
    values: [Option<u32>; N],
}

impl<const N: usize> Default for ListOfValues<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> ListOfValues<N> {
    pub fn new() -> Self {
        Self { values: [None; N] }
    }

    pub fn mask_and_count(&self) -> (u32, u16) {
        let mut bitmask: u32 = 0;
        let mut n: u16 = 0;

        for value in self.values.iter().rev() {
            bitmask <<= 1;
            bitmask |= value.is_some() as u32;
            n += value.is_some() as u16;
        }

        (bitmask, n)
    }

    pub fn to_le_bytes_if_set(&self, w: &mut impl Write) -> io::Result<()> {
        for value in self.values.iter().flatten() {
            write_le_bytes!(w, value);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum XProtocolVersion {
    V11_0,
}

#[derive(Debug, Clone)]
pub struct InitializeConnection {
    pub major_version: u16,
    pub minor_version: u16,
    pub authorization_protocol_name: Vec<u8>,
    pub authorization_protocol_data: Vec<u8>,
}

impl InitializeConnection {
    pub(crate) fn new(
        protocol_version: XProtocolVersion,
        authorization_protocol_name: Vec<u8>,
        authorization_protocol_data: Vec<u8>,
    ) -> Self {
        match protocol_version {
            XProtocolVersion::V11_0 => Self {
                major_version: 11,
                minor_version: 0,
                authorization_protocol_name,
                authorization_protocol_data,
            },
        }
    }
}

impl ToLeBytes for InitializeConnection {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let authorization_name_len = self.authorization_protocol_name.len();
        let authorization_name_pad = pad(authorization_name_len);
        let authorization_data_len = self.authorization_protocol_data.len();
        let authorization_data_pad = pad(authorization_data_len);

        w.write_all(b"l\0")?;
        write_le_bytes!(w, self.major_version);
        write_le_bytes!(w, self.minor_version);
        write_le_bytes!(w, authorization_name_len as u16);
        write_le_bytes!(w, authorization_data_len as u16);
        w.write_all(&[0u8; 2])?; // unused
        w.write_all(&self.authorization_protocol_name)?;
        w.write_all(&vec![0u8; authorization_name_pad])?; // unused, pad
        w.write_all(&self.authorization_protocol_data)?;
        w.write_all(&vec![0u8; authorization_data_pad])?; // unused, pad

        Ok(())
    }
}

impl_xrequest_without_response!(InitializeConnection);

// Source: https://www.x.org/releases/X11R7.7/doc/xproto/x11protocol.html#Encoding::Requests

/*
CreateWindow
     1     1                               opcode
     1     CARD8                           depth
     2     8+n                             request length
     4     WINDOW                          wid
     4     WINDOW                          parent
     2     INT16                           x
     2     INT16                           y
     2     CARD16                          width
     2     CARD16                          height
     2     CARD16                          border-width
     2                                     class
          0     CopyFromParent
          1     InputOutput
          2     InputOnly
     4     VISUALID                        visual
          0     CopyFromParent
     4     BITMASK                         value-mask (has n bits set to 1)
          #x00000001     background-pixmap
          #x00000002     background-pixel
          #x00000004     border-pixmap
          #x00000008     border-pixel
          #x00000010     bit-gravity
          #x00000020     win-gravity
          #x00000040     backing-store
          #x00000080     backing-planes
          #x00000100     backing-pixel
          #x00000200     override-redirect
          #x00000400     save-under
          #x00000800     event-mask
          #x00001000     do-not-propagate-mask
          #x00002000     colormap
          #x00004000     cursor
     4n     LISTofVALUE                    value-list

  VALUEs
     4     PIXMAP                          background-pixmap
          0     None
          1     ParentRelative
     4     CARD32                          background-pixel
     4     PIXMAP                          border-pixmap
          0     CopyFromParent
     4     CARD32                          border-pixel
     1     BITGRAVITY                      bit-gravity
     1     WINGRAVITY                      win-gravity
     1                                     backing-store
          0     NotUseful
          1     WhenMapped
          2     Always
     4     CARD32                          backing-planes
     4     CARD32                          backing-pixel
     1     BOOL                            override-redirect
     1     BOOL                            save-under
     4     SETofEVENT                      event-mask
     4     SETofDEVICEEVENT                do-not-propagate-mask
     4     COLORMAP                        colormap
          0     CopyFromParent
     4     CURSOR                          cursor
          0     None

ChangeWindowAttributes
     1     2                               opcode
     1                                     unused
     2     3+n                             request length
     4     WINDOW                          window
     4     BITMASK                         value-mask (has n bits set to 1)
          encodings are the same as for CreateWindow
     4n     LISTofVALUE                    value-list
          encodings are the same as for CreateWindow
*/
impl_raw_fields! {
    /// Test comment
    WindowCreationAttributes[15] {
        /// Test comment: Set background color.
        set_background_pixmap: u32,
        set_background_pixel: u32,
        set_border_pixmap: u32,
        set_border_pixel: u32,
        set_bit_gravity: u32,
        set_win_gravity: u32,
        set_backing_store: u32,
        set_backing_planes: u32,
        set_backing_pixel: u32,
        set_override_redirect: u32,
        set_save_under: u32,
        set_event_mask: EventType,
        set_do_not_propagate_mask: u32,
        set_colormap: u32,
        set_cursor: u32,
    }
}

#[derive(Debug, Clone)]
pub struct CreateWindow {
    pub depth: u8,
    pub wid: WindowId,
    pub parent: WindowId,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub window_class: WindowClass,
    pub visual: WindowVisual,
    pub attributes: WindowCreationAttributes,
}

impl ToLeBytes for CreateWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.attributes.values.mask_and_count();

        write_le_bytes!(w, opcodes::CREATE_WINDOW);
        write_le_bytes!(w, self.depth);
        write_le_bytes!(w, 8u16 + n); // length
        write_le_bytes!(w, self.wid);
        write_le_bytes!(w, self.parent);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.border_width);
        write_le_bytes!(w, self.window_class as u16);
        write_le_bytes!(w, self.visual.value());
        write_le_bytes!(w, bitmask);
        self.attributes.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(CreateWindow);

/*
ChangeWindowAttributes
     1     2                               opcode
     1                                     unused
     2     3+n                             request length
     4     WINDOW                          window
     4     BITMASK                         value-mask (has n bits set to 1)
          encodings are the same as for CreateWindow
     4n     LISTofVALUE                    value-list
          encodings are the same as for CreateWindow
*/

#[derive(Debug, Clone)]
pub struct ChangeWindowAttributes {
    pub window: WindowId,
    pub attributes: WindowCreationAttributes,
}

impl ToLeBytes for ChangeWindowAttributes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.attributes.values.mask_and_count();

        write_le_bytes!(w, opcodes::CHANGE_WINDOW_ATTRIBUTES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + n); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, bitmask);
        self.attributes.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeWindowAttributes);

/*
GetWindowAttributes
     1     3                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct GetWindowAttributes {
    pub window: WindowId,
}

impl ToLeBytes for GetWindowAttributes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_WINDOW_ATTRIBUTES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(GetWindowAttributes);

/*
DestroyWindow
     1     4                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct DestroyWindow {
    pub window: WindowId,
}

impl ToLeBytes for DestroyWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DESTROY_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(DestroyWindow);

/*
DestroySubwindows
     1     5                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct DestroySubwindows {
    pub window: WindowId,
}

impl ToLeBytes for DestroySubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DESTROY_SUBWINDOWS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(DestroySubwindows);

/*
ChangeSaveSet
     1     6                               opcode
     1                                     mode
          0     Insert
          1     Delete
     2     2                               request length
     4     WINDOW                          window
*/

impl_enum! {
    #[repr(u8)]
    enum ChangeSaveSetMode {
        Insert = 0,
        Delete = 1,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeSaveSet {
    pub mode: ChangeSaveSetMode,
    pub window: WindowId,
}

impl ToLeBytes for ChangeSaveSet {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CHANGE_SAVE_SET);
        write_le_bytes!(w, self.mode as u8);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeSaveSet);

/*
ReparentWindow
     1     7                               opcode
     1                                     unused
     2     4                               request length
     4     WINDOW                          window
     4     WINDOW                          parent
     2     INT16                           x
     2     INT16                           y
 */

#[derive(Debug, Clone, Copy)]
pub struct ReparentWindow {
    pub window: WindowId,
    pub parent: WindowId,
    pub x: i16,
    pub y: i16,
}

impl ToLeBytes for ReparentWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::REPARENT_WINDOW);
        write_le_bytes!(w, 0u8); //unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.parent);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);

        Ok(())
    }
}

impl_xrequest_without_response!(ReparentWindow);

/*
MapWindow
     1     8                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct MapWindow {
    pub window: WindowId,
}

impl ToLeBytes for MapWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::MAP_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(MapWindow);

/*
MapSubwindows
     1     9                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct MapSubwindows {
    pub window: WindowId,
}

impl ToLeBytes for MapSubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::MAP_SUBWINDOWS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(MapSubwindows);

/*
UnmapWindow
     1     10                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct UnmapWindow {
    pub window: WindowId,
}

impl ToLeBytes for UnmapWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNMAP_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(UnmapWindow);

/*
UnmapSubwindows
     1     11                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct UnmapSubwindows {
    pub window: WindowId,
}

impl ToLeBytes for UnmapSubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNMAP_SUBWINDOWS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(UnmapSubwindows);

/*
ConfigureWindow
     1     12                              opcode
     1                                     unused
     2     3+n                             request length
     4     WINDOW                          window
     2     BITMASK                         value-mask (has n bits set to 1)
          #x0001     x
          #x0002     y
          #x0004     width
          #x0008     height
          #x0010     border-width
          #x0020     sibling
          #x0040     stack-mode
     2               unused
     4n     LISTofVALUE                    value-list
*/

impl_raw_fields! {
    ConfigureWindowAttributes[7] {
        set_x: i16,
        set_y: i16,
        set_width: u16,
        set_height: u16,
        set_border_width: u16,
        set_sibling: WindowId,
        set_stack_mode: StackMode,
    }
}

impl From<&events::ConfigureRequest> for ConfigureWindowAttributes {
    fn from(event: &events::ConfigureRequest) -> Self {
        let attributes = ConfigureWindowAttributes::new()
            .set_x(event.x)
            .set_y(event.y)
            .set_width(event.width)
            .set_height(event.height)
            .set_border_width(event.border_width)
            .set_stack_mode(event.stack_mode);

        match event.sibling.value() {
            None => attributes,
            Some(sibling) => attributes.set_sibling(sibling),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigureWindow {
    pub window: WindowId,
    pub attributes: ConfigureWindowAttributes,
}

impl ToLeBytes for ConfigureWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.attributes.values.mask_and_count();

        write_le_bytes!(w, opcodes::CONFIGURE_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, (3 + n) as u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, bitmask as u16);
        write_le_bytes!(w, 0u16); // unused
        self.attributes.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(ConfigureWindow);

/*
CirculateWindow
     1     13                              opcode
     1                                     direction
          0     RaiseLowest
          1     LowerHighest
     2     2                               request length
     4     WINDOW                          window
*/

impl_enum! {
    #[repr(u8)]
    enum CirculateWindowDirection {
        RaiseLowest = 0,
        LowerHighest = 1,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CirculateWindow {
    pub direction: CirculateWindowDirection,
    pub window: WindowId,
}

impl ToLeBytes for CirculateWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CIRCULATE_WINDOW);
        write_le_bytes!(w, self.direction as u8);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_without_response!(CirculateWindow);

/*
GetGeometry
     1     14                              opcode
     1                                     unused
     2     2                               request length
     4     DRAWABLE                        drawable
*/

#[derive(Debug, Clone, Copy)]
pub struct GetGeometry {
    pub drawable: Drawable,
}

impl ToLeBytes for GetGeometry {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_GEOMETRY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.drawable.value());

        Ok(())
    }
}

impl_xrequest_with_response!(GetGeometry);

/*
QueryTree
     1     15                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryTree {
    pub window: WindowId,
}

impl ToLeBytes for QueryTree {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_TREE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(QueryTree);

/*
InternAtom
     1     16                              opcode
     1     BOOL                            only-if-exists
     2     2+(n+p)/4                       request length
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct InternAtom {
    pub only_if_exists: bool,
    pub name: String8,
}

impl ToLeBytes for InternAtom {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_len = (2 + (n + p) / 4) as u16;

        write_le_bytes!(w, opcodes::INTERN_ATOM);
        write_le_bytes!(w, self.only_if_exists as u8);
        write_le_bytes!(w, request_len);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_with_response!(InternAtom);

/*
GetAtomName
     1     17                              opcode
     1                                     unused
     2     2                               request length
     4     ATOM                            atom
*/

#[derive(Debug, Clone, Copy)]
pub struct GetAtomName {
    pub atom: AtomId,
}

impl ToLeBytes for GetAtomName {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_ATOM_NAME);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.atom);

        Ok(())
    }
}

impl_xrequest_with_response!(GetAtomName);

/*
ChangeProperty
     1     18                              opcode
     1                                     mode
          0     Replace
          1     Prepend
          2     Append
     2     6+(n+p)/4                       request length
     4     WINDOW                          window
     4     ATOM                            property
     4     ATOM                            type
     1     CARD8                           format
     3                                     unused
     4     CARD32                          length of data in format units
                    (= n for format = 8)
                    (= n/2 for format = 16)
                    (= n/4 for format = 32)
     n     LISTofBYTE                      data
                    (n is a multiple of 2 for format = 16)
                    (n is a multiple of 4 for format = 32)
     p                                     unused, p=pad(n)
*/

impl_enum! {
    #[repr(u8)]
    enum ChangePropertyMode {
        Replace = 0,
        Prepend = 1,
        Append = 2,
    }
}

impl_enum! {
    #[repr(u8)]
    enum ChangePropertyFormat {
        Format8 = 8,
        Format16 = 16,
        Format32 = 32,
    }
}

#[derive(Debug, Clone)]
pub struct ChangeProperty {
    pub mode: ChangePropertyMode,
    pub window: WindowId,
    pub property: AtomId,
    pub type_: AtomId,
    pub format: ChangePropertyFormat,
    pub data: Vec<u8>,
}

impl ToLeBytes for ChangeProperty {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.data.len();
        let p = pad(n);
        let request_len = 6 + (n + p) / 4;

        write_le_bytes!(w, opcodes::CHANGE_PROPERTY);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, request_len);
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.property);
        write_le_bytes!(w, self.type_);
        write_le_bytes!(w, self.format);
        w.write_all(&[0u8; 3])?; // unused
        write_le_bytes!(w, (n as u64) / (self.format as u64 / 8));
        w.write_all(&self.data)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeProperty);

/*
DeleteProperty
     1     19                              opcode
     1                                     unused
     2     3                               request length
     4     WINDOW                          window
     4     ATOM                            property
*/

#[derive(Debug, Clone, Copy)]
pub struct DeleteProperty {
    pub window: WindowId,
    pub property: AtomId,
}

impl ToLeBytes for DeleteProperty {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DELETE_PROPERTY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.property);

        Ok(())
    }
}

impl_xrequest_without_response!(DeleteProperty);

/*
GetProperty
     1     20                              opcode
     1     BOOL                            delete
     2     6                               request length
     4     WINDOW                          window
     4     ATOM                            property
     4     ATOM                            type
          0     AnyPropertyType
     4     CARD32                          long-offset
     4     CARD32                          long-length
*/

#[derive(Debug, Clone, Copy)]
pub struct GetProperty {
    pub delete: bool,
    pub window: WindowId,
    pub property: AtomId,
    pub type_: AtomId, // TODO: Type
    pub long_offset: u32,
    pub long_length: u32,
}

impl ToLeBytes for GetProperty {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_PROPERTY);
        write_le_bytes!(w, self.delete as u8);
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.property);
        write_le_bytes!(w, self.type_);
        write_le_bytes!(w, self.long_offset);
        write_le_bytes!(w, self.long_length);

        Ok(())
    }
}

impl_xrequest_with_response!(GetProperty);

/*
ListProperties
     1     21                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct ListProperties {
    pub window: WindowId,
}

impl ToLeBytes for ListProperties {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::LIST_PROPERTIES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(ListProperties);

/*
SetSelectionOwner
     1     22                              opcode
     1                                     unused
     2     4                               request length
     4     WINDOW                          owner
          0     None
     4     ATOM                            selection
     4     TIMESTAMP                       time
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct SetSelectionOwner {
    pub owner: OrNone<WindowId>,
    pub selection: AtomId,
    pub time: Timestamp,
}

impl ToLeBytes for SetSelectionOwner {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_SELECTION_OWNER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.owner.0);
        write_le_bytes!(w, self.selection);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(SetSelectionOwner);

/*
GetSelectionOwner
     1     23                              opcode
     1                                     unused
     2     2                               request length
     4     ATOM                            selection
*/

#[derive(Debug, Clone, Copy)]
pub struct GetSelectionOwner {
    pub selection: AtomId,
}

impl ToLeBytes for GetSelectionOwner {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SELECTION_OWNER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.selection);

        Ok(())
    }
}

impl_xrequest_with_response!(GetSelectionOwner);

/*
ConvertSelection
     1     24                              opcode
     1                                     unused
     2     6                               request length
     4     WINDOW                          requestor
     4     ATOM                            selection
     4     ATOM                            target
     4     ATOM                            property
          0     None
     4     TIMESTAMP                       time
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct ConvertSelection {
    pub requestor: WindowId,
    pub selection: AtomId,
    pub target: AtomId,
    pub property: OrNone<AtomId>,
    pub time: Timestamp,
}

impl ToLeBytes for ConvertSelection {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CONVERT_SELECTION);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.requestor);
        write_le_bytes!(w, self.selection);
        write_le_bytes!(w, self.target);
        write_le_bytes!(w, self.property.0);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(ConvertSelection);

/*
SendEvent
     1     25                              opcode
     1     BOOL                            propagate
     2     11                              request length
     4     WINDOW                          destination
          0     PointerWindow
          1     InputFocus
     4     SETofEVENT                      event-mask
     32                                    event
          standard event format (see the Events section)
*/

#[derive(Debug, Clone, Copy)]
pub struct SendEvent {
    pub propagate: bool,
    pub destination: WindowId, // TODO: Type
    pub event_mask: u32,       // TODO: Type
    pub event: [u8; 32],       // TODO: type
}

impl ToLeBytes for SendEvent {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SEND_EVENT);
        write_le_bytes!(w, self.propagate as u8);
        write_le_bytes!(w, 11u16); // length
        write_le_bytes!(w, self.destination);
        write_le_bytes!(w, self.event_mask);
        w.write_all(&self.event)?;

        Ok(())
    }
}

impl_xrequest_without_response!(SendEvent);

/*
GrabPointer
     1     26                              opcode
     1     BOOL                            owner-events
     2     6                               request length
     4     WINDOW                          grab-window
     2     SETofPOINTEREVENT               event-mask
     1                                     pointer-mode
          0     Synchronous
          1     Asynchronous
     1                                     keyboard-mode
          0     Synchronous
          1     Asynchronous
     4     WINDOW                          confine-to
          0     None
     4     CURSOR                          cursor
          0     None
     4     TIMESTAMP                       time
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct GrabPointer {
    pub owner_events: bool,
    pub grab_window: WindowId,
    pub event_mask: u16, // TODO: Type
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
    pub confine_to: OrNone<WindowId>,
    pub cursor: OrNone<CursorId>,
    pub time: Timestamp,
}

impl ToLeBytes for GrabPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_POINTER);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        write_le_bytes!(w, self.confine_to.0);
        write_le_bytes!(w, self.cursor.0);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_with_response!(GrabPointer);

/*
UngrabPointer
     1     27                              opcode
     1                                     unused
     2     2                               request length
     4     TIMESTAMP                       time
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabPointer {
    pub time: Timestamp,
}

impl ToLeBytes for UngrabPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(UngrabPointer);

/*
GrabButton
     1     28                              opcode
     1     BOOL                            owner-events
     2     6                               request length
     4     WINDOW                          grab-window
     2     SETofPOINTEREVENT               event-mask
     1                                     pointer-mode
          0     Synchronous
          1     Asynchronous
     1                                     keyboard-mode
          0     Synchronous
          1     Asynchronous
     4     WINDOW                          confine-to
          0     None
     4     CURSOR                          cursor
          0     None
     1     BUTTON                          button
          0     AnyButton
     1                                     unused
     2     SETofKEYMASK                    modifiers
          #x8000                           AnyModifier
*/

#[derive(Debug, Clone, Copy)]
pub struct GrabButton {
    pub owner_events: bool,
    pub grab_window: WindowId,
    pub event_mask: u16, // TODO: type
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
    pub confine_to: OrNone<WindowId>,
    pub cursor: OrNone<CursorId>,
    pub button: u8,     // TODO: Type
    pub modifiers: u16, // TODO: Type
}

impl ToLeBytes for GrabButton {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_BUTTON);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        write_le_bytes!(w, self.confine_to.0);
        write_le_bytes!(w, self.cursor.0);
        write_le_bytes!(w, self.button);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, self.modifiers);

        Ok(())
    }
}

impl_xrequest_without_response!(GrabButton);

/*
UngrabButton
     1     29                              opcode
     1     BUTTON                          button
          0     AnyButton
     2     3                               request length
     4     WINDOW                          grab-window
     2     SETofKEYMASK                    modifiers
          #x8000                           AnyModifier
     2                                     unused
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabButton {
    pub button: u8, // TODO: Type
    pub grab_window: WindowId,
    pub modifiers: u16, // TODO: Type
}

impl ToLeBytes for UngrabButton {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_BUTTON);
        write_le_bytes!(w, self.button);
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.modifiers);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_without_response!(UngrabButton);

/*
ChangeActivePointerGrab
     1     30                              opcode
     1                                     unused
     2     4                               request length
     4     CURSOR                          cursor
          0     None
     4     TIMESTAMP                       time
          0     CurrentTime
     2     SETofPOINTEREVENT               event-mask
     2                                     unused
*/

#[derive(Debug, Clone, Copy)]
pub struct ChangeActivePointerGrab {
    pub cursor: OrNone<CursorId>,
    pub time: Timestamp,
    pub event_mask: u16, // TODO: TYpe
}

impl ToLeBytes for ChangeActivePointerGrab {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CHANGE_ACTIVE_POINTER_GRAB);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.cursor.0);
        write_le_bytes!(w, self.time);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeActivePointerGrab);

/*
GrabKeyboard
     1     31                              opcode
     1     BOOL                            owner-events
     2     4                               request length
     4     WINDOW                          grab-window
     4     TIMESTAMP                       time
          0     CurrentTime
     1                                     pointer-mode
          0     Synchronous
          1     Asynchronous
     1                                     keyboard-mode
          0     Synchronous
          1     Asynchronous
     2                                     unused
*/

#[derive(Debug, Clone, Copy)]
pub struct GrabKeyboard {
    pub owner_events: bool,
    pub grab_window: WindowId,
    pub time: Timestamp,
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
}

impl ToLeBytes for GrabKeyboard {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_KEYBOARD);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.time);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_with_response!(GrabKeyboard);

/*
UngrabKeyboard
     1     32                              opcode
     1                                     unused
     2     2                               request length
     4     TIMESTAMP                       time
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabKeyboard {
    pub time: u32,
}

impl ToLeBytes for UngrabKeyboard {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_KEYBOARD);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(UngrabKeyboard);

/*
GrabKey
     1     33                              opcode
     1     BOOL                            owner-events
     2     4                               request length
     4     WINDOW                          grab-window
     2     SETofKEYMASK                    modifiers
          #x8000     AnyModifier
     1     KEYCODE                         key
          0     AnyKey
     1                                     pointer-mode
          0     Synchronous
          1     Asynchronous
     1                                     keyboard-mode
          0     Synchronous
          1     Asynchronous
     3                                     unused
*/

impl_enum! {
    #[repr(u8)]
    enum GrabMode {
        Synchronous = 0,
        Asynchronous = 1,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GrabKey {
    pub owner_events: bool,
    pub grab_window: WindowId,
    pub modifiers: KeyModifier,
    pub key: KeyCode,
    pub pointer_mode: GrabMode,
    pub keyboard_mode: GrabMode,
}

impl ToLeBytes for GrabKey {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_KEY);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.modifiers.raw() as u16);
        write_le_bytes!(w, self.key);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        w.write_all(&[0u8; 3])?; // unused

        Ok(())
    }
}

impl_xrequest_without_response!(GrabKey);

/*
UngrabKey
     1     34                              opcode
     1     KEYCODE                         key
          0     AnyKey
     2     3                               request length
     4     WINDOW                          grab-window
     2     SETofKEYMASK                    modifiers
          #x8000     AnyModifier
     2                                     unused
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabKey {
    pub key: u8, // TODO: Type
    pub grab_window: WindowId,
    pub modifiers: u16, // TODO: Tyep
}

impl ToLeBytes for UngrabKey {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_KEY);
        write_le_bytes!(w, self.key);
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.modifiers);
        w.write_all(&[0u8; 2])?; // unused

        Ok(())
    }
}

impl_xrequest_without_response!(UngrabKey);

/*
AllowEvents
     1     35                              opcode
     1                                     mode
          0     AsyncPointer
          1     SyncPointer
          2     ReplayPointer
          3     AsyncKeyboard
          4     SyncKeyboard
          5     ReplayKeyboard
          6     AsyncBoth
          7     SyncBoth
     2     2                               request length
     4     TIMESTAMP                       time
          0     CurrentTime
*/

impl_enum! {
    #[repr(u8)]
    enum AllowEventsMode {
        AsyncPointer = 0,
        SyncPointer = 1,
        ReplayPointer = 2,
        AsyncKeyboard = 3,
        SyncKeyboard = 4,
        ReplayKeyboard = 5,
        AsyncBoth = 6,
        SyncBoth = 7,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllowEvents {
    pub mode: AllowEventsMode,
    pub time: u32,
}

impl ToLeBytes for AllowEvents {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::ALLOW_EVENTS);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(AllowEvents);

/*
GrabServer
     1     36                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct GrabServer;

impl ToLeBytes for GrabServer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_SERVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl_xrequest_without_response!(GrabServer);

/*
UngrabServer
     1     37                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabServer;

impl ToLeBytes for UngrabServer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_SERVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl_xrequest_without_response!(UngrabServer);

/*
QueryPointer
     1     38                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryPointer {
    pub window: WindowId,
}

impl ToLeBytes for QueryPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(QueryPointer);

/*
GetMotionEvents
     1     39                              opcode
     1                                     unused
     2     4                               request length
     4     WINDOW                          window
     4     TIMESTAMP                       start
          0     CurrentTime
     4     TIMESTAMP                       stop
          0     CurrentTime
*/

#[derive(Debug, Clone, Copy)]
pub struct GetMotionEvents {
    pub window: WindowId,
    pub start: Timestamp,
    pub stop: Timestamp,
}

impl ToLeBytes for GetMotionEvents {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_MOTION_EVENTS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.start);
        write_le_bytes!(w, self.stop);

        Ok(())
    }
}

impl_xrequest_with_response!(GetMotionEvents);

/*
TranslateCoordinates
     1     40                              opcode
     1                                     unused
     2     4                               request length
     4     WINDOW                          src-window
     4     WINDOW                          dst-window
     2     INT16                           src-x
     2     INT16                           src-y
*/

#[derive(Debug, Clone, Copy)]
pub struct TranslateCoordinates {
    pub src_window: WindowId,
    pub dst_window: WindowId,
    pub src_x: i16,
    pub src_y: i16,
}

impl ToLeBytes for TranslateCoordinates {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::TRANSLATE_COORDINATES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.src_window);
        write_le_bytes!(w, self.dst_window);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);

        Ok(())
    }
}

impl_xrequest_with_response!(TranslateCoordinates);

/*
WarpPointer
     1     41                              opcode
     1                                     unused
     2     6                               request length
     4     WINDOW                          src-window
          0     None
     4     WINDOW                          dst-window
          0     None
     2     INT16                           src-x
     2     INT16                           src-y
     2     CARD16                          src-width
     2     CARD16                          src-height
     2     INT16                           dst-x
     2     INT16                           dst-y
*/

#[derive(Debug, Clone, Copy)]
pub struct WarpPointer {
    pub src_window: OrNone<WindowId>,
    pub dst_window: OrNone<WindowId>,
    pub src_x: i16,
    pub src_y: i16,
    pub src_width: u16,
    pub src_height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
}

impl ToLeBytes for WarpPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::WARP_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.src_window.0);
        write_le_bytes!(w, self.dst_window.0);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);
        write_le_bytes!(w, self.src_width);
        write_le_bytes!(w, self.src_height);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);

        Ok(())
    }
}

impl_xrequest_without_response!(WarpPointer);

/*
SetInputFocus
     1     42                              opcode
     1                                     revert-to
          0     None
          1     PointerRoot
          2     Parent
     2     3                               request length
     4     WINDOW                          focus
          0     None
          1     PointerRoot
     4     TIMESTAMP                       time
          0     CurrentTime
*/

impl_enum! {
    #[repr(u8)]
    enum RevertTo {
        None = 0,
        PointerRoot = 1,
        Parent = 2,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SetInputFocus {
    pub revert_to: RevertTo,
    pub focus: WindowId, // TODO: type
    pub time: Timestamp,
}

impl ToLeBytes for SetInputFocus {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_INPUT_FOCUS);
        write_le_bytes!(w, self.revert_to);
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.focus);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl_xrequest_without_response!(SetInputFocus);

/*
GetInputFocus
     1     43                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl ToLeBytes for GetInputFocus {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, &opcodes::GET_INPUT_FOCUS);
        write_le_bytes!(w, &0u8); // unused
        write_le_bytes!(w, &1u16); // length

        Ok(())
    }
}

impl_xrequest_with_response!(GetInputFocus);

/*
QueryKeymap
     1     44                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryKeymap;

impl ToLeBytes for QueryKeymap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_KEYMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl_xrequest_with_response!(QueryKeymap);

/*
OpenFont
     1     45                              opcode
     1                                     unused
     2     3+(n+p)/4                       request length
     4     FONT                            fid
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct OpenFont {
    pub fid: FontId,
    pub name: Vec<u8>,
}

impl ToLeBytes for OpenFont {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_length = 3 + (n + p) / 4;

        write_le_bytes!(w, opcodes::OPEN_FONT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16); // length
        write_le_bytes!(w, self.fid);
        write_le_bytes!(w, n as u16); // length of name
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_without_response!(OpenFont);

/*
CloseFont
     1     46                              opcode
     1                                     unused
     2     2                               request length
     4     FONT                            font
*/

#[derive(Debug, Clone, Copy)]
pub struct CloseFont {
    pub font: FontId,
}

impl ToLeBytes for CloseFont {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CLOSE_FONT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.font);

        Ok(())
    }
}

impl_xrequest_without_response!(CloseFont);

/*
QueryFont
     1     47                              opcode
     1                                     unused
     2     2                               request length
     4     FONTABLE                        font
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryFont {
    pub font: FontId, // TODO: Fontable
}

impl ToLeBytes for QueryFont {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_FONT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.font);

        Ok(())
    }
}

impl_xrequest_with_response!(QueryFont);

/*
QueryTextExtents
     1     48                              opcode
     1     BOOL                            odd length, True if p = 2
     2     2+(2n+p)/4                      request length
     4     FONTABLE                        font
     2n     STRING16                       string
     p                                     unused, p=pad(2n)
*/

#[derive(Debug, Clone)]
pub struct QueryTextExtents {
    pub font: FontId, // TODO: Fontable
    pub string: Vec<u16>,
}

impl ToLeBytes for QueryTextExtents {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.string.len();
        let p = pad(2 * n);
        let request_length = 2 + (2 * n + p) / 4;
        let odd_length = p == 2;

        write_le_bytes!(w, opcodes::QUERY_TEXT_EXTENTS);
        write_le_bytes!(w, odd_length as u8);
        write_le_bytes!(w, request_length as u16); // length
        write_le_bytes!(w, self.font);
        w.write_all(
            &self
                .string
                .iter()
                .flat_map(|c| c.to_le_bytes())
                .collect::<Vec<u8>>(),
        )?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_with_response!(QueryTextExtents);

/*
ListFonts
     1     49                              opcode
     1                                     unused
     2     2+(n+p)/4                       request length
     2     CARD16                          max-names
     2     n                               length of pattern
     n     STRING8                         pattern
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct ListFonts {
    pub max_names: u16,
    pub pattern: Vec<u8>,
}

impl ToLeBytes for ListFonts {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.pattern.len();
        let p = pad(n);
        let request_length = 2 + (n + p) / 4;

        write_le_bytes!(w, opcodes::LIST_FONTS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.max_names);
        write_le_bytes!(w, n as u16);
        w.write_all(&self.pattern)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_with_response!(ListFonts);

/*
ListFontsWithInfo
     1     50                              opcode
     1                                     unused
     2     2+(n+p)/4                       request length
     2     CARD16                          max-names
     2     n                               length of pattern
     n     STRING8                         pattern
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct ListFontsWithInfo {
    pub max_names: u16,
    pub pattern: Vec<u8>,
}

impl ToLeBytes for ListFontsWithInfo {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.pattern.len();
        let p = pad(n);
        let request_length = 2 + (n + p) / 4;

        write_le_bytes!(w, opcodes::LIST_FONTS_WITH_INFO);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.max_names);
        write_le_bytes!(w, n as u16);
        w.write_all(&self.pattern)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_with_response!(ListFontsWithInfo);

/*
SetFontPath
     1     51                              opcode
     1                                     unused
     2     2+(n+p)/4                       request length
     2     CARD16                          number of STRs in path
     2                                     unused
     n     LISTofSTR                       path
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct SetFontPath {
    pub paths: ListOfStr,
}

impl ToLeBytes for SetFontPath {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.paths.encoded_len();
        let p = pad(n);
        let request_length = 2 + (n + p) / 4;

        write_le_bytes!(w, opcodes::SET_FONT_PATH);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16); // length
        write_le_bytes!(w, self.paths.strings.len() as u16);
        write_le_bytes!(w, 0u16); // unused
        self.paths.to_le_bytes(w)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl_xrequest_without_response!(SetFontPath);

/*
GetFontPath
     1     52                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct GetFontPath;

impl ToLeBytes for GetFontPath {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_FONT_PATH);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl_xrequest_with_response!(GetFontPath);

/*
CreatePixmap
     1     53                              opcode
     1     CARD8                           depth
     2     4                               request length
     4     PIXMAP                          pid
     4     DRAWABLE                        drawable
     2     CARD16                          width
     2     CARD16                          height
*/

#[derive(Debug, Clone, Copy)]
pub struct CreatePixmap {
    pub depth: u8,
    pub pid: PixmapId,
    pub drawable: Drawable,
    pub width: u16,
    pub height: u16,
}

impl ToLeBytes for CreatePixmap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_PIXMAP);
        write_le_bytes!(w, self.depth);
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.pid);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);

        Ok(())
    }
}

impl_xrequest_without_response!(CreatePixmap);

/*
FreePixmap
     1     54                              opcode
     1                                     unused
     2     2                               request length
     4     PIXMAP                          pixmap
*/

#[derive(Debug, Clone, Copy)]
pub struct FreePixmap {
    pub pixmap: PixmapId,
}

impl ToLeBytes for FreePixmap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::FREE_PIXMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.pixmap);

        Ok(())
    }
}

impl_xrequest_without_response!(FreePixmap);

/*
CreateGC
     1     55                              opcode
     1                                     unused
     2     4+n                             request length
     4     GCONTEXT                        cid
     4     DRAWABLE                        drawable
     4     BITMASK                         value-mask (has n bits set to 1)
          #x00000001     function
          #x00000002     plane-mask
          #x00000004     foreground
          #x00000008     background
          #x00000010     line-width
          #x00000020     line-style
          #x00000040     cap-style
          #x00000080     join-style
          #x00000100     fill-style
          #x00000200     fill-rule
          #x00000400     tile
          #x00000800     stipple
          #x00001000     tile-stipple-x-origin
          #x00002000     tile-stipple-y-origin
          #x00004000     font
          #x00008000     subwindow-mode
          #x00010000     graphics-exposures
          #x00020000     clip-x-origin
          #x00040000     clip-y-origin
          #x00080000     clip-mask
          #x00100000     dash-offset
          #x00200000     dashes
          #x00400000     arc-mode
     4n     LISTofVALUE                    value-list

  VALUEs
     1                                     function
           0     Clear
           1     And
           2     AndReverse
           3     Copy
           4     AndInverted
           5     NoOp
           6     Xor
           7     Or
           8     Nor
           9     Equiv
          10     Invert
          11     OrReverse
          12     CopyInverted
          13     OrInverted
          14     Nand
          15     Set
     4     CARD32                          plane-mask
     4     CARD32                          foreground
     4     CARD32                          background
     2     CARD16                          line-width
     1                                     line-style
          0     Solid
          1     OnOffDash
          2     DoubleDash
     1                                     cap-style
          0     NotLast
          1     Butt
          2     Round
          3     Projecting
     1                                     join-style
          0     Miter
          1     Round
          2     Bevel
     1                                     fill-style
          0     Solid
          1     Tiled
          2     Stippled
          3     OpaqueStippled
     1                                     fill-rule
          0     EvenOdd
          1     Winding
     4     PIXMAP                          tile
     4     PIXMAP                          stipple
     2     INT16                           tile-stipple-x-origin
     2     INT16                           tile-stipple-y-origin
     4     FONT                            font
     1                                     subwindow-mode
          0     ClipByChildren
          1     IncludeInferiors
     1     BOOL                            graphics-exposures
     2     INT16                           clip-x-origin
     2     INT16                           clip-y-origin
     4     PIXMAP                          clip-mask
          0     None
     2     CARD16                          dash-offset
     1     CARD8                           dashes
     1                                     arc-mode
          0     Chord
          1     PieSlice
*/

impl_raw_fields! {
    GContextSettings[23] {
        set_function: u32, // TODO: type
        set_plane_mask: u32,
        set_foreground: u32,
        set_background: u32,
        set_line_width: u16,
        set_line_style: u32, // TODO: type
        set_cap_style: u32, // TODO: type
        set_join_style: u32, // TODO: type
        set_fill_style: u32, // TODO: type
        set_fill_rule: u32, // TODO: type
        set_tile: PixmapId,
        set_stipple: PixmapId,
        set_tile_stipple_x_origin: u16,
        set_tile_stipple_y_origin: u16,
        set_font: FontId,
        set_subwindow_mode: u32,
        set_graphics_exposures: bool,
        set_clip_x_origin: u16,
        set_clip_y_origin: u16,
        set_clip_mask: PixmapId, // TODO: or None
        set_dash_offset: u16,
        set_dashes: u8,
        set_arc_mode: u32,
    }
}

#[derive(Debug, Clone)]
pub struct CreateGC {
    pub cid: GContextId,
    pub drawable: Drawable,
    pub values: GContextSettings,
}

impl ToLeBytes for CreateGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_le_bytes!(w, opcodes::CREATE_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16 + n); // length
        write_le_bytes!(w, self.cid);
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, bitmask);
        self.values.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(CreateGC);

/*
ChangeGC
     1     56                              opcode
     1                                     unused
     2     3+n                             request length
     4     GCONTEXT                        gc
     4     BITMASK                         value-mask (has n bits set to 1)
          encodings are the same as for CreateGC
     4n     LISTofVALUE                    value-list
          encodings are the same as for CreateGC
*/

#[derive(Debug, Clone)]
pub struct ChangeGC {
    pub gcontext: GContextId,
    pub values: GContextSettings,
}

impl ToLeBytes for ChangeGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();

        write_le_bytes!(w, opcodes::CHANGE_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + n); // length
        write_le_bytes!(w, self.gcontext);
        write_le_bytes!(w, bitmask);
        self.values.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeGC);

/*
CopyGC
     1     57                              opcode
     1                                     unused
     2     4                               request length
     4     GCONTEXT                        src-gc
     4     GCONTEXT                        dst-gc
     4     BITMASK                         value-mask
          encodings are the same as for CreateGC
*/

#[derive(Debug, Clone, Copy)]
pub struct CopyGC {
    pub src_gc: GContextId,
    pub dst_gc: GContextId,
    pub value_mask: u32,
}

impl ToLeBytes for CopyGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::COPY_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.src_gc);
        write_le_bytes!(w, self.dst_gc);
        write_le_bytes!(w, self.value_mask);

        Ok(())
    }
}

impl_xrequest_without_response!(CopyGC);

/*
SetDashes
     1     58                              opcode
     1                                     unused
     2     3+(n+p)/4                       request length
     4     GCONTEXT                        gc
     2     CARD16                          dash-offset
     2     n                               length of dashes
     n     LISTofCARD8                     dashes
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct SetDashes {
    pub gc: GContextId,
    pub dash_offset: u16,
    pub dashes: Vec<u8>,
}

impl ToLeBytes for SetDashes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.dashes.len();
        let p = pad(n);
        let request_length = 3 + (n + p) / 4;

        write_le_bytes!(w, opcodes::SET_DASHES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.dash_offset);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.dashes)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(SetDashes);

/*
SetClipRectangles
     1     59                              opcode
     1                                     ordering
          0     UnSorted
          1     YSorted
          2     YXSorted
          3     YXBanded
     2     3+2n                            request length
     4     GCONTEXT                        gc
     2     INT16                           clip-x-origin
     2     INT16                           clip-y-origin
     8n     LISTofRECTANGLE                rectangles
*/

impl_enum! {
    #[repr(u8)]
    enum Ordering {
        UnSorted = 0,
        YSorted = 1,
        YXSorted = 2,
        YXBanded = 3,
    }
}

#[derive(Debug, Clone)]
pub struct SetClipRectangles {
    pub ordering: Ordering,
    pub gc: GContextId,
    pub clip_x_origin: i16,
    pub clip_y_origin: i16,
    pub rectangles: Vec<Rectangle>,
}

impl ToLeBytes for SetClipRectangles {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.rectangles.len();
        let request_length = 3 + 2 * n;

        write_le_bytes!(w, opcodes::SET_CLIP_RECTANGLES);
        write_le_bytes!(w, self.ordering as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.clip_x_origin);
        write_le_bytes!(w, self.clip_y_origin);
        for rectangle in &self.rectangles {
            write_le_bytes!(w, rectangle);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(SetClipRectangles);

/*
FreeGC
     1     60                              opcode
     1                                     unused
     2     2                               request length
     4     GCONTEXT                        gc
*/

#[derive(Debug, Clone)]
pub struct FreeGC {
    pub gc: GContextId,
}

impl ToLeBytes for FreeGC {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::FREE_GC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.gc);

        Ok(())
    }
}

impl_xrequest_without_response!(FreeGC);

/*
ClearArea
     1     61                              opcode
     1     BOOL                            exposures
     2     4                               request length
     4     WINDOW                          window
     2     INT16                           x
     2     INT16                           y
     2     CARD16                          width
     2     CARD16                          height
*/

#[derive(Debug, Clone)]
pub struct ClearArea {
    pub exposures: bool,
    pub window: WindowId,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl ToLeBytes for ClearArea {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CLEAR_AREA);
        write_le_bytes!(w, self.exposures as u8);
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);

        Ok(())
    }
}

impl_xrequest_without_response!(ClearArea);

/*
CopyArea
     1     62                              opcode
     1                                     unused
     2     7                               request length
     4     DRAWABLE                        src-drawable
     4     DRAWABLE                        dst-drawable
     4     GCONTEXT                        gc
     2     INT16                           src-x
     2     INT16                           src-y
     2     INT16                           dst-x
     2     INT16                           dst-y
     2     CARD16                          width
     2     CARD16                          height
*/

#[derive(Debug, Clone)]
pub struct CopyArea {
    pub src_drawable: Drawable,
    pub dst_drawable: Drawable,
    pub gc: GContextId,
    pub src_x: i16,
    pub src_y: i16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub width: u16,
    pub height: u16,
}

impl ToLeBytes for CopyArea {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::COPY_AREA);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 7u16); // request length
        write_le_bytes!(w, self.src_drawable);
        write_le_bytes!(w, self.dst_drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);

        Ok(())
    }
}

impl_xrequest_without_response!(CopyArea);

/*
CopyPlane
     1     63                              opcode
     1                                     unused
     2     8                               request length
     4     DRAWABLE                        src-drawable
     4     DRAWABLE                        dst-drawable
     4     GCONTEXT                        gc
     2     INT16                           src-x
     2     INT16                           src-y
     2     INT16                           dst-x
     2     INT16                           dst-y
     2     CARD16                          width
     2     CARD16                          height
     4     CARD32                          bit-plane
*/

#[derive(Debug, Clone)]
pub struct CopyPlane {
    pub src_drawable: Drawable,
    pub dst_drawable: Drawable,
    pub gc: GContextId,
    pub src_x: i16,
    pub src_y: i16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub width: u16,
    pub height: u16,
    pub bit_plane: u32,
}

impl ToLeBytes for CopyPlane {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::COPY_PLANE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 8u16); // request length
        write_le_bytes!(w, self.src_drawable);
        write_le_bytes!(w, self.dst_drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.bit_plane);

        Ok(())
    }
}

impl_xrequest_without_response!(CopyPlane);

/*
PolyPoint
     1     64                              opcode
     1                                     coordinate-mode
          0     Origin
          1     Previous
     2     3+n                             request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     4n     LISTofPOINT                    points
*/

impl_enum! {
    #[repr(u8)]
    enum CoordinateMode {
        Origin = 0,
        Previous = 1,
    }
}

#[derive(Debug, Clone)]
pub struct PolyPoint {
    pub coordinate_mode: CoordinateMode,
    pub drawable: Drawable,
    pub gc: GContextId,
    pub points: Vec<Point>,
}

impl ToLeBytes for PolyPoint {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.points.len();
        let request_length = 3 + n;

        write_le_bytes!(w, opcodes::POLY_POINT);
        write_le_bytes!(w, self.coordinate_mode as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for point in &self.points {
            write_le_bytes!(w, point);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyPoint);

/*
PolyLine
     1     65                              opcode
     1                                     coordinate-mode
          0     Origin
          1     Previous
     2     3+n                             request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     4n     LISTofPOINT                    points
*/

#[derive(Debug, Clone)]
pub struct PolyLine {
    pub coordinate_mode: CoordinateMode,
    pub drawable: Drawable,
    pub gc: GContextId,
    pub points: Vec<Point>,
}

impl ToLeBytes for PolyLine {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.points.len();
        let request_length = 3 + n;

        write_le_bytes!(w, opcodes::POLY_LINE);
        write_le_bytes!(w, self.coordinate_mode as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for point in &self.points {
            write_le_bytes!(w, point);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyLine);

/*
PolySegment
     1     66                              opcode
     1                                     unused
     2     3+2n                            request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     8n     LISTofSEGMENT                  segments

  SEGMENT
     2     INT16                           x1
     2     INT16                           y1
     2     INT16                           x2
     2     INT16                           y2
*/

#[derive(Debug, Clone)]
pub struct Segment {
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
}

impl ToLeBytes for Segment {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, self.x1);
        write_le_bytes!(w, self.y1);
        write_le_bytes!(w, self.x2);
        write_le_bytes!(w, self.y2);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PolySegment {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub segments: Vec<Segment>,
}

impl ToLeBytes for PolySegment {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.segments.len();
        let request_length = 3 + 2 * n;

        write_le_bytes!(w, opcodes::POLY_SEGMENT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for segment in &self.segments {
            write_le_bytes!(w, segment.x1);
            write_le_bytes!(w, segment.y1);
            write_le_bytes!(w, segment.x2);
            write_le_bytes!(w, segment.y2);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolySegment);

/*
PolyRectangle
     1     67                              opcode
     1                                     unused
     2     3+2n                            request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     8n     LISTofRECTANGLE                rectangles
*/

#[derive(Debug, Clone)]
pub struct PolyRectangle {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub rectangles: Vec<Rectangle>,
}

impl ToLeBytes for PolyRectangle {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.rectangles.len();
        let request_length = 3 + 2 * n;

        write_le_bytes!(w, opcodes::POLY_RECTANGLE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for rectangle in &self.rectangles {
            write_le_bytes!(w, rectangle);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyRectangle);

/*
PolyArc
     1     68                              opcode
     1                                     unused
     2     3+3n                            request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     12n     LISTofARC                     arcs
*/

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Arc {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub angle1: i16,
    pub angle2: i16,
}

impl Arc {
    fn to_le_bytes(self) -> [u8; 12] {
        unsafe { mem::transmute(self) }
    }
}

#[derive(Debug, Clone)]
pub struct PolyArc {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub arcs: Vec<Arc>,
}

impl ToLeBytes for PolyArc {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.arcs.len();
        let request_length = 3 + 3 * n;

        write_le_bytes!(w, opcodes::POLY_ARC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for arc in &self.arcs {
            write_le_bytes!(w, arc);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyArc);

/*
FillPoly
     1     69                              opcode
     1                                     unused
     2     4+n                             request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     1                                     shape
          0     Complex
          1     Nonconvex
          2     Convex
     1                                     coordinate-mode
          0     Origin
          1     Previous
     2                                     unused
     4n     LISTofPOINT                    points
*/

impl_enum! {
    #[repr(u8)]
    enum FillPolyShape {
        Complex = 0,
        Nonconvex = 1,
        Convex = 2,
    }
}

#[derive(Debug, Clone)]
pub struct FillPoly {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub shape: FillPolyShape,
    pub coordinate_mode: CoordinateMode,
    pub points: Vec<Point>,
}

impl ToLeBytes for FillPoly {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.points.len();
        let request_length = 4 + n;

        write_le_bytes!(w, opcodes::FILL_POLY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.shape as u8);
        write_le_bytes!(w, self.coordinate_mode as u8);
        write_le_bytes!(w, 0u16); // unused

        for point in &self.points {
            write_le_bytes!(w, point);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(FillPoly);

/*
PolyFillRectangle
     1     70                              opcode
     1                                     unused
     2     3+2n                            request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     8n     LISTofRECTANGLE                rectangles
*/

#[derive(Debug, Clone)]
pub struct PolyFillRectangle {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub rectangles: Vec<Rectangle>,
}

impl ToLeBytes for PolyFillRectangle {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n: u16 = self.rectangles.len() as u16;
        let request_length = 3 + 2 * n;

        write_le_bytes!(w, opcodes::POLY_FILL_RECTANGLE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length);
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, self.gc);
        for rectangle in &self.rectangles {
            write_le_bytes!(w, rectangle);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyFillRectangle);

/*
PolyFillArc
     1     71                              opcode
     1                                     unused
     2     3+3n                            request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     12n     LISTofARC                     arcs
*/

#[derive(Debug, Clone)]
pub struct PolyFillArc {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub arcs: Vec<Arc>,
}

impl ToLeBytes for PolyFillArc {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.arcs.len();
        let request_length = 3 + 3 * n;

        write_le_bytes!(w, opcodes::POLY_FILL_ARC);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);

        for arc in &self.arcs {
            write_le_bytes!(w, arc);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyFillArc);

/*
PutImage
     1     72                              opcode
     1                                     format
          0     Bitmap
          1     XYPixmap
          2     ZPixmap
     2     6+(n+p)/4                       request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     2     CARD16                          width
     2     CARD16                          height
     2     INT16                           dst-x
     2     INT16                           dst-y
     1     CARD8                           left-pad
     1     CARD8                           depth
     2                                     unused
     n     LISTofBYTE                      data
     p                                     unused, p=pad(n)
*/

impl_enum! {
    #[repr(u8)]
    /// Image format
    enum PutImageFormat {
        Bitmap = 0,
        XYPixmap = 1,
        /// Probably the one you want. Pixels stores as scanlines
        ZPixmap = 2,
    }
}

#[derive(Debug, Clone)]
pub struct PutImageOwned {
    pub format: PutImageFormat,
    pub drawable: Drawable,
    pub gc: GContextId,
    pub width: u16,
    pub height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub left_pad: u8,
    pub depth: u8,
    pub data: Vec<u8>,
}

impl PutImageOwned {
    pub fn to_shared(&self) -> PutImage<'_> {
        PutImage {
            format: self.format,
            drawable: self.drawable,
            gc: self.gc,
            width: self.width,
            height: self.height,
            dst_x: self.dst_x,
            dst_y: self.dst_y,
            left_pad: self.left_pad,
            depth: self.depth,
            data: &self.data,
        }
    }
}

impl ToLeBytes for PutImageOwned {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        self.to_shared().to_le_bytes(w)
    }
}

impl_xrequest_without_response!(PutImageOwned);

#[derive(Debug, Clone)]
pub struct PutImage<'data> {
    pub format: PutImageFormat,
    pub drawable: Drawable,
    pub gc: GContextId,
    pub width: u16,
    pub height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub left_pad: u8,

    /// If format is [`PutImageFormat::ZPixmap`], `depth` must equal to depth of `drawable`
    pub depth: u8,
    pub data: &'data [u8],
}

impl<'data> ToLeBytes for PutImage<'data> {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.data.len();
        let p = pad(n);
        let len = (6 + ((n + p) / 4)) as u16;

        write_le_bytes!(w, &opcodes::PUT_IMAGE);
        write_le_bytes!(w, self.format as u8);
        write_le_bytes!(w, len);
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);
        write_le_bytes!(w, self.left_pad);
        write_le_bytes!(w, self.depth);
        w.write_all(&[0u8; 2])?; // unused
        w.write_all(self.data)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl<'data> XRequestBase for PutImage<'data> {
    type Reply = NoReply;

    fn reply_type() -> Option<ReplyType> {
        None
    }
}

impl<'data> XRequest for PutImage<'data> {}

/*
GetImage
     1     73                              opcode
     1                                     format
          1     XYPixmap
          2     ZPixmap
     2     5                               request length
     4     DRAWABLE                        drawable
     2     INT16                           x
     2     INT16                           y
     2     CARD16                          width
     2     CARD16                          height
     4     CARD32                          plane-mask
*/

impl_enum! {
    #[repr(u8)]
    enum GetImageImageFormat {
        XYPixmap = 1,
        ZPixmap = 2,
    }
}

#[derive(Debug, Clone)]
pub struct GetImage {
    pub format: GetImageImageFormat,
    pub drawable: Drawable,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub plane_mask: u32,
}

impl ToLeBytes for GetImage {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_IMAGE);
        write_le_bytes!(w, self.format as u8);
        write_le_bytes!(w, 5u16); // request length
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.plane_mask);

        Ok(())
    }
}

impl_xrequest_with_response!(GetImage);

/*
PolyText8
     1     74                              opcode
     1                                     unused
     2     4+(n+p)/4                       request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     2     INT16                           x
     2     INT16                           y
     n     LISTofTEXTITEM8                 items
     p                                     unused, p=pad(n)  (p is always 0
                                           or 1)

  TEXTITEM8
     1     m                               length of string (cannot be 255)
     1     INT8                            delta
     m     STRING8                         string
  or
     1     255                             font-shift indicator
     1                                     font byte 3 (most-significant)
     1                                     font byte 2
     1                                     font byte 1
     1                                     font byte 0 (least-significant)
*/

#[derive(Debug, Clone)]
pub enum TextItem8 {
    Text { delta: i8, string: Vec<u8> },
    FontShift { font_bytes: [u8; 4] },
}

impl ToLeBytes for TextItem8 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        match self {
            TextItem8::Text { delta, string } => {
                let len = string.len();
                assert!(len < 255);

                write_le_bytes!(w, len as u8);
                write_le_bytes!(w, *delta);
                w.write_all(string)?;
            }
            TextItem8::FontShift { font_bytes } => {
                write_le_bytes!(w, 255u8);
                w.write_all(&[font_bytes[3], font_bytes[2], font_bytes[1], font_bytes[0]])?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PolyText8 {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub x: i16,
    pub y: i16,
    pub items: Vec<TextItem8>,
}

impl ToLeBytes for PolyText8 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.items.len();
        let p = pad(n);
        let request_length = 4 + (n + p) / 4;

        write_le_bytes!(w, opcodes::POLY_TEXT8);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);

        for item in &self.items {
            item.to_le_bytes(w)?;
        }

        debug_assert!([0, 1].contains(&p));
        if p == 1 {
            write_le_bytes!(w, 0u8); // pad
        }

        Ok(())
    }
}

impl_xrequest_without_response!(PolyText8);

/*
PolyText16
     1     75                              opcode
     1                                     unused
     2     4+(n+p)/4                       request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     2     INT16                           x
     2     INT16                           y
     n     LISTofTEXTITEM16                items
     p                                     unused, p=pad(n)  (p must be 0 or
                                           1)

  TEXTITEM16
     1     m                               number of CHAR2Bs in string
                                           (cannot be 255)
     1     INT8                            delta
     2m     STRING16                       string
  or
     1     255                             font-shift indicator
     1                                     font byte 3 (most-significant)
     1                                     font byte 2
     1                                     font byte 1
     1                                     font byte 0 (least-significant)
*/

#[derive(Debug, Clone)]
pub struct PolyText16 {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub x: i16,
    pub y: i16,
    pub items: Vec<TextItem16>,
}

#[derive(Debug, Clone)]
pub enum TextItem16 {
    Text { delta: i8, string: Vec<u16> },
    FontShift { font_bytes: [u8; 4] },
}

impl ToLeBytes for TextItem16 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        match self {
            TextItem16::Text { delta, string } => {
                let len = string.len();
                assert!(len < 255);

                write_le_bytes!(w, len as u8);
                write_le_bytes!(w, *delta);
                for c in string {
                    write_le_bytes!(w, *c);
                }
            }
            TextItem16::FontShift { font_bytes } => {
                write_le_bytes!(w, 255u8);
                w.write_all(&[font_bytes[3], font_bytes[2], font_bytes[1], font_bytes[0]])?;
            }
        }

        Ok(())
    }
}

impl ToLeBytes for PolyText16 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.items.len();
        let p = pad(n);
        let request_length = 4 + (n + p) / 4;

        write_le_bytes!(w, opcodes::POLY_TEXT16);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);

        for item in &self.items {
            item.to_le_bytes(w)?;
        }

        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(PolyText16);

/*
ImageText8
     1     76                              opcode
     1     n                               length of string
     2     4+(n+p)/4                       request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     2     INT16                           x
     2     INT16                           y
     n     STRING8                         string
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct ImageText8 {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub x: i16,
    pub y: i16,
    pub string: Vec<u8>,
}

impl ToLeBytes for ImageText8 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.string.len();
        let p = pad(n);
        let request_length = 4 + (n + p) / 4;

        write_le_bytes!(w, opcodes::IMAGE_TEXT8);
        write_le_bytes!(w, n as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        w.write_all(&self.string)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(ImageText8);

/*
ImageText16
     1     77                              opcode
     1     n                               number of CHAR2Bs in string
     2     4+(2n+p)/4                      request length
     4     DRAWABLE                        drawable
     4     GCONTEXT                        gc
     2     INT16                           x
     2     INT16                           y
     2n     STRING16                       string
     p                                     unused, p=pad(2n)
*/

#[derive(Debug, Clone)]
pub struct ImageText16 {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub x: i16,
    pub y: i16,
    pub string: Vec<u16>,
}

impl ToLeBytes for ImageText16 {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.string.len();
        let p = pad(2 * n);
        let request_length = 4 + (2 * n + p) / 4;

        write_le_bytes!(w, opcodes::IMAGE_TEXT16);
        write_le_bytes!(w, n as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);

        for c in &self.string {
            write_le_bytes!(w, *c);
        }

        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(ImageText16);

/*
CreateColormap
     1     78                              opcode
     1                                     alloc
          0     None
          1     All
     2     4                               request length
     4     COLORMAP                        mid
     4     WINDOW                          window
     4     VISUALID                        visual
*/

impl_enum! {
    #[repr(u8)]
    enum CreateColormapAlloc {
        None = 0,
        All = 1,
    }
}

#[derive(Debug, Clone)]
pub struct CreateColormap {
    pub alloc: CreateColormapAlloc,
    pub mid: ColormapId,
    pub window: WindowId,
    pub visual: VisualId,
}

impl ToLeBytes for CreateColormap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_COLORMAP);
        write_le_bytes!(w, self.alloc as u8);
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.mid);
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.visual);

        Ok(())
    }
}

impl_xrequest_without_response!(CreateColormap);

/*
FreeColormap
     1     79                              opcode
     1                                     unused
     2     2                               request length
     4     COLORMAP                        cmap
*/

#[derive(Debug, Clone)]
pub struct FreeColormap {
    pub cmap: ColormapId,
}

impl ToLeBytes for FreeColormap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::FREE_COLORMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.cmap);

        Ok(())
    }
}

impl_xrequest_without_response!(FreeColormap);

/*
CopyColormapAndFree
     1     80                              opcode
     1                                     unused
     2     3                               request length
     4     COLORMAP                        mid
     4     COLORMAP                        src-cmap
*/

#[derive(Debug, Clone)]
pub struct CopyColormapAndFree {
    pub mid: ColormapId,
    pub src_cmap: ColormapId,
}

impl ToLeBytes for CopyColormapAndFree {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::COPY_COLORMAP_AND_FREE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.mid);
        write_le_bytes!(w, self.src_cmap);

        Ok(())
    }
}

impl_xrequest_without_response!(CopyColormapAndFree);

/*
InstallColormap
     1     81                              opcode
     1                                     unused
     2     2                               request length
     4     COLORMAP                        cmap
*/

#[derive(Debug, Clone)]
pub struct InstallColormap {
    pub cmap: ColormapId,
}

impl ToLeBytes for InstallColormap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::INSTALL_COLORMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.cmap);

        Ok(())
    }
}

impl_xrequest_without_response!(InstallColormap);

/*
UninstallColormap
     1     82                              opcode
     1                                     unused
     2     2                               request length
     4     COLORMAP                        cmap
*/

#[derive(Debug, Clone)]
pub struct UninstallColormap {
    pub cmap: ColormapId,
}

impl ToLeBytes for UninstallColormap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNINSTALL_COLORMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.cmap);

        Ok(())
    }
}

impl_xrequest_without_response!(UninstallColormap);

/*
ListInstalledColormaps
     1     83                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone)]
pub struct ListInstalledColormaps {
    pub window: WindowId,
}

impl ToLeBytes for ListInstalledColormaps {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::LIST_INSTALLED_COLORMAPS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(ListInstalledColormaps);

/*
AllocColor
     1     84                              opcode
     1                                     unused
     2     4                               request length
     4     COLORMAP                        cmap
     2     CARD16                          red
     2     CARD16                          green
     2     CARD16                          blue
     2                                     unused
*/

#[derive(Debug, Clone)]
pub struct AllocColor {
    pub cmap: ColormapId,
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl ToLeBytes for AllocColor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::ALLOC_COLOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, self.red);
        write_le_bytes!(w, self.green);
        write_le_bytes!(w, self.blue);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_with_response!(AllocColor);

/*
AllocNamedColor
     1     85                              opcode
     1                                     unused
     2     3+(n+p)/4                       request length
     4     COLORMAP                        cmap
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct AllocNamedColor {
    pub cmap: ColormapId,
    pub name: Vec<u8>,
}

impl ToLeBytes for AllocNamedColor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_length = 3 + (n + p) / 4;

        write_le_bytes!(w, opcodes::ALLOC_NAMED_COLOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_with_response!(AllocNamedColor);

/*
AllocColorCells
     1     86                              opcode
     1     BOOL                            contiguous
     2     3                               request length
     4     COLORMAP                        cmap
     2     CARD16                          colors
     2     CARD16                          planes
*/

#[derive(Debug, Clone)]
pub struct AllocColorCells {
    pub contiguous: bool,
    pub cmap: ColormapId,
    pub colors: u16,
    pub planes: u16,
}

impl ToLeBytes for AllocColorCells {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::ALLOC_COLOR_CELLS);
        write_le_bytes!(w, self.contiguous as u8);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, self.colors);
        write_le_bytes!(w, self.planes);

        Ok(())
    }
}

impl_xrequest_with_response!(AllocColorCells);

/*
AllocColorPlanes
     1     87                              opcode
     1     BOOL                            contiguous
     2     4                               request length
     4     COLORMAP                        cmap
     2     CARD16                          colors
     2     CARD16                          reds
     2     CARD16                          greens
     2     CARD16                          blues
*/

#[derive(Debug, Clone)]
pub struct AllocColorPlanes {
    pub contiguous: bool,
    pub cmap: ColormapId,
    pub colors: u16,
    pub reds: u16,
    pub greens: u16,
    pub blues: u16,
}

impl ToLeBytes for AllocColorPlanes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::ALLOC_COLOR_PLANES);
        write_le_bytes!(w, self.contiguous as u8);
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, self.colors);
        write_le_bytes!(w, self.reds);
        write_le_bytes!(w, self.greens);
        write_le_bytes!(w, self.blues);

        Ok(())
    }
}

impl_xrequest_with_response!(AllocColorPlanes);

/*
FreeColors
     1     88                              opcode
     1                                     unused
     2     3+n                             request length
     4     COLORMAP                        cmap
     4     CARD32                          plane-mask
     4n     LISTofCARD32                   pixels
*/

#[derive(Debug, Clone)]
pub struct FreeColors {
    pub cmap: ColormapId,
    pub plane_mask: u32,
    pub pixels: Vec<u32>,
}

impl ToLeBytes for FreeColors {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.pixels.len();
        let request_length = 3 + n;

        write_le_bytes!(w, opcodes::FREE_COLORS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, self.plane_mask);
        for pixel in &self.pixels {
            write_le_bytes!(w, *pixel);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(FreeColors);

/*
StoreColors
     1     89                              opcode
     1                                     unused
     2     2+3n                            request length
     4     COLORMAP                        cmap
     12n     LISTofCOLORITEM               items

  COLORITEM
     4     CARD32                          pixel
     2     CARD16                          red
     2     CARD16                          green
     2     CARD16                          blue
     1                                     do-red, do-green, do-blue
          #x01     do-red (1 is True, 0 is False)
          #x02     do-green (1 is True, 0 is False)
          #x04     do-blue (1 is True, 0 is False)
          #xF8     unused
     1                                     unused
*/

#[derive(Debug, Clone, Copy)]
pub struct ColorItem {
    pub pixel: u32,
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub do_red: bool,
    pub do_green: bool,
    pub do_blue: bool,
}

impl ToLeBytes for ColorItem {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, self.pixel);
        write_le_bytes!(w, self.red);
        write_le_bytes!(w, self.green);
        write_le_bytes!(w, self.blue);

        let flags = if self.do_red { 0x01 } else { 0x00 }
            | if self.do_green { 0x02 } else { 0x00 }
            | if self.do_blue { 0x04 } else { 0x00 };
        write_le_bytes!(w, flags as u8);
        write_le_bytes!(w, 0u8); // unused

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StoreColors {
    pub cmap: ColormapId,
    pub items: Vec<ColorItem>,
}

impl ToLeBytes for StoreColors {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.items.len();
        let request_length = 2 + 3 * n;

        write_le_bytes!(w, opcodes::STORE_COLORS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);

        for item in &self.items {
            item.to_le_bytes(w)?;
        }

        Ok(())
    }
}

impl_xrequest_without_response!(StoreColors);

/*
StoreNamedColor
     1     90                              opcode
     1                                     do-red, do-green, do-blue
          #x01     do-red (1 is True, 0 is False)
          #x02     do-green (1 is True, 0 is False)
          #x04     do-blue (1 is True, 0 is False)
          #xF8     unused
     2     4+(n+p)/4                       request length
     4     COLORMAP                        cmap
     4     CARD32                          pixel
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct StoreNamedColor {
    pub cmap: ColormapId,
    pub do_red: bool,
    pub do_green: bool,
    pub do_blue: bool,
    pub pixel: u32,
    pub name: Vec<u8>,
}

impl ToLeBytes for StoreNamedColor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_length = 4 + (n + p) / 4;

        let flags = if self.do_red { 0x01 } else { 0x00 }
            | if self.do_green { 0x02 } else { 0x00 }
            | if self.do_blue { 0x04 } else { 0x00 };

        write_le_bytes!(w, opcodes::STORE_NAMED_COLOR);
        write_le_bytes!(w, flags as u8);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, self.pixel);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(StoreNamedColor);

/*
QueryColors
     1     91                              opcode
     1                                     unused
     2     2+n                             request length
     4     COLORMAP                        cmap
     4n     LISTofCARD32                   pixels
*/

#[derive(Debug, Clone)]
pub struct QueryColors {
    pub cmap: ColormapId,
    pub pixels: Vec<u32>,
}

impl ToLeBytes for QueryColors {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.pixels.len();
        let request_length = 2 + n;

        write_le_bytes!(w, opcodes::QUERY_COLORS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);

        for pixel in &self.pixels {
            write_le_bytes!(w, *pixel);
        }

        Ok(())
    }
}

impl_xrequest_with_response!(QueryColors);

/*
LookupColor
     1     92                              opcode
     1                                     unused
     2     3+(n+p)/4                       request length
     4     COLORMAP                        cmap
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct LookupColor {
    pub cmap: ColormapId,
    pub name: Vec<u8>,
}

impl ToLeBytes for LookupColor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_length = 3 + (n + p) / 4;

        write_le_bytes!(w, opcodes::LOOKUP_COLOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.cmap);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_with_response!(LookupColor);

/*
CreateCursor
     1     93                              opcode
     1                                     unused
     2     8                               request length
     4     CURSOR                          cid
     4     PIXMAP                          source
     4     PIXMAP                          mask
          0     None
     2     CARD16                          fore-red
     2     CARD16                          fore-green
     2     CARD16                          fore-blue
     2     CARD16                          back-red
     2     CARD16                          back-green
     2     CARD16                          back-blue
     2     CARD16                          x
     2     CARD16                          y
*/

#[derive(Debug, Clone)]
pub struct CreateCursor {
    pub cid: CursorId,
    pub source: PixmapId,
    pub mask: OrNone<PixmapId>,
    pub fore_red: u16,
    pub fore_green: u16,
    pub fore_blue: u16,
    pub back_red: u16,
    pub back_green: u16,
    pub back_blue: u16,
    pub x: u16,
    pub y: u16,
}

impl ToLeBytes for CreateCursor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_CURSOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 8u16); // request length
        write_le_bytes!(w, self.cid);
        write_le_bytes!(w, self.source);
        write_le_bytes!(w, self.mask.0);
        write_le_bytes!(w, 0u32); // None
        write_le_bytes!(w, self.fore_red);
        write_le_bytes!(w, self.fore_green);
        write_le_bytes!(w, self.fore_blue);
        write_le_bytes!(w, self.back_red);
        write_le_bytes!(w, self.back_green);
        write_le_bytes!(w, self.back_blue);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);

        Ok(())
    }
}

impl_xrequest_without_response!(CreateCursor);

/*
CreateGlyphCursor
     1     94                              opcode
     1                                     unused
     2     8                               request length
     4     CURSOR                          cid
     4     FONT                            source-font
     4     FONT                            mask-font
          0     None
     2     CARD16                          source-char
     2     CARD16                          mask-char
     2     CARD16                          fore-red
     2     CARD16                          fore-green
     2     CARD16                          fore-blue
     2     CARD16                          back-red
     2     CARD16                          back-green
     2     CARD16                          back-blue
*/

#[derive(Debug, Clone)]
pub struct CreateGlyphCursor {
    pub cid: CursorId,
    pub source_font: FontId,
    pub mask_font: OrNone<FontId>,
    pub source_char: u16,
    pub mask_char: u16,
    pub fore_red: u16,
    pub fore_green: u16,
    pub fore_blue: u16,
    pub back_red: u16,
    pub back_green: u16,
    pub back_blue: u16,
}

impl ToLeBytes for CreateGlyphCursor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_GLYPH_CURSOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 8u16); // request length
        write_le_bytes!(w, self.cid);
        write_le_bytes!(w, self.source_font);
        write_le_bytes!(w, self.mask_font.0);
        write_le_bytes!(w, 0u32); // None
        write_le_bytes!(w, self.source_char);
        write_le_bytes!(w, self.mask_char);
        write_le_bytes!(w, self.fore_red);
        write_le_bytes!(w, self.fore_green);
        write_le_bytes!(w, self.fore_blue);
        write_le_bytes!(w, self.back_red);
        write_le_bytes!(w, self.back_green);
        write_le_bytes!(w, self.back_blue);

        Ok(())
    }
}

impl_xrequest_without_response!(CreateGlyphCursor);

/*
FreeCursor
     1     95                              opcode
     1                                     unused
     2     2                               request length
     4     CURSOR                          cursor
*/

#[derive(Debug, Clone)]
pub struct FreeCursor {
    pub cursor: CursorId,
}

impl ToLeBytes for FreeCursor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::FREE_CURSOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.cursor);

        Ok(())
    }
}

impl_xrequest_without_response!(FreeCursor);

/*
RecolorCursor
     1     96                              opcode
     1                                     unused
     2     5                               request length
     4     CURSOR                          cursor
     2     CARD16                          fore-red
     2     CARD16                          fore-green
     2     CARD16                          fore-blue
     2     CARD16                          back-red
     2     CARD16                          back-green
     2     CARD16                          back-blue
*/

#[derive(Debug, Clone)]
pub struct RecolorCursor {
    pub cursor: CursorId,
    pub fore_red: u16,
    pub fore_green: u16,
    pub fore_blue: u16,
    pub back_red: u16,
    pub back_green: u16,
    pub back_blue: u16,
}

impl ToLeBytes for RecolorCursor {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::RECOLOR_CURSOR);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 5u16); // request length
        write_le_bytes!(w, self.cursor);
        write_le_bytes!(w, self.fore_red);
        write_le_bytes!(w, self.fore_green);
        write_le_bytes!(w, self.fore_blue);
        write_le_bytes!(w, self.back_red);
        write_le_bytes!(w, self.back_green);
        write_le_bytes!(w, self.back_blue);

        Ok(())
    }
}

impl_xrequest_without_response!(RecolorCursor);

/*
QueryBestSize
     1     97                              opcode
     1                                     class
          0     Cursor
          1     Tile
          2     Stipple
     2     3                               request length
     4     DRAWABLE                        drawable
     2     CARD16                          width
     2     CARD16                          height
*/

impl_enum! {
    #[repr(u8)]
    enum QueryBestSizeClass {
        Cursor = 0,
        Tile = 1,
        Stipple = 2,
    }
}

#[derive(Debug, Clone)]
pub struct QueryBestSize {
    pub class: QueryBestSizeClass,
    pub drawable: Drawable,
    pub width: u16,
    pub height: u16,
}

impl ToLeBytes for QueryBestSize {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_BEST_SIZE);
        write_le_bytes!(w, self.class);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);

        Ok(())
    }
}

impl_xrequest_with_response!(QueryBestSize);

/*
QueryExtension
     1     98                              opcode
     1                                     unused
     2     2+(n+p)/4                       request length
     2     n                               length of name
     2                                     unused
     n     STRING8                         name
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct QueryExtension {
    pub name: Vec<u8>,
}

impl ToLeBytes for QueryExtension {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_length = 2 + (n + p) / 4;

        write_le_bytes!(w, opcodes::QUERY_EXTENSION);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_with_response!(QueryExtension);

/*
ListExtensions
     1     99                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct ListExtensions;

impl ToLeBytes for ListExtensions {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::LIST_EXTENSIONS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(ListExtensions);

/*
ChangeKeyboardMapping
     1     100                             opcode
     1     n                               keycode-count
     2     2+nm                            request length
     1     KEYCODE                         first-keycode
     1     m                               keysyms-per-keycode
     2                                     unused
     4nm     LISTofKEYSYM                  keysyms
*/

#[derive(Debug, Clone)]
pub struct ChangeKeyboardMapping {
    pub keycode_count: u8,
    pub first_keycode: KeyCode,
    pub keysyms_per_keycode: u8,
    pub keysyms: Vec<KeySym>,
}

impl ToLeBytes for ChangeKeyboardMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let nm = self.keycode_count as usize * self.keysyms_per_keycode as usize;
        let request_length = 2 + nm / 4;

        write_le_bytes!(w, opcodes::CHANGE_KEYBOARD_MAPPING);
        write_le_bytes!(w, self.keycode_count);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.first_keycode);
        write_le_bytes!(w, self.keysyms_per_keycode);
        write_le_bytes!(w, 0u16); // unused
        for keysym in &self.keysyms {
            write_le_bytes!(w, *keysym);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeKeyboardMapping);

/*
GetKeyboardMapping
     1     101                             opcode
     1                                     unused
     2     2                               request length
     1     KEYCODE                         first-keycode
     1     m                               count
     2                                     unused
*/

#[derive(Debug, Clone)]
pub struct GetKeyboardMapping {
    pub first_keycode: KeyCode,
    pub count: u8,
}

impl ToLeBytes for GetKeyboardMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_KEYBOARD_MAPPING);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.first_keycode);
        write_le_bytes!(w, self.count);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_with_response!(GetKeyboardMapping);

/*
ChangeKeyboardControl
     1     102                             opcode
     1                                     unused
     2     2+n                             request length
     4     BITMASK                         value-mask (has n bits set to 1)
          #x0001     key-click-percent
          #x0002     bell-percent
          #x0004     bell-pitch
          #x0008     bell-duration
          #x0010     led
          #x0020     led-mode
          #x0040     key
          #x0080     auto-repeat-mode
     4n     LISTofVALUE                    value-list

  VALUEs
     1     INT8                            key-click-percent
     1     INT8                            bell-percent
     2     INT16                           bell-pitch
     2     INT16                           bell-duration
     1     CARD8                           led
     1                                     led-mode
          0     Off
          1     On
     1     KEYCODE                         key
     1                                     auto-repeat-mode
          0     Off
          1     On
          2     Default
*/

impl_raw_fields! {
    ChangeKeyboardControlValues[8] {
        set_key_click_percent: i8,
        set_bell_percent: i8,
        set_bell_pitch: i16,
        set_bell_duration: i16,
        set_led: u8,
        set_led_mode: bool,
        set_key: KeyCode,
        set_auto_repeat_m: u8,
    }
}

#[derive(Debug, Clone)]
pub struct ChangeKeyboardControl {
    pub values: ChangeKeyboardControlValues,
}

impl ToLeBytes for ChangeKeyboardControl {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.values.values.mask_and_count();
        let request_length = 2 + n;

        write_le_bytes!(w, opcodes::CHANGE_KEYBOARD_CONTROL);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length);
        write_le_bytes!(w, bitmask);
        self.values.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeKeyboardControl);

/*
GetKeyboardControl
     1     103                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetKeyboardControl;

impl ToLeBytes for GetKeyboardControl {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_KEYBOARD_CONTROL);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(GetKeyboardControl);

/*
Bell
     1     104                             opcode
     1     INT8                            percent
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct Bell {
    pub percent: i8,
}

impl ToLeBytes for Bell {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::BELL);
        write_le_bytes!(w, self.percent);
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_without_response!(Bell);

/*
ChangePointerControl
     1     105                             opcode
     1                                     unused
     2     3                               request length
     2     INT16                           acceleration-numerator
     2     INT16                           acceleration-denominator
     2     INT16                           threshold
     1     BOOL                            do-acceleration
     1     BOOL                            do-threshold
*/

#[derive(Debug, Clone)]
pub struct ChangePointerControl {
    pub acceleration_numerator: i16,
    pub acceleration_denominator: i16,
    pub threshold: i16,
    pub do_acceleration: bool,
    pub do_threshold: bool,
}

impl ToLeBytes for ChangePointerControl {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CHANGE_POINTER_CONTROL);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.acceleration_numerator);
        write_le_bytes!(w, self.acceleration_denominator);
        write_le_bytes!(w, self.threshold);
        write_le_bytes!(w, self.do_acceleration as u8);
        write_le_bytes!(w, self.do_threshold as u8);

        Ok(())
    }
}

impl_xrequest_without_response!(ChangePointerControl);

/*
GetPointerControl
     1     106                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetPointerControl;

impl ToLeBytes for GetPointerControl {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_POINTER_CONTROL);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(GetPointerControl);

/*
SetScreenSaver
     1     107                             opcode
     1                                     unused
     2     3                               request length
     2     INT16                           timeout
     2     INT16                           interval
     1                                     prefer-blanking
          0     No
          1     Yes
          2     Default
     1                                     allow-exposures
          0     No
          1     Yes
          2     Default
     2                                     unused
*/

impl_enum! {
    #[repr(u8)]
    enum NoYesDefault {
        No = 0,
        Yes = 1,
        Default = 2,
    }
}

#[derive(Debug, Clone)]
pub struct SetScreenSaver {
    pub timeout: i16,
    pub interval: i16,
    pub prefer_blanking: NoYesDefault,
    pub allow_exposures: NoYesDefault,
}

impl ToLeBytes for SetScreenSaver {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_SCREEN_SAVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.timeout);
        write_le_bytes!(w, self.interval);
        write_le_bytes!(w, self.prefer_blanking);
        write_le_bytes!(w, self.allow_exposures);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl_xrequest_without_response!(SetScreenSaver);

/*
GetScreenSaver
     1     108                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetScreenSaver;

impl ToLeBytes for GetScreenSaver {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SCREEN_SAVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(GetScreenSaver);

/*
ChangeHosts
     1     109                             opcode
     1                                     mode
          0     Insert
          1     Delete
     2     2+(n+p)/4                       request length
     1                                     family
          0     Internet
          1     DECnet
          2     Chaos
     1                                     unused
     2     n                               length of address
     n     LISTofCARD8                     address
     p                                     unused, p=pad(n)
*/

impl_enum! {
    #[repr(u8)]
    enum ChangeHostsMode {
        Insert = 0,
        Delete = 1,
    }
}

impl_enum! {
    #[repr(u8)]
    enum ChangeHostsFamily {
        Internet = 0,
        DECnet = 1,
        Chaos = 2,
    }
}

#[derive(Debug, Clone)]
pub struct ChangeHosts {
    pub mode: ChangeHostsMode,
    pub family: ChangeHostsFamily,
    pub address: Vec<u8>,
}

impl ToLeBytes for ChangeHosts {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.address.len();
        let p = pad(n);
        let request_length = 2 + (n + p) / 4;

        write_le_bytes!(w, opcodes::CHANGE_HOSTS);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.family);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, n as u16);
        w.write_all(&self.address)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_without_response!(ChangeHosts);

/*
ListHosts
     1     110                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct ListHosts;

impl ToLeBytes for ListHosts {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::LIST_HOSTS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(ListHosts);

/*
SetAccessControl
     1     111                             opcode
     1                                     mode
          0     Disable
          1     Enable
     2     1                               request length
*/

impl_enum! {
    #[repr(u8)]
    enum SetAccessControlMode {
        Disable = 0,
        Enable = 1,
    }
}

#[derive(Debug, Clone)]
pub struct SetAccessControl {
    pub mode: SetAccessControlMode,
}

impl ToLeBytes for SetAccessControl {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_ACCESS_CONTROL);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_without_response!(SetAccessControl);

/*
SetCloseDownMode
     1     112                             opcode
     1                                     mode
          0     Destroy
          1     RetainPermanent
          2     RetainTemporary
     2     1                               request length
*/

impl_enum! {
    #[repr(u8)]
    enum SetCloseDownModeMode {
        Destroy = 0,
        RetainPermanent = 1,
        RetainTemporary = 2,
    }
}

#[derive(Debug, Clone)]
pub struct SetCloseDownMode {
    pub mode: SetCloseDownModeMode,
}

impl ToLeBytes for SetCloseDownMode {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_CLOSE_DOWN_MODE);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_without_response!(SetCloseDownMode);

/*
KillClient
     1     113                             opcode
     1                                     unused
     2     2                               request length
     4     CARD32                          resource
          0     AllTemporary
*/

#[derive(Debug, Clone)]
pub struct KillClient {
    pub resource: u32,
}

impl ToLeBytes for KillClient {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::KILL_CLIENT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.resource);

        Ok(())
    }
}

impl_xrequest_without_response!(KillClient);

/*
RotateProperties
     1     114                             opcode
     1                                     unused
     2     3+n                             request length
     4     WINDOW                          window
     2     n                               number of properties
     2     INT16                           delta
     4n    LISTofATOM                      properties
*/

#[derive(Debug, Clone)]
pub struct RotateProperties {
    pub window: WindowId,
    pub number_of_properties: u16,
    pub delta: i16,
    pub properties: Vec<AtomId>,
}

impl ToLeBytes for RotateProperties {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.properties.len();
        let request_length = 3 + n;

        write_le_bytes!(w, opcodes::ROTATE_PROPERTIES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, n as u16);
        write_le_bytes!(w, self.delta);
        for prop in &self.properties {
            write_le_bytes!(w, prop);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(RotateProperties);

/*
ForceScreenSaver
     1     115                             opcode
     1                                     mode
          0     Reset
          1     Activate
     2     1                               request length
*/

impl_enum! {
    #[repr(u8)]
    enum ForceScreenSaverMode {
        Reset = 0,
        Activate = 1,
    }
}

#[derive(Debug, Clone)]
pub struct ForceScreenSaver {
    pub mode: ForceScreenSaverMode,
}

impl ToLeBytes for ForceScreenSaver {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::FORCE_SCREEN_SAVER);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_without_response!(ForceScreenSaver);

/*
SetPointerMapping
     1     116                             opcode
     1     n                               length of map
     2     1+(n+p)/4                       request length
     n     LISTofCARD8                     map
     p                                     unused, p=pad(n)
*/

#[derive(Debug, Clone)]
pub struct SetPointerMapping {
    pub map: Vec<u8>,
}

impl ToLeBytes for SetPointerMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.map.len();
        let p = pad(n);
        let request_length = 1 + (n + p) / 4;

        write_le_bytes!(w, opcodes::SET_POINTER_MAPPING);
        write_le_bytes!(w, n as u8);
        write_le_bytes!(w, request_length as u16);
        w.write_all(&self.map)?;
        w.write_all(&vec![0u8; p])?; // pad

        Ok(())
    }
}

impl_xrequest_with_response!(SetPointerMapping);

/*
GetPointerMapping
     1     117                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetPointerMapping;

impl ToLeBytes for GetPointerMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_POINTER_MAPPING);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(GetPointerMapping);

/*
SetModifierMapping
     1     118                             opcode
     1     n                               keycodes-per-modifier
     2     1+2n                            request length
     8n    LISTofKEYCODE                   keycodes
*/

#[derive(Debug, Clone)]
pub struct SetModifierMapping {
    pub keycodes: Vec<KeyCode>,
}

impl ToLeBytes for SetModifierMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.keycodes.len();
        let request_length = 1 + 2 * n;

        write_le_bytes!(w, opcodes::SET_MODIFIER_MAPPING);
        write_le_bytes!(w, n as u8);
        write_le_bytes!(w, request_length as u16);
        w.write_all(
            &self
                .keycodes
                .iter()
                .map(|key_code| key_code.0)
                .collect::<Vec<u8>>(),
        )?;

        Ok(())
    }
}

impl_xrequest_with_response!(SetModifierMapping);

/*
GetModifierMapping
     1     119                             opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetModifierMapping;

impl ToLeBytes for GetModifierMapping {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_MODIFIER_MAPPING);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(GetModifierMapping);

/*
NoOperation
     1     127                             opcode
     1                                     unused
     2     1+n                             request length
     4n                                    unused
*/

#[derive(Debug, Clone)]
pub struct NoOperation {
    pub unused: Vec<u32>,
}

impl ToLeBytes for NoOperation {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.unused.len();
        let request_length = 1 + n;

        write_le_bytes!(w, opcodes::NO_OPERATION);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, request_length as u16);
        for u in &self.unused {
            write_le_bytes!(w, u);
        }

        Ok(())
    }
}

impl_xrequest_without_response!(NoOperation);

bitmask! {
    #[repr(u16)]
    bitmask KeyModifier {
        SHIFT = 0x0001,
        LOCK = 0x0002,
        CONTROL = 0x0004,
        MOD_1 = 0x0008,
        MOD_2 = 0x0010,
        MOD_3 = 0x0020,
        MOD_4 = 0x0040,
        MOD_5 = 0x0080,

        /// Match any modifier key
        ANY = 0x8000,
    }
}
