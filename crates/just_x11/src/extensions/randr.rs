//! RANDR extension
//!
//! Implementation was done and tested using 1.6 version of the extension that that is the only one
//! that is supported

use crate::{
    atoms::AtomId,
    bitmask,
    connection::XConnection,
    error::Error,
    replies::read_vec,
    requests::write_le_bytes,
    utils::{impl_enum, impl_resource_id},
    FromLeBytes, ToLeBytes,
};

pub mod replies;
pub mod requests;

/// Name of the extension as returned by the X11 server. Can be used in [`crate::requests::QueryExtension`].
pub const EXTENSION_NAME: [u8; 5] = *b"RANDR";

pub const SUPPORTED_MAJOR: u32 = 1;
pub const SUPPORTED_MINOR: u32 = 6;

/* CRTC { XID } */

impl_resource_id!(CrtcId);

/* SIZEID { CARD16 } */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeId {
    inner: u16,
}

impl SizeId {
    #[inline(always)]
    fn to_le_bytes(&self) -> [u8; 2] {
        self.inner.to_le_bytes()
    }
}

impl From<u16> for SizeId {
    fn from(inner: u16) -> Self {
        Self { inner }
    }
}

impl From<SizeId> for u16 {
    fn from(value: SizeId) -> Self {
        value.inner
    }
}

// A.1 Common Types

/*
┌───
    ROTATION
        0x0001  Rotate_0
        0x0002  Rotate_90
        0x0004  Rotate_180
        0x0008  Rotate_270
        0x0010  Reflect_X
        0x0020  Reflect_Y
└───
        Used to encode both sets of possible rotations and individual
        selected rotations.
*/

// TODO: Double check reprs
bitmask! {
    #[repr(u16)]
    bitmask PossibleRotation {
        ROTATE_0 = 0x0001,
        ROTATE_90 = 0x0002,
        ROTATE_180 = 0x0004,
        ROTATE_270 = 0x0008,
        REFLECT_X = 0x0010,
        REFLECT_Y = 0x0020,
    }
}

impl_enum! {
    #[repr(u16)]
    enum Rotation {
        Rotate0 = 0x0001,
        Rotate90 = 0x0002,
        Rotate180 = 0x0004,
        Rotate270 = 0x0008,
        ReflectX = 0x0010,
        ReflectY = 0x0020,
    }
}

/*
┌───
    RRSELECTMASK
        0x0001  ScreenChangeNotifyMask
        0x0002  CrtcChangeNotifyMask            Added in version 1.2
        0x0004  OutputChangeNotifyMask          Added in version 1.2
        0x0008  OutputPropertyNotifyMask        Added in version 1.2
        0x0010  ProviderChangeNotifyMask        Added in version 1.4
        0x0020  ProviderPropertyNotifyMask      Added in version 1.4
        0x0040  ResourceChangeNotifyMask        Added in version 1.4

└───
      Event select mask for RRSelectInput
 */

bitmask! {
    #[repr(u16)]
    /// Event select mask for [`requests::SelectInput`]
    bitmask SelectMask {
        SCREEN_CHANGE_NOTIFY_MASK = 0x0001,
        CRTC_CHANGE_NOTIFY_MASK = 0x0002,
        OUTPUT_CHANGE_NOTIFY_MASK = 0x0004,
        OUTPUT_PROPERTY_NOTIFY_MASK = 0x0008,
        PROVIDER_CHANGE_NOTIFY_MASK = 0x0010,
        PROVIDER_PROPERTY_NOTIFY_MASK = 0x0020,
        RESOURCE_CHANGE_NOTIFY_MASK = 0x0040,
    }
}

/*
┌───
    RRCONFIGSTATUS
        0x0 Success
        0x1 InvalidConfigTime
        0x2 InvalidTime
        0x3 Failed
└───
        Return status for requests which depend on time.
*/

impl_enum! {
    #[repr(u8)]
    /// Return status for requests which depend on time.
    enum ConfigStatus {
        Success = 0x0,
        InvalidConfigTime = 0x1,
        InvalidTime = 0x2,
        Failed = 0x3,
    }
}

/*
┌───
    MODEINFO (32)                               Added in version 1.2
        4       CARD32          id
        2       CARD16          width in pixels
        2       CARD16          height in pixels
        4       CARD32          dot clock
        2       CARD16          h sync start
        2       CARD16          h sync end
        2       CARD16          h total
        2       CARD16          h skew
        2       CARD16          v sync start
        2       CARD16          v sync end
        2       CARD16          v total
        2       CARD16          name length
        4       SETofMODEFLAG   mode flags
└───
 */

