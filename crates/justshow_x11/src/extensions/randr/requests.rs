use crate::{
    extensions::randr::CrtcId,
    requests::{write_le_bytes, Timestamp, XExtensionRequest, XRequestBase},
    ToLeBytes, WindowId,
};

mod opcodes;

macro_rules! impl_xrequest_with_response {
    ($r:tt) => {
        impl XRequestBase for $r {
            type Reply = super::replies::$r;

            #[inline(always)]
            fn reply_type() -> Option<crate::replies::ReplyType> {
                Some(crate::replies::ReplyType::ExtensionRandr(
                    super::replies::ReplyType::$r,
                ))
            }
        }

        impl XExtensionRequest for $r {}
    };
}

/*
RRQueryVersion
    1       CARD8                   major opcode
    1       0                       RandR opcode
    2       3                       length
    4       CARD32                  major version
    4       CARD32                  minor version
*/

#[derive(Debug)]
pub struct QueryVersion {
    pub major_version: u32,
    pub minor_version: u32,
}

impl ToLeBytes for QueryVersion {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_VERSION);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.major_version);
        write_le_bytes!(w, self.minor_version);

        Ok(())
    }
}

impl_xrequest_with_response!(QueryVersion);

/*
RRGetMonitors
    1       CARD8                   major opcode
    1       42                      RandR opcode
    2       2                       request length
    4       WINDOW                  window
*/

#[derive(Debug)]
pub struct GetMonitors {
    pub window: WindowId,
    pub get_active: bool,
}

impl ToLeBytes for GetMonitors {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_MONITORS);

        // The spec says 2 not 3, why? idk, probably a bug.
        write_le_bytes!(w, 3u16); // request length

        write_le_bytes!(w, self.window);

        // Why this is not in spec? idk.
        write_le_bytes!(w, self.get_active as u8);
        w.write_all(&[0u8; 3])?; // unused

        Ok(())
    }
}

impl_xrequest_with_response!(GetMonitors);

/*
RRGetCrtcInfo
    1       CARD8                   major opcode
    1       20                      RandR opcode
    2       3                       length
    4       CRTC                    crtc
    4       TIMESTAMP               config-timestamp
*/

#[derive(Debug)]
pub struct GetCrtcInfo {
    pub crtc: CrtcId,
    pub timestamp: Timestamp,
}

impl ToLeBytes for GetCrtcInfo {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_CRTC_INFO);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.crtc);
        write_le_bytes!(w, self.timestamp);

        Ok(())
    }
}

impl_xrequest_with_response!(GetCrtcInfo);

// TODO
pub struct SelectInput {}
