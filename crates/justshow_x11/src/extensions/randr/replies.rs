use crate::{
    connection::XConnection,
    error::Error,
    extensions::{
        randr::{ConfigStatus, MonitorInfo, PossibleRotation},
        render::Subpixel,
    },
    replies::{read_vec, XReply},
    requests::Timestamp,
    utils::impl_resource_id,
    FromLeBytes, WindowId,
};

macro_rules! impl_xreply {
    ($t:tt) => {
        impl XReply for $t {
            fn from_reply(reply: $crate::replies::SomeReply) -> Option<Self> {
                match reply {
                    $crate::replies::SomeReply::ExtensionRandr(SomeReply::$t(r)) => Some(r),
                    _ => None,
                }
            }
        }
    };
}

/*
┌───
    RRQueryVersion
      ▶
        1       1                       Reply
        1                               unused
        2       CARD16                  sequence number
        4       0                       reply length
        1       CARD32                  major version
        1       CARD32                  minor version
└───
*/

#[derive(Debug, Clone)]
pub struct QueryVersion {
    pub major_version: u32,
    pub minor_version: u32,
}

impl FromLeBytes for QueryVersion {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let major_version = conn.read_le_u32()?;
        let minor_version = conn.read_le_u32()?;

        // HACK: One may ask why are we doing this here. I don't know.
        // Spec doesn't say to do it but we get 16 zero bytes at the end of the response
        // so we're dropping it here
        drop(conn.drain(16)?);

        Ok(Self {
            major_version,
            minor_version,
        })
    }
}

impl_xreply!(QueryVersion);

/*
┌───
    RRSetScreenConfig
      ▶
        1       1                       Reply
        1       RRCONFIGSTATUS          status
        2       CARD16                  sequence number
        4       0                       reply length
        4       TIMESTAMP               new timestamp
        4       TIMESTAMP               new configuration timestamp
        4       WINDOW                  root
        2       SUBPIXELORDER           subpixel order defined in Render
        2       CARD16                  pad4
        4       CARD32                  pad5
        4       CARD32                  pad6
└───
*/

#[derive(Debug, Clone)]
pub struct SetScreenConfig {
    pub status: ConfigStatus,
    pub new_timestamp: Timestamp,
    pub new_configuration_timestamp: Timestamp,
    pub root: WindowId,
    pub subpixel_order: Subpixel,
}

impl FromLeBytes for SetScreenConfig {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let status = ConfigStatus::from_le_bytes(conn)?;
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let new_timestamp = Timestamp::from_le_bytes(conn)?;
        let new_configuration_timestamp = Timestamp::from_le_bytes(conn)?;
        let root = WindowId::from_le_bytes(conn)?;
        let subpixel_order = Subpixel::from_le_bytes(conn)?;
        drop(conn.drain(2 + 4 + 4)?);

        Ok(Self {
            status,
            new_timestamp,
            new_configuration_timestamp,
            root,
            subpixel_order,
        })
    }
}

impl_xreply!(SetScreenConfig);

/*
┌───
    RRGetScreenInfo
      ▶
        1       1                       Reply
        1       CARD8                   set of Rotations
        2       CARD16                  sequence number
        4       0                       reply length
        4       WINDOW                  root window
        4       TIMESTAMP               timestamp
        4       TIMESTAMP               config timestamp
        2       CARD16                  number of SCREENSIZE following
        2       SIZEID                  current size index
        2       ROTATION                current rotation and reflection
        2       CARD16                  current rate (added in version 1.1)
        2       CARD16                  length of rate info (number of CARD16s)
        2       CARD16                  pad

        SCREENSIZE
        2       CARD16                  width in pixels
        2       CARD16                  height in pixels
        2       CARD16                  width in millimeters
        2       CARD16                  height in millimeters

        REFRESH
        2       CARD16                  number of rates (n)
        2n      CARD16                  rates
└───
*/

#[derive(Debug, Clone, Copy)]
pub struct ScreenSize {
    pub width_in_pixels: u16,
    pub height_in_pixels: u16,
    pub width_in_millimeters: u16,
    pub height_in_millimeters: u16,
}

impl FromLeBytes for ScreenSize {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let width_in_pixels = conn.read_le_u16()?;
        let height_in_pixels = conn.read_le_u16()?;
        let width_in_millimeters = conn.read_le_u16()?;
        let height_in_millimeters = conn.read_le_u16()?;

        Ok(Self {
            width_in_pixels,
            height_in_pixels,
            width_in_millimeters,
            height_in_millimeters,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Refresh {
    pub rates: Vec<u16>,
}

impl FromLeBytes for Refresh {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let n = conn.read_le_u16()?;
        let rates = read_vec!(n, conn.read_le_u16()?);

        Ok(Self { rates })
    }
}

#[derive(Debug, Clone)]
pub struct GetScreenInfo {
    /// Set of rotations and reflections supported by the screen
    pub supported_rotations: PossibleRotation,

    /// The root window of the screen
    pub root: WindowId,

    /// indicates when the screen configuration information last changed:
    /// requests to set the screen will fail unless the timestamp indicates that the information
    /// the client is using is up to date, to ensure clients can be well behaved in the
    /// face of race conditions.
    pub timestamp: Timestamp,

    /// Indicates when the configuration was last set.
    pub config_timestamp: Timestamp,

    /// Indicates which size is active
    pub current_size: ScreenSize,

    pub current_rotation_and_reflection: PossibleRotation, // TODO: Split type in two?
    pub current_rate: u16,
    pub screen_sizes: Vec<ScreenSize>,
    pub refresh_rates: Vec<Refresh>,
}

impl FromLeBytes for GetScreenInfo {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let supported_rotations = PossibleRotation::from(conn.read_u8()? as u16);
        let _sequence_number = conn.read_le_u16()?;
        let _reply_length = conn.read_le_u32()?;
        let root = WindowId::from_le_bytes(conn)?;
        let timestamp = Timestamp::from_le_bytes(conn)?;
        let config_timestamp = Timestamp::from_le_bytes(conn)?;
        let no_of_screensize = conn.read_le_u16()?;
        let current_size_index = conn.read_le_u16()?;
        let current_rotation_and_reflection = PossibleRotation::from(conn.read_le_u16()?);
        let current_rate = conn.read_le_u16()?;
        let _no_of_rateinfo_total = conn.read_le_u16()?;
        let _pad = conn.read_le_u16()?;

        let screen_sizes = read_vec!(no_of_screensize, ScreenSize::from_le_bytes(conn)?);

        // no_of_screensize is correct here
        let refresh_rates = read_vec!(no_of_screensize, Refresh::from_le_bytes(conn)?);

        // HACK: idk why it's needed, there are extra two bytes at the end
        drop(conn.drain(2)?);

        Ok(Self {
            supported_rotations,
            root,
            timestamp,
            config_timestamp,
            current_size: screen_sizes[current_size_index as usize],
            current_rotation_and_reflection,
            current_rate,
            screen_sizes,
            refresh_rates,
        })
    }
}

impl_xreply!(GetScreenInfo);

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
    SetScreenConfig(SetScreenConfig),
    GetScreenInfo(GetScreenInfo),
    GetCrtcInfo(GetCrtcInfo),
    GetMonitors(GetMonitors),
}

#[derive(Debug, Clone, Copy)]
pub enum ReplyType {
    QueryVersion,
    SetScreenConfig,
    GetScreenInfo,
    GetCrtcInfo,
    GetMonitors,
}