#[derive(Debug, Clone, PartialEq, Eq)]
/// An output mode specifies the complete CRTC timings for a specific mode.
///
/// The vertical and horizontal synchronization rates can be computed given the dot clock and
/// the h total/v total values.
/// If the dot clock is zero, then all of the timing parameters and flags are not used,
/// and must be zero as this indicates that the timings are unknown or otherwise unused.
/// The name itself will be encoded separately in each usage.
pub struct ModeInfo {
    pub id: u32,
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub dot_closk: u32,
    pub h_sync_start: u16,
    pub h_sync_end: u16,
    pub h_total: u16,
    pub h_skew: u16,
    pub v_sync_start: u16,
    pub v_sync_end: u16,
    pub v_total: u16,
    pub name_length: u16,
    pub mode_flags: ModeFlag,
}

impl FromLeBytes for ModeInfo {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let id = conn.read_le_u32()?;
        let width_in_pixels = conn.read_le_u16()?;
        let height_in_pixels = conn.read_le_u16()?;
        let dot_closk = conn.read_le_u32()?;
        let h_sync_start = conn.read_le_u16()?;
        let h_sync_end = conn.read_le_u16()?;
        let h_total = conn.read_le_u16()?;
        let h_skew = conn.read_le_u16()?;
        let v_sync_start = conn.read_le_u16()?;
        let v_sync_end = conn.read_le_u16()?;
        let v_total = conn.read_le_u16()?;
        let name_length = conn.read_le_u16()?;
        let mode_flags = ModeFlag::from_le_bytes(conn)?;

        Ok(Self {
            id,
            width_in_pixels,
            height_in_pixels,
            dot_closk,
            h_sync_start,
            h_sync_end,
            h_total,
            h_skew,
            v_sync_start,
            v_sync_end,
            v_total,
            name_length,
            mode_flags,
        })
    }
}

/*
┌───
    MODEFLAG
        0x00000001      HSyncPositive
        0x00000002      HSyncNegative
        0x00000004      VSyncPositive
        0x00000008      VSyncNegative
        0x00000010      Interlace
        0x00000020      DoubleScan
        0x00000040      CSync
        0x00000080      CSyncPositive
        0x00000100      CSyncNegative
        0x00000200      HSkewPresent
        0x00000400      BCast
        0x00000800      PixelMultiplex
        0x00001000      DoubleClock
        0x00002000      ClockDivideBy2
└───
*/

bitmask! {
    #[repr(u32)]
    bitmask ModeFlag {
        HSYNC_POSITIVE = 0x00000001,
        HSYNC_NEGATIVE = 0x00000002,
        VSYNC_POSITIVE = 0x00000004,
        VSYNC_NEGATIVE = 0x00000008,
        INTERLACE = 0x00000010,
        DOUBLE_SCAN = 0x00000020,
        CSYNC = 0x00000040,
        CSYNC_POSITIVE = 0x00000080,
        CSYNC_NEGATIVE = 0x00000100,
        HSKEW_PRESENT = 0x00000200,
        BCAST = 0x00000400,
        PIXEL_MULTIPLEX = 0x00000800,
        DOUBLE_CLOCK = 0x00001000,
        CLOCK_DIVIDE_BY_2 = 0x00002000,
    }
}

/*
┌───
    CONNECTION
        0               Connected
        1               Disconnected
        2               UnknownConnection
└───
*/

impl_enum! {
    #[repr(u8)]
    enum Connection {
        Connected = 0,
        Disconnected = 1,
        UnknownConnection = 2,
    }
}

/*
┌───
    PROVIDER_CAPS                               Added in version 1.4
        0x00000001      SourceOutput
        0x00000002      SinkOutput
        0x00000004      SourceOffload
        0x00000008      SinkOffload
└───
 */

// TODO: Double check repr
bitmask! {
    #[repr(u32)]
    bitmask ProviderCapability {
        SOURCE_OUTPUT = 0x00000001,
        SINK_OUTPUT = 0x00000002,
        SOURCE_OFFLOAD = 0x00000004,
        SINK_OFFLOAD = 0x00000008,
    }
}

// A.1.1 Common Types added in version 1.5 of the protocol

/*
┌───
    MONITORINFO (16 + 4*n)
        4       ATOM            name
        1       BOOL            primary
        1       BOOL            automatic
        2       CARD16          ncrtcs
        2       INT16           x
        2       INT16           y
        2       CARD16          width in pixels
        2       CARD16          height in pixels
        4       CARD32          width in millimeters
        4       CARD32          height in millimeters
        4*n     CRTC            crtcs
└───
 */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorInfo {
    pub name: AtomId,
    pub primary: bool,
    pub automatic: bool,
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

impl ToLeBytes for MonitorInfo {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, self.name);
        write_le_bytes!(w, self.primary as u8);
        write_le_bytes!(w, self.automatic as u8);
        write_le_bytes!(w, self.crtcs.len() as u16);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width_in_pixels);
        write_le_bytes!(w, self.height_in_pixels);
        write_le_bytes!(w, self.width_in_millimeters);
        write_le_bytes!(w, self.height_in_millimeters);
        for crtc in &self.crtcs {
            write_le_bytes!(w, crtc);
        }

        Ok(())
    }
}
