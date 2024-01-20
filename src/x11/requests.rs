use crate::x11::{
    replies::ReplyType, utils::pad, Atom, Drawable, Font, GContext, LeBytes, Pixmap, Rectangle,
    Window, WindowClass, WindowVisual,
};
use std::{
    io::{self, Write},
    ops::BitOr,
};

mod opcodes;

// TODO: proper type
type Timestamp = u32;

macro_rules! impl_raw_field {
    ($ty:path, $setter:ident, $idx:expr) => {
        pub fn $setter(mut self, new_value: $ty) -> Self {
            self.values.values[$idx] = Some(new_value.into());
            self
        }
    };
}

macro_rules! impl_raw_fields_go {
    ($idx:expr $(,)?) => { };

    ($idx:expr, $setter:ident: $ty:path, $($rest:tt)*) => {
        impl_raw_field!($ty, $setter, $idx);
        impl_raw_fields_go!($idx + 1, $($rest)*);
    };
}

macro_rules! impl_raw_fields {
    ($($rest:tt)*) => {
        impl_raw_fields_go!(0, $($rest)*);
    };
}

pub trait XRequest: LeBytes {
    fn reply_type() -> Option<ReplyType> {
        None
    }
}

macro_rules! write_le_bytes {
    ($w:expr, $content:expr) => {
        $w.write_all(&(($content).to_le_bytes()))?;
    };
}

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

#[derive(Debug, Clone)]
pub struct InitializeConnection {
    pub major_version: u16,
    pub minor_version: u16,
    pub authorization_protocol_name: Vec<u8>,
    pub authorization_protocol_data: Vec<u8>,
}

impl LeBytes for InitializeConnection {
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

impl XRequest for InitializeConnection {}

#[derive(Debug, Clone)]
pub struct WindowCreationAttributes {
    values: ListOfValues<15>,
}

impl Default for WindowCreationAttributes {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowCreationAttributes {
    pub fn new() -> Self {
        Self {
            values: ListOfValues::new(),
        }
    }

    impl_raw_fields! {
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
    pub wid: Window,
    pub parent: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub window_class: WindowClass,
    pub visual: WindowVisual,
    pub attributes: WindowCreationAttributes,
}

impl LeBytes for CreateWindow {
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

impl XRequest for CreateWindow {}

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
    pub window: Window,
    pub attributes: WindowCreationAttributes,
}

impl LeBytes for ChangeWindowAttributes {
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

impl XRequest for ChangeWindowAttributes {}

/*
GetWindowAttributes
     1     3                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct GetWindowAttributes {
    pub window: Window,
}

impl LeBytes for GetWindowAttributes {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_WINDOW_ATTRIBUTES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for GetWindowAttributes {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetWindowAttributes)
    }
}

/*
DestroyWindow
     1     4                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct DestroyWindow {
    pub window: Window,
}

impl LeBytes for DestroyWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DESTROY_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for DestroyWindow {}

/*
DestroySubwindows
     1     5                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct DestroySubwindows {
    pub window: Window,
}

impl LeBytes for DestroySubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DESTROY_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for DestroySubwindows {}

/*
ChangeSaveSet
     1     6                               opcode
     1                                     mode
          0     Insert
          1     Delete
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ChangeSaveSetMode {
    Insert = 0,
    Delete = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct ChangeSaveSet {
    pub mode: ChangeSaveSetMode,
    pub window: Window,
}

impl LeBytes for ChangeSaveSet {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CHANGE_SAVE_SET);
        write_le_bytes!(w, self.mode as u8);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for ChangeSaveSet {}

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
    pub window: Window,
    pub parent: Window,
    pub x: i16,
    pub y: i16,
}

impl LeBytes for ReparentWindow {
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

impl XRequest for ReparentWindow {}

/*
MapWindow
     1     8                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct MapWindow {
    pub window: Window,
}

impl LeBytes for MapWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::MAP_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for MapWindow {}

/*
MapSubwindows
     1     9                               opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct MapSubwindows {
    pub window: Window,
}

impl LeBytes for MapSubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::MAP_SUBWINDOWS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for MapSubwindows {}

/*
UnmapWindow
     1     10                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct UnmapWindow {
    pub window: Window,
}

impl LeBytes for UnmapWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNMAP_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for UnmapWindow {}

/*
UnmapSubwindows
     1     11                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct UnmapSubwindows {
    pub window: Window,
}

impl LeBytes for UnmapSubwindows {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNMAP_SUBWINDOWS);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for UnmapSubwindows {}

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

#[derive(Debug, Clone)]
pub struct ConfigureWindowAttributes {
    values: ListOfValues<7>,
}

#[derive(Debug, Clone)]
pub struct ConfigureWindow {
    pub window: Window,
    pub attributes: ConfigureWindowAttributes,
}

impl LeBytes for ConfigureWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let (bitmask, n) = self.attributes.values.mask_and_count();

        write_le_bytes!(w, opcodes::CONFIGURE_WINDOW);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + n); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, bitmask);
        write_le_bytes!(w, 0u16); // unused
        self.attributes.values.to_le_bytes_if_set(w)?;

        Ok(())
    }
}

impl XRequest for ConfigureWindow {}

/*
CirculateWindow
     1     13                              opcode
     1                                     direction
          0     RaiseLowest
          1     LowerHighest
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CirculateWindowDirection {
    RaiseLowest = 0,
    LowerHighest = 1,
}

#[derive(Debug, Clone, Copy)]
pub struct CirculateWindow {
    pub direction: CirculateWindowDirection,
    pub window: Window,
}

impl LeBytes for CirculateWindow {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CIRCULATE_WINDOW);
        write_le_bytes!(w, self.direction as u8);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for CirculateWindow {}

/*
GetGeometry
     1     14                              opcode
     1                                     unused
     2     2                               request length
     4     DRAWABLE                        drawable
*/

pub struct GetGeometry {
    pub drawable: Drawable,
}

impl LeBytes for GetGeometry {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_GEOMETRY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // size
        write_le_bytes!(w, self.drawable.value());

