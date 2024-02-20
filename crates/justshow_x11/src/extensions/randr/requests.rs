use crate::{
    requests::{write_le_bytes, XRequest},
    LeBytes, WindowId,
};

macro_rules! impl_xrequest_with_response {
    ($r:tt) => {
        impl XRequest for $r {
            type Reply = super::replies::$r;

            #[inline(always)]
            fn reply_type() -> Option<crate::replies::ReplyType> {
                Some(crate::replies::ReplyType::ExtensionRandr(
                    super::replies::ReplyType::$r,
                ))
            }
        }
    };
}

#[derive(Debug)]
pub struct GetMonitors {
    pub window: WindowId,
    pub get_active: bool,
}

impl LeBytes for GetMonitors {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, 140u8); // major opcode // FIXME: don't hardcode, query it from server
        write_le_bytes!(w, 42u8); // minor opcode

        // The spec says 2 not 3, why? idk, probably a bug.
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.get_active as u8);
        w.write_all(&[0u8; 3])?; // unused

        Ok(())
    }
}

impl_xrequest_with_response!(GetMonitors);
