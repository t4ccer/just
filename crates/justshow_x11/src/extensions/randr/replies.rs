use crate::{
    atoms::AtomId,
    connection::XConnection,
    error::Error,
    replies::{read_vec, XReply},
    requests::Timestamp,
    utils::impl_resource_id,
};

macro_rules! impl_xreply {
    ($t:tt) => {
        impl XReply for $t {
            fn from_reply(reply: crate::replies::SomeReply) -> Option<Self> {
                match reply {
                    crate::replies::SomeReply::ExtensionRandr(SomeReply::$t(r)) => Some(r),
                    _ => None,
                }
            }
        }
    };
}

impl_resource_id!(CrtcId);

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub name: AtomId,
    pub primary: bool,
    pub automatic: bool,
    pub ncrtcs: u16,
    pub x: i16,
    pub y: i16,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u32,
    pub height_in_millimeters: u32,
    pub crtcs: Vec<CrtcId>,
}

impl MonitorInfo {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let name = AtomId::unchecked_from(conn.read_le_u32()?);
        let primary = conn.read_bool()?;
        let automatic = conn.read_bool()?;
        let ncrtcs = conn.read_le_u16()?;
        let x = conn.read_le_i16()?;
        let y = conn.read_le_i16()?;
        let width_in_pixels = conn.read_le_u16()?;
        let height_in_pixels = conn.read_le_u16()?;
        let width_in_millimeters = conn.read_le_u32()?;
        let height_in_millimeters = conn.read_le_u32()?;
        let crtcs = read_vec!(ncrtcs, CrtcId::unchecked_from(conn.read_le_u32()?));

        Ok(Self {
            name,
            primary,
            automatic,
            ncrtcs,
            x,
            y,
            width_in_pixels,
            height_in_pixels,
            width_in_millimeters,
            height_in_millimeters,
            crtcs,
        })
    }
}

/*
RRGetMonitors
▶
     1       1                       Reply
     1                               unused
     2       CARD16                  sequence number
     4       6*n + o                 reply length
     4       TIMESTAMP               timestamp
     4       n                       nmonitors
     4       o                       noutputs
     12                              unused
     n*24+o*4 LISTofMONITORINFO      monitors
*/

#[derive(Debug, Clone)]
pub struct GetMonitors {
    pub timestamp: Timestamp,
    pub monitors: Vec<MonitorInfo>,
}

impl GetMonitors {
    pub(crate) fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = dbg!(conn.read_le_u32()?);
        let timestamp = Timestamp::from(conn.read_le_u32()?);
        let nmonitors = conn.read_le_u32()?;
        let _noutputs = conn.read_le_u32()?;
        drop(conn.drain(12)?);
        let monitors = read_vec!(nmonitors, MonitorInfo::from_le_bytes(conn)?);

        Ok(Self {
            timestamp,
            monitors,
        })
    }
}

impl_xreply!(GetMonitors);

#[derive(Debug, Clone)]
pub enum SomeReply {
    GetMonitors(GetMonitors),
}

#[derive(Debug, Clone, Copy)]
pub enum ReplyType {
    GetMonitors,
}
