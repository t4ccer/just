use std::num::NonZeroU32;

use crate::x11::{connection::XConnection, error::Error, ResourceId, Window};

#[derive(Debug, Clone)]
pub struct KeyPressRelease {
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
}

impl KeyPressRelease {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let detail = conn.read_u8()?;
        let _sequence_number = conn.read_be_u16()?;
        let time = conn.read_be_u32()?;
        let root = conn.read_be_u32()?;
        let event = conn.read_be_u32()?;
        let child = conn.read_be_u32()?;
        let root_x = conn.read_be_i16()?;
        let root_y = conn.read_be_i16()?;
        let event_x = conn.read_be_i16()?;
        let event_y = conn.read_be_i16()?;
        let state = conn.read_be_u16()?;
        let same_screen = conn.read_bool()?;
        let _unused = conn.drain(1)?;

        Ok(Self {
            detail,
            time,
            root,
            event,
            child,
            root_x,
            root_y,
            event_x,
            event_y,
            state,
            same_screen,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ResizeRequest {
    pub window: Window,
    pub width: u16,
    pub height: u16,
}

impl ResizeRequest {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_be_u16()?;
        let window = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let width = conn.read_be_u16()?;
        let height = conn.read_be_u16()?;

        Ok(Self {
            window,
            width,
            height,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MapNotify {
    pub event: Window,
    pub window: Window,
    pub override_redirect: bool,
}

impl MapNotify {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_be_u16()?;
        let event = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let window = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let override_redirect = conn.read_bool()?;
        let _unused = conn.drain(19)?;

        Ok(Self {
            event,
            window,
            override_redirect,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MapRequest {
    pub parent: Window,
    pub window: Window,
}

impl MapRequest {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_be_u16()?;
        let parent = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let window = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let _unused = conn.drain(20)?;

        Ok(Self { parent, window })
    }
}

#[derive(Debug, Clone)]
pub struct ConfigureNotify {
    pub event: Window,
    pub window: Window,
    pub above_sibling: Option<Window>,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub override_redirect: bool,
}

impl ConfigureNotify {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_be_u16()?;
        let event = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let window = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let above_sibling =
            NonZeroU32::new(conn.read_be_u32()?).map(|value| Window(ResourceId { value }));
        let x = conn.read_be_i16()?;
        let y = conn.read_be_i16()?;
        let width = conn.read_be_u16()?;
        let height = conn.read_be_u16()?;
        let border_width = conn.read_be_u16()?;
        let override_redirect = conn.read_bool()?;
        let _unused = conn.drain(5)?;

        Ok(Self {
            event,
            window,
            above_sibling,
            x,
            y,
            width,
            height,
            border_width,
            override_redirect,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    KeyPress(KeyPressRelease),        // 2
    KeyRelease(KeyPressRelease),      // 3
    ButtonPress(KeyPressRelease),     // 4
    ButtonRelease(KeyPressRelease),   // 5
    MapNotify(MapNotify),             // 19
    MapRequest(MapRequest),           // 20
    ConfigureNotify(ConfigureNotify), // 22
    ResizeRequest(ResizeRequest),     // 25
    Temp,                             // invalid
}
