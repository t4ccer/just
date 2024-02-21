use crate::{
    requests::{write_le_bytes, Timestamp, XExtensionRequest, XRequestBase},
    LeBytes, WindowId,
};

use super::replies::CrtcId;

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

impl LeBytes for GetMonitors {
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

impl LeBytes for GetCrtcInfo {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_CRTC_INFO);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.crtc);
        write_le_bytes!(w, self.timestamp);

        Ok(())
    }
}

impl_xrequest_with_response!(GetCrtcInfo);
