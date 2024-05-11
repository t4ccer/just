use crate::{connection::XConnection, error::Error, FromLeBytes, VisualId};

macro_rules! impl_xreply {
    ($t:tt) => {
        impl $crate::XReply for $t {
            #[inline(always)]
            fn from_reply(reply: $crate::replies::SomeReply) -> Option<Self> {
                match reply {
                    $crate::replies::SomeReply::ExtensionMitShm(SomeReply::$t(r)) => Some(r),
                    _ => None,
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryVersion {
    pub shared_pixmaps: bool,
    pub major_version: u16,
    pub minor_version: u16,
    pub uid: u16,
    pub gid: u16,
    pub pixmap_format: u8,
}

impl FromLeBytes for QueryVersion {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let shared_pixmaps = conn.read_bool()?;
        let _sequence_number = conn.read_le_u16()?;
        let _length = conn.read_le_u32()?;
        let major_version = conn.read_le_u16()?;
        let minor_version = conn.read_le_u16()?;
        let uid = conn.read_le_u16()?;
        let gid = conn.read_le_u16()?;
        let pixmap_format = conn.read_u8()?;
        drop(conn.drain(15)?);

        Ok(Self {
            shared_pixmaps,
            major_version,
            minor_version,
            uid,
            gid,
            pixmap_format,
        })
    }
}

impl_xreply!(QueryVersion);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetImage {
    pub depth: u8,
    pub visual: VisualId,
    pub size: u32,
}

impl FromLeBytes for GetImage {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _length = conn.read_le_u32()?;
        let visual = VisualId::from_le_bytes(conn)?;
        let size = conn.read_le_u32()?;

        drop(conn.drain(16)?);

        Ok(Self {
            depth,
            visual,
            size,
        })
    }
}

impl_xreply!(GetImage);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSegment {
    pub nfd: u8,
}

impl FromLeBytes for CreateSegment {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let nfd = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _length = conn.read_le_u32()?;

        drop(conn.drain(24)?);

        Ok(Self { nfd })
    }
}

impl_xreply!(CreateSegment);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SomeReply {
    QueryVersion(QueryVersion),
    GetImage(GetImage),
    CreateSegment(CreateSegment),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyType {
    QueryVersion,
    GetImage,
    CreateSegment,
}