        Ok(())
    }
}

impl XRequest for GetGeometry {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetGeometry)
    }
}

/*
QueryTree
     1     15                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryTree {
    pub window: Window,
}

impl LeBytes for QueryTree {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_TREE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for QueryTree {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::QueryTree)
    }
}

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
    pub name: Vec<u8>,
}

impl LeBytes for InternAtom {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n = self.name.len();
        let p = pad(n);
        let request_len = (2 + (n + p) / 4) as u16;

        write_le_bytes!(w, opcodes::INTERN_ATOM);
        write_le_bytes!(w, self.only_if_exists as u8);
        write_le_bytes!(w, request_len);
        write_le_bytes!(w, n);
        write_le_bytes!(w, 0u16); // unused
        w.write_all(&self.name)?;
        write_le_bytes!(w, p);

        Ok(())
    }
}

impl XRequest for InternAtom {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::InternAtom)
    }
}

/*
GetAtomName
     1     17                              opcode
     1                                     unused
     2     2                               request length
     4     ATOM                            atom
*/

#[derive(Debug, Clone, Copy)]
pub struct GetAtomName {
    pub atom: Atom,
}

impl LeBytes for GetAtomName {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_ATOM_NAME);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.atom);

        Ok(())
    }
}

impl XRequest for GetAtomName {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetAtomName)
    }
}

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

#[derive(Debug, Clone)]
pub struct ChangeProperty {
    pub mode: u8, // TODO: type
    pub window: Window,
    pub property: Atom,
    pub type_: Atom,
    pub format: u8, // TODO: type
    pub data: Vec<u8>,
}

impl LeBytes for ChangeProperty {
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
        write_le_bytes!(w, n);
        w.write_all(&self.data)?;
        w.write_all(&vec![0u8; p])?;

        Ok(())
    }
}

impl XRequest for ChangeProperty {}

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
    pub window: Window,
    pub property: Atom,
}

impl LeBytes for DeleteProperty {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::DELETE_PROPERTY);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.property);

        Ok(())
    }
}

impl XRequest for DeleteProperty {}

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
    pub window: Window,
    pub property: Atom,
    pub type_: Atom,
    pub long_offset: u32,
    pub long_length: u32,
}

impl LeBytes for GetProperty {
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

impl XRequest for GetProperty {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetProperty)
    }
}

/*
ListProperties
     1     21                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct ListProperties {
    pub window: Window,
}

impl LeBytes for ListProperties {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::LIST_PROPERTIES);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for ListProperties {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::ListProperties)
    }
}

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
    pub owner: Window,
    pub selection: Atom,
    pub time: u32,
}

impl LeBytes for SetSelectionOwner {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_SELECTION_OWNER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.owner);
        write_le_bytes!(w, self.selection);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for SetSelectionOwner {}

/*
GetSelectionOwner
     1     23                              opcode
     1                                     unused
     2     2                               request length
     4     ATOM                            selection
*/

#[derive(Debug, Clone, Copy)]
pub struct GetSelectionOwner {
    pub selection: Atom,
}

impl LeBytes for GetSelectionOwner {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SELECTION_OWNER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.selection);

        Ok(())
    }
}

