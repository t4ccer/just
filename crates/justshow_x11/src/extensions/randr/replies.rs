use crate::{
    connection::XConnection,
    error::Error,
    extensions::randr::MonitorInfo,
    replies::{read_vec, XReply},
    requests::Timestamp,
    utils::impl_resource_id,
    FromLeBytes,
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

/*
RRQueryVersion
  ▶
    1       1                       Reply
    1                               unused
    2       CARD16                  sequence number
    4       0                       reply length
    1       CARD32                  major version
    1       CARD32                  minor version
 */

#[derive(Debug, Clone)]
pub struct QueryVersion {
    pub major_version: u32,
    pub minor_version: u32,
}

impl FromLeBytes for QueryVersion {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_nubmer = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let major_version = conn.read_le_u32()?;
        let minor_version = conn.read_le_u32()?;

        Ok(Self {
            major_version,
            minor_version,
        })
    }
}

impl_xreply!(QueryVersion);

/*
RRGetCrtcInfo
  ▶
    1       1                       Reply
    1       RRCONFIGSTATUS          status
    2       CARD16                  sequence number
    4       o+p                     reply length
    4       TIMESTAMP               timestamp
    2       INT16                   x
    2       INT16                   y
    2       CARD16                  width
    2       CARD16                  height
    4       MODE                    mode
    2       ROTATION                current rotation and reflection
    2       ROTATION                set of possible rotations
    2       o                       number of outputs
    2       p                       number of possible outputs
    4o      LISTofOUTPUT            outputs
    4p      LISTofOUTPUT            possible outputs
*/

impl_resource_id!(OutputId);

#[derive(Debug, Clone)]
pub struct GetCrtcInfo {
    pub status: u8,
    pub timestamp: Timestamp,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub mode: u32,
    pub current_rotation: u16,
    pub available_rotations: u16,
    pub outputs: Vec<OutputId>,
    pub possible_outputs: Vec<OutputId>,
}

impl FromLeBytes for GetCrtcInfo {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = conn.read_u8()?;
        let _sequence_nubmer = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let timestamp = Timestamp::from(conn.read_le_u32()?);
        let x = conn.read_le_i16()?;
        let y = conn.read_le_i16()?;
        let width = conn.read_le_u16()?;
        let height = conn.read_le_u16()?;
        let mode = conn.read_le_u32()?;
        let current_rotation = conn.read_le_u16()?;
        let available_rotations = conn.read_le_u16()?;
        let outputs_count = conn.read_le_u16()?;
        let possible_outputs_count = conn.read_le_u16()?;
        let outputs = read_vec!(outputs_count, OutputId::unchecked_from(conn.read_le_u32()?));
        let possible_outputs = read_vec!(
            possible_outputs_count,
            OutputId::unchecked_from(conn.read_le_u32()?)
        );

        Ok(Self {
            status,
            timestamp,
            x,
            y,
            width,
            height,
            mode,
            current_rotation,
            available_rotations,
            outputs,
            possible_outputs,
        })
    }
}

impl_xreply!(GetCrtcInfo);

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

impl FromLeBytes for GetMonitors {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let timestamp = conn.read_le_u32()?;
        eprintln!("{:#08x}", timestamp);
        let timestamp = Timestamp::from(timestamp);
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
    QueryVersion(QueryVersion),
    GetCrtcInfo(GetCrtcInfo),
    GetMonitors(GetMonitors),
}

#[derive(Debug, Clone, Copy)]
pub enum ReplyType {
    QueryVersion,
    GetCrtcInfo,
    GetMonitors,
}
