use crate::{
    extensions::randr::{CrtcId, Rotation, SelectMask, SizeId},
    requests::{write_le_bytes, Timestamp},
    ToLeBytes, WindowId,
};

mod opcodes;

macro_rules! impl_xrequest_with_response {
    ($r:tt) => {
        impl $crate::requests::XRequestBase for $r {
            type Reply = super::replies::$r;

            #[inline(always)]
            fn reply_type() -> Option<crate::replies::ReplyType> {
                Some(crate::replies::ReplyType::ExtensionRandr(
                    super::replies::ReplyType::$r,
                ))
            }
        }

        impl $crate::requests::XExtensionRequest for $r {}
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

        impl $crate::requests::XExtensionRequest for $r {}
    };
}

/*
┌───
    RRQueryVersion

        1       CARD8                   major opcode
        1       0                       RandR opcode
        2       3                       length
        4       CARD32                  major version
        4       CARD32                  minor version
     ▶
└───
*/

#[derive(Debug, Clone)]
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
┌───
    RRSetScreenConfig

        1       CARD8                   major opcode
        1       2                       RandR opcode
        2       6                       length
        4       WINDOW                  window on screen to be configured
        4       TIMESTAMP               timestamp
        4       TIMESTAMP               config timestamp
        2       SIZEID                  size index
        2       ROTATION                rotation/reflection
        2       CARD16                  refresh rate (1.1 only)
        2       CARD16                  pad
     ▶
└───
*/

#[derive(Debug, Clone)]
pub struct SetScreenConfig {
    pub window: WindowId,
    pub timestamp: Timestamp,
    pub config_timestamp: Timestamp,
    pub size_index: SizeId,
    pub rotation: Rotation,
}

impl ToLeBytes for SetScreenConfig {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::SET_SCREEN_CONFIG);
        write_le_bytes!(w, 6u16); // request length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.timestamp);
        write_le_bytes!(w, self.config_timestamp);
        write_le_bytes!(w, self.size_index);
        write_le_bytes!(w, self.rotation);
        write_le_bytes!(w, 0u16); // refresh rate (deprecated)
        write_le_bytes!(w, 0u16); // pad

        Ok(())
    }
}

impl_xrequest_with_response!(SetScreenConfig);

/*
┌───
    RRSelectInput

        1       CARD8                   major opcode
        1       4                       RandR opcode
        2       3                       length
        4       WINDOW                  window
        2       SETofRRSELECTMASK       enable
        2       CARD16                  pad
└───
*/

#[derive(Debug, Clone)]
pub struct SelectInput {
    pub window: WindowId,
    pub enable: SelectMask,
}

impl ToLeBytes for SelectInput {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::SELECT_INPUT);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.window);
        write_le_bytes!(w, self.enable.raw());
        write_le_bytes!(w, 0u16); // pad

        Ok(())
    }
}

impl_xrequest_without_response!(SelectInput);

/*
┌───
    RRGetScreenInfo

        1       CARD8                   major opcode
        1       5                       RandR opcode
        2       2                       length
        4       WINDOW                  window
      ▶
└───
*/

#[derive(Debug, Clone)]
pub struct GetScreenInfo {
    pub window: WindowId,
}

impl ToLeBytes for GetScreenInfo {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SCREEN_INFO);
        write_le_bytes!(w, 2u16);
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(GetScreenInfo);

/*
┌───
    RRGetScreenSizeRange
        1       CARD8                   major opcode
        1       6                       RandR opcode
        2       2                       length
        4       WINDOW                  window
      ▶
└───
*/

#[derive(Debug, Clone)]
pub struct GetScreenSizeRange {
    pub window: WindowId,
}

impl ToLeBytes for GetScreenSizeRange {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SCREEN_SIZE_RANGE);
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(GetScreenSizeRange);

/*
┌───
    RRGetCrtcInfo
        1       CARD8                   major opcode
        1       20                      RandR opcode
        2       3                       length
        4       CRTC                    crtc
        4       TIMESTAMP               config-timestamp
      ▶
└───
*/

#[derive(Debug, Clone)]
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

/*
┌───
    RRGetScreenResourcesCurrent
        1       CARD8                   major opcode
        1       25                      RandR opcode
        2       2                       length
        4       WINDOW                  window
      ▶
└───
*/

#[derive(Debug, Clone)]
pub struct GetScreenResourcesCurrent {
    pub window: WindowId,
}

impl ToLeBytes for GetScreenResourcesCurrent {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_SCREEN_RESOURCES_CURRENT);
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.window);

        Ok(())
    }
}

impl_xrequest_with_response!(GetScreenResourcesCurrent);

/*
┌───
    RRGetMonitors
        1       CARD8                   major opcode
        1       42                      RandR opcode
        2       2                       request length
        4       WINDOW                  window
     ▶
└───
*/

#[derive(Debug, Clone)]
pub struct GetMonitors {
    pub window: WindowId,
    pub get_active: bool,
}

impl ToLeBytes for GetMonitors {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_MONITORS);

        // HACK: I know that the spec says 2 not 3 but it doesn't work otherwise
        write_le_bytes!(w, 3u16); // request length

        write_le_bytes!(w, self.window);

        // Why this is not in spec? idk.
        write_le_bytes!(w, self.get_active as u8);
        w.write_all(&[0u8; 3])?; // unused

        Ok(())
    }
}

impl_xrequest_with_response!(GetMonitors);