impl XRequest for GetSelectionOwner {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetSelectionOwner)
    }
}

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
    pub requestor: Window,
    pub selection: Atom,
    pub target: Atom,
    pub property: Atom,
    pub time: u32,
}

impl LeBytes for ConvertSelection {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CONVERT_SELECTION);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.requestor);
        write_le_bytes!(w, self.selection);
        write_le_bytes!(w, self.target);
        write_le_bytes!(w, self.property);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for ConvertSelection {}

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
    pub destination: Window,
    pub event_mask: u32,
    pub event: [u8; 32], // TODO: type
}

impl LeBytes for SendEvent {
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

impl XRequest for SendEvent {}

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
    pub grab_window: Window,
    pub event_mask: u16,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub confine_to: Window,
    pub cursor: u32,
    pub time: u32,
}

impl LeBytes for GrabPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_POINTER);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        write_le_bytes!(w, self.confine_to);
        write_le_bytes!(w, self.cursor);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for GrabPointer {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GrabPointer)
    }
}

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
    pub time: u32,
}

impl LeBytes for UngrabPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for UngrabPointer {}

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
    pub grab_window: Window,
    pub event_mask: u16,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
    pub confine_to: Window,
    pub cursor: u32,
    pub button: u8,
    pub modifiers: u16,
}

impl LeBytes for GrabButton {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_BUTTON);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        write_le_bytes!(w, self.confine_to);
        write_le_bytes!(w, self.cursor);
        write_le_bytes!(w, self.button);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, self.modifiers);

        Ok(())
    }
}

impl XRequest for GrabButton {}

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
    pub button: u8,
    pub grab_window: Window,
    pub modifiers: u16,
}

impl LeBytes for UngrabButton {
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

impl XRequest for UngrabButton {}

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
    pub cursor: u32,
    pub time: u32,
    pub event_mask: u16,
}

impl LeBytes for ChangeActivePointerGrab {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CHANGE_ACTIVE_POINTER_GRAB);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.cursor);
        write_le_bytes!(w, self.time);
        write_le_bytes!(w, self.event_mask);
        write_le_bytes!(w, 0u16); // unused

        Ok(())
    }
}

impl XRequest for ChangeActivePointerGrab {}

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
    pub grab_window: Window,
    pub time: u32,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
}

impl LeBytes for GrabKeyboard {
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

impl XRequest for GrabKeyboard {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GrabKeyboard)
    }
}

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

impl LeBytes for UngrabKeyboard {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_KEYBOARD);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for UngrabKeyboard {}

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

#[derive(Debug, Clone, Copy)]
pub struct GrabKey {
    pub owner_events: bool,
    pub grab_window: Window,
    pub modifiers: u16,
    pub key: u8,
    pub pointer_mode: u8,
    pub keyboard_mode: u8,
}

impl LeBytes for GrabKey {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_KEY);
        write_le_bytes!(w, self.owner_events as u8);
        write_le_bytes!(w, 4u16); // length
        write_le_bytes!(w, self.grab_window);
        write_le_bytes!(w, self.modifiers);
        write_le_bytes!(w, self.key);
        write_le_bytes!(w, self.pointer_mode);
        write_le_bytes!(w, self.keyboard_mode);
        w.write_all(&[0u8; 3])?; // unused

        Ok(())
    }
}

impl XRequest for GrabKey {}

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
    pub key: u8,
    pub grab_window: Window,
    pub modifiers: u16,
}

impl LeBytes for UngrabKey {
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

impl XRequest for UngrabKey {}

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

#[derive(Debug, Clone, Copy)]
pub struct AllowEvents {
    pub mode: u8, // TODO: type
    pub time: u32,
}

impl LeBytes for AllowEvents {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::ALLOW_EVENTS);
        write_le_bytes!(w, self.mode);
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for AllowEvents {}

/*
GrabServer
     1     36                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct GrabServer;

impl LeBytes for GrabServer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GRAB_SERVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl XRequest for GrabServer {}

/*
UngrabServer
     1     37                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct UngrabServer;

impl LeBytes for UngrabServer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::UNGRAB_SERVER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl XRequest for UngrabServer {}

/*
QueryPointer
     1     38                              opcode
     1                                     unused
     2     2                               request length
     4     WINDOW                          window
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryPointer {
    pub window: Window,
}

impl LeBytes for QueryPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl XRequest for QueryPointer {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::QueryPointer)
    }
}

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
    pub window: Window,
    pub start: Timestamp,
    pub stop: Timestamp,
}

impl LeBytes for GetMotionEvents {
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

impl XRequest for GetMotionEvents {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetMotionEvents)
    }
}

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
    pub src_window: Window,
    pub dst_window: Window,
    pub src_x: i16,
    pub src_y: i16,
}

impl LeBytes for TranslateCoordinates {
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

impl XRequest for TranslateCoordinates {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::TranslateCoordinates)
    }
}

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
    pub src_window: Window,
    pub dst_window: Window,
    pub src_x: i16,
    pub src_y: i16,
    pub src_width: u16,
    pub src_height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
}

impl LeBytes for WarpPointer {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::WARP_POINTER);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 6u16); // length
        write_le_bytes!(w, self.src_window);
        write_le_bytes!(w, self.dst_window);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);
        write_le_bytes!(w, self.src_width);
        write_le_bytes!(w, self.src_height);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);

        Ok(())
    }
}

impl XRequest for WarpPointer {}

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

#[derive(Debug, Clone, Copy)]
pub struct SetInputFocus {
    pub revert_to: u8,
    pub focus: Window,
    pub time: Timestamp,
}

impl LeBytes for SetInputFocus {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::SET_INPUT_FOCUS);
        write_le_bytes!(w, self.revert_to);
        write_le_bytes!(w, 3u16); // length
        write_le_bytes!(w, self.focus);
        write_le_bytes!(w, self.time);

        Ok(())
    }
}

impl XRequest for SetInputFocus {}

/*
GetInputFocus
     1     43                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone)]
pub struct GetInputFocus;

impl LeBytes for GetInputFocus {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, &opcodes::GET_INPUT_FOCUS);
        write_le_bytes!(w, &0u8); // unused
        write_le_bytes!(w, &1u16); // length

        Ok(())
    }
}

impl XRequest for GetInputFocus {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetInputFocus)
    }
}

/*
QueryKeymap
     1     44                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryKeymap;

impl LeBytes for QueryKeymap {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_KEYMAP);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl XRequest for QueryKeymap {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::QueryKeymap)
    }
}

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
    pub fid: Font,
    pub name: Vec<u8>,
}

impl LeBytes for OpenFont {
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

impl XRequest for OpenFont {}

/*
CloseFont
     1     46                              opcode
     1                                     unused
     2     2                               request length
     4     FONT                            font
*/

#[derive(Debug, Clone, Copy)]
pub struct CloseFont {
    pub font: Font,
}

impl LeBytes for CloseFont {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::CLOSE_FONT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.font);

        Ok(())
    }
}

impl XRequest for CloseFont {}

/*
QueryFont
     1     47                              opcode
     1                                     unused
     2     2                               request length
     4     FONTABLE                        font
*/

#[derive(Debug, Clone, Copy)]
pub struct QueryFont {
    pub font: Font, // TODO: Fontable
}

impl LeBytes for QueryFont {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_FONT);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 2u16); // length
        write_le_bytes!(w, self.font);

        Ok(())
    }
}

impl XRequest for QueryFont {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::QueryFont)
    }
}

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
    pub font: Font, // TODO: Fontable
    pub string: Vec<u16>,
}

impl LeBytes for QueryTextExtents {
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

impl XRequest for QueryTextExtents {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::QueryTextExtents)
    }
}

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

impl LeBytes for ListFonts {
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

impl XRequest for ListFonts {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::ListFonts)
    }
}

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

impl LeBytes for ListFontsWithInfo {
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

impl XRequest for ListFontsWithInfo {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::ListFontsWithInfo)
    }
}

// TODO: SetFontPath: Need to figure out LISTofSTR encoding first

/*
GetFontPath
     1     52                              opcode
     1                                     unused
     2     1                               request length
*/

#[derive(Debug, Clone, Copy)]
pub struct GetFontPath;

impl LeBytes for GetFontPath {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        write_le_bytes!(w, opcodes::GET_FONT_PATH);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 1u16); // length

        Ok(())
    }
}

impl XRequest for GetFontPath {
    fn reply_type() -> Option<ReplyType> {
        Some(ReplyType::GetFontPath)
    }
}

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
    pub gc: GContext,
    pub rectangles: Vec<Rectangle>,
}

impl LeBytes for PolyFillRectangle {
    fn to_le_bytes(&self, w: &mut impl Write) -> io::Result<()> {
        let n: u16 = self.rectangles.len() as u16;

        write_le_bytes!(w, opcodes::POLY_FILL_RECTANGLE);
        write_le_bytes!(w, 0u8); // unused
        write_le_bytes!(w, 3 + (2 * n)); // request length
        write_le_bytes!(w, self.drawable.value());
        write_le_bytes!(w, self.gc);
        for rectangle in &self.rectangles {
            rectangle.to_le_bytes(w)?;
        }

        Ok(())
    }
}

impl XRequest for PolyFillRectangle {}

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

#[derive(Debug, Clone)]
pub struct GContextSettings {
    values: ListOfValues<23>,
}

impl Default for GContextSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl GContextSettings {
    pub fn new() -> Self {
        Self {
            values: ListOfValues::new(),
        }
    }

    impl_raw_fields! {
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
      set_tile: Pixmap,
      set_stipple: Pixmap,
      set_tile_stipple_x_origin: u16,
      set_tile_stipple_y_origin: u16,
      set_font: Font,
      set_subwindow_mode: u32,
      set_graphics_exposures: bool,
      set_clip_x_origin: u16,
      set_clip_y_origin: u16,
      set_clip_mask: Pixmap, // TODO: or None
      set_dash_offset: u16,
      set_dashes: u8,
      set_arc_mode: u32,
    }
}

#[derive(Debug, Clone)]
pub struct CreateGC {
    pub cid: GContext,
    pub drawable: Drawable,
    pub values: GContextSettings,
}

impl LeBytes for CreateGC {
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

impl XRequest for CreateGC {}

#[derive(Debug, Clone)]
pub struct ChangeGC {
    pub gcontext: GContext,
    pub values: GContextSettings,
}

impl LeBytes for ChangeGC {
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

impl XRequest for ChangeGC {}

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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PutImageFormat {
    Bitmap = 0,
    XYPixmap = 1,

    /// Probably the one you want. Pixels stores as scanlines
    ZPixmap = 2,
}

#[derive(Debug, Clone)]
pub struct PutImage<'data> {
    pub format: PutImageFormat,
    pub drawable: Drawable,
    pub gc: GContext,
    pub width: u16,
    pub height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub left_pad: u8,

    /// If format is [`PutImageFormat::ZPixmap`], `depth` must equal to depth of `drawable`
    pub depth: u8,
    pub data: &'data [u8],
}

impl<'data> LeBytes for PutImage<'data> {
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

impl<'data> XRequest for PutImage<'data> {}

// TODO: Move to events or something

pub struct EventType {
    value: u32,
}

// TODO: Macro for these
impl EventType {
    pub const KEY_PRESS: Self = Self { value: 0x00000001 };
    pub const KEY_RELEASE: Self = Self { value: 0x00000002 };
    pub const BUTTON_PRESS: Self = Self { value: 0x00000004 };
    pub const BUTTON_RELEASE: Self = Self { value: 0x00000008 };
    pub const ENTER_WINDOW: Self = Self { value: 0x00000010 };
    pub const LEAVE_WINDOW: Self = Self { value: 0x00000020 };
    pub const POINTER_MOTION: Self = Self { value: 0x00000040 };
    pub const POINTER_MOTION_HINT: Self = Self { value: 0x00000080 };
    pub const BUTTON1_MOTION: Self = Self { value: 0x00000100 };
    pub const BUTTON2_MOTION: Self = Self { value: 0x00000200 };
    pub const BUTTON3_MOTION: Self = Self { value: 0x00000400 };
    pub const BUTTON4_MOTION: Self = Self { value: 0x00000800 };
    pub const BUTTON5_MOTION: Self = Self { value: 0x00001000 };
    pub const BUTTON_MOTION: Self = Self { value: 0x00002000 };
    pub const KEYMAP_STATE: Self = Self { value: 0x00004000 };
    pub const EXPOSURE: Self = Self { value: 0x00008000 };
    pub const VISIBILITY_CHANGE: Self = Self { value: 0x00010000 };
    pub const STRUCTURE_NOTIFY: Self = Self { value: 0x00020000 };
    pub const RESIZE_REDIRECT: Self = Self { value: 0x00040000 };
    pub const SUBSTRUCTURE_NOTIFY: Self = Self { value: 0x00080000 };
    pub const SUBSTRUCTURE_REDIRECT: Self = Self { value: 0x00100000 };
    pub const FOCUS_CHANGE: Self = Self { value: 0x00200000 };
    pub const PROPERTY_CHANGE: Self = Self { value: 0x00400000 };
    pub const COLORMAP_CHANGE: Self = Self { value: 0x00800000 };
    pub const OWNER_GRAB_BUTTON: Self = Self { value: 0x01000000 };

    fn contains(self, other: Self) -> bool {
        (self.value & other.value) != 0
    }
}

impl BitOr for EventType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value | rhs.value,
        }
    }
}

impl From<EventType> for u32 {
    fn from(val: EventType) -> Self {
        val.value
    }
}
