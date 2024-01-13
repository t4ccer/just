use std::{
    fmt::Display,
    fs,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    mem,
    os::unix::net::UnixStream,
    str::FromStr,
};

mod bin_parse {
    #[inline]
    pub fn u16_be(raw: &[u8]) -> Option<(u16, &[u8])> {
        let (bytes, raw) = raw.split_at(2);
        let res = u16::from_be_bytes(bytes.try_into().ok()?);

        Some((res, raw))
    }

    /// Vector with size as u16 big endian before elements
    #[inline]
    pub fn sized_u16_be_vec(raw: &[u8]) -> Option<(Vec<u8>, &[u8])> {
        let (len, raw) = u16_be(raw)?;
        let elements = raw.get(0..(len as usize))?.to_vec();
        Some((elements, &raw[(len as usize)..]))
    }
}

#[derive(Debug)]
enum Error {
    InvalidXAuth,
    InvalidDisplayEnv,
    InvalidResponse,
    NoEnv(&'static str),
    IOError(io::Error),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

struct XConnectionRead {
    inner: BufReader<Box<dyn Read>>,
}

impl XConnectionRead {
    pub fn new(inner: BufReader<Box<dyn Read>>) -> Self {
        Self { inner }
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.inner.read_exact(buf)
    }

    pub fn read_n_bytes(&mut self, mut to_read: usize) -> io::Result<Vec<u8>> {
        let to_read_init = to_read;
        let mut res = Vec::with_capacity(to_read_init);

        while to_read != 0 {
            let available_data = self.inner.buffer().len();

            if to_read > available_data {
                res.extend(&self.inner.buffer()[..]);
                self.inner.consume(available_data);
                self.inner.fill_buf()?;
                to_read -= available_data;
            } else {
                res.extend(&self.inner.buffer()[..to_read]);
                self.inner.consume(to_read);
                to_read = 0;
            }
        }

        Ok(res)
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        let mut res = [0u8; 1];
        self.inner.read_exact(&mut res)?;
        Ok(res[0])
    }

    pub fn read_u16_be(&mut self) -> io::Result<u16> {
        let mut res = [0u8; 2];
        self.inner.read_exact(&mut res)?;
        Ok(u16::from_be_bytes(res))
    }

    pub fn read_u32_be(&mut self) -> io::Result<u32> {
        let mut res = [0u8; 4];
        self.inner.read_exact(&mut res)?;
        Ok(u32::from_be_bytes(res))
    }

    pub fn read_many<T, E>(
        &mut self,
        len: usize,
        parser: impl Fn(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E> {
        (0..len)
            .map(|_| parser(self))
            .collect::<Result<Vec<T>, E>>()
    }
}

struct XConnection {
    read_end: XConnectionRead,
    write_end: BufWriter<Box<dyn Write>>,
}

impl TryFrom<UnixStream> for XConnection {
    type Error = Error;

    fn try_from(stream: UnixStream) -> Result<Self, Error> {
        Ok(Self {
            read_end: XConnectionRead::new(BufReader::new(Box::new(stream.try_clone()?))),
            write_end: BufWriter::new(Box::new(stream)),
        })
    }
}

impl XConnection {
    pub fn send_request(&mut self, request: &impl XRequest) -> Result<(), Error> {
        self.write_end.write_all(&request.to_be_bytes())?;
        self.write_end.flush()?;
        Ok(())
    }

    pub fn read_response<T>(&mut self) -> Result<T, Error>
    where
        T: XResponse,
    {
        T::from_be_bytes(&mut self.read_end)
    }
}

fn pad(e: usize) -> usize {
    (4 - (e % 4)) % 4
}

#[allow(dead_code)]
#[derive(Debug)]
struct XAuth {
    family: u16,
    address: Vec<u8>,
    seat: Vec<u8>,
    name: Vec<u8>,
    data: Vec<u8>,
}

impl XAuth {
    fn from_bytes(raw: &[u8]) -> Option<Self> {
        let (family, raw) = bin_parse::u16_be(raw)?;
        let (address, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (seat, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (name, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (data, raw) = bin_parse::sized_u16_be_vec(raw)?;

        (raw.len() == 0).then_some(Self {
            family,
            address,
            seat,
            name,
            data,
        })
    }

    fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut auth_file = fs::File::open(path)?;
        let mut auth_raw = Vec::new();
        auth_file.read_to_end(&mut auth_raw)?;
        XAuth::from_bytes(&auth_raw).ok_or(Error::InvalidXAuth)
    }

    fn from_env() -> Result<Self, Error> {
        let file_path = std::env::var("XAUTHORITY").map_err(|_| Error::NoEnv("XAUTHORITY"))?;
        Self::from_file(file_path)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct DisplayVar {
    hostname: String,
    display_sequence: u32,
    screen: Option<u32>,
}

impl FromStr for DisplayVar {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hostname, s) = s.split_once(|c| c == ':').ok_or(Error::InvalidDisplayEnv)?;
        let (display_sequence, screen) = match s.split_once(|c| c == '.') {
            Some((display_sequence, screen)) => {
                let display_sequence = display_sequence
                    .parse::<u32>()
                    .map_err(|_| Error::InvalidDisplayEnv)?;
                let screen = Some(
                    screen
                        .parse::<u32>()
                        .map_err(|_| Error::InvalidDisplayEnv)?,
                );
                (display_sequence, screen)
            }
            None => {
                let display_sequence = s.parse::<u32>().map_err(|_| Error::InvalidDisplayEnv)?;
                (display_sequence, None)
            }
        };

        Ok(Self {
            hostname: hostname.to_string(),
            display_sequence,
            screen,
        })
    }
}

impl DisplayVar {
    pub fn from_env() -> Result<Self, Error> {
        let value = std::env::var("DISPLAY").map_err(|_| Error::NoEnv("DISPLAY"))?;
        Self::from_str(&value)
    }
}

trait XRequest: Sized {
    fn to_be_bytes(&self) -> Vec<u8>;
}

trait XResponse: Sized {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error>;
}

#[derive(Debug)]
struct InitializeConnectionRequest {
    major_version: u16,
    minor_version: u16,
    authorization_protocol_name: Vec<u8>,
    authorization_protocol_data: Vec<u8>,
}

impl XRequest for InitializeConnectionRequest {
    fn to_be_bytes(&self) -> Vec<u8> {
        let n = self.authorization_protocol_name.len();
        let p = pad(n);
        let d = self.authorization_protocol_data.len();
        let q = pad(d);
        let mut bytes = Vec::<u8>::with_capacity(10 + n + p + d + q);

        bytes.extend(b"B\0");
        bytes.extend(self.major_version.to_be_bytes());
        bytes.extend(self.minor_version.to_be_bytes());
        bytes.extend((n as u16).to_be_bytes());
        bytes.extend((d as u16).to_be_bytes());
        bytes.extend([0u8; 2]); // unused
        bytes.extend(&self.authorization_protocol_name);
        bytes.extend(vec![0u8; p]); // unused, pad
        bytes.extend(&self.authorization_protocol_data);
        bytes.extend(vec![0u8; q]); // unused, pad

        bytes
    }
}

#[derive(Debug)]
enum InitializeConnectionResponse {
    Refused(InitializeConnectionResponseRefused),
    Success(InitializeConnectionResponseSuccess),
}

impl XResponse for InitializeConnectionResponse {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let response_code = conn.read_u8()?;
        match response_code {
            0 => Ok(Self::Refused(
                InitializeConnectionResponseRefused::from_be_bytes(conn)?,
            )),
            1 => Ok(Self::Success(
                InitializeConnectionResponseSuccess::from_be_bytes(conn)?,
            )),
            2 => todo!("Authenticate"),
            _ => Err(Error::InvalidResponse),
        }
    }
}

#[derive(Debug)]
struct InitializeConnectionResponseRefused {
    protocol_major_version: u16,
    protocol_minor_version: u16,
    reason: Vec<u8>,
}

impl XResponse for InitializeConnectionResponseRefused {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let reason_length = conn.read_u8()?;
        let protocol_major_version = conn.read_u16_be()?;
        let protocol_minor_version = conn.read_u16_be()?;
        let _ = conn.read_u16_be()?;
        let reason = conn.read_n_bytes(reason_length as usize)?;
        let _pad = conn.read_n_bytes(pad(reason_length as usize))?;
        Ok(Self {
            protocol_major_version,
            protocol_minor_version,
            reason,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Window(u32);

#[allow(dead_code)]
#[derive(Debug)]
#[repr(u8)]
enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PseudoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl TryFrom<u8> for VisualClass {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 5 {
            return Err(value);
        }
        Ok(unsafe { mem::transmute(value) })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Visual {
    id: u32,
    class: VisualClass,
    bits_per_rgb_value: u8,
    colormap_entries: u16,
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
}

impl Visual {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let id = conn.read_u32_be()?;
        let class = VisualClass::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let bits_per_rgb_value = conn.read_u8()?;
        let colormap_entries = conn.read_u16_be()?;
        let red_mask = conn.read_u32_be()?;
        let green_mask = conn.read_u32_be()?;
        let blue_mask = conn.read_u32_be()?;
        let _unused = conn.read_u32_be()?;
        Ok(Self {
            id,
            class,
            bits_per_rgb_value,
            colormap_entries,
            red_mask,
            green_mask,
            blue_mask,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Depth {
    depth: u8,
    visuals: Vec<Visual>,
}

impl Depth {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _unused = conn.read_u8()?;
        let visuals_length = conn.read_u16_be()?;
        let _unused = conn.read_u32_be()?;
        let visuals = conn.read_many(visuals_length as usize, Visual::from_be_bytes)?;
        Ok(Self { depth, visuals })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(u8)]
enum BackingStore {
    NotUseful = 0,
    WhenMapped = 1,
    Always = 2,
}

impl TryFrom<u8> for BackingStore {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 2 {
            return Err(value);
        }

        Ok(unsafe { mem::transmute(value) })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Screen {
    root: Window,
    default_colormat: u32,
    white_pixel: u32,
    black_pixel: u32,
    current_input_masks: u32,
    width_in_pixels: u16,
    height_in_pixels: u16,
    width_in_millimeters: u16,
    height_in_millimeters: u16,
    min_installed_maps: u16,
    max_installed_maps: u16,
    root_visual: u32,
    backing_stores: BackingStore,
    save_unders: bool,
    root_depth: u8,
    allowed_depths: Vec<Depth>,
}

impl Screen {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let root = Window(conn.read_u32_be()?);
        let default_colormat = conn.read_u32_be()?;
        let white_pixel = conn.read_u32_be()?;
        let black_pixel = conn.read_u32_be()?;
        let current_input_masks = conn.read_u32_be()?;
        let width_in_pixels = conn.read_u16_be()?;
        let height_in_pixels = conn.read_u16_be()?;
        let width_in_millimeters = conn.read_u16_be()?;
        let height_in_millimeters = conn.read_u16_be()?;
        let min_installed_maps = conn.read_u16_be()?;
        let max_installed_maps = conn.read_u16_be()?;
        let root_visual = conn.read_u32_be()?;
        let backing_stores =
            BackingStore::try_from(conn.read_u8()?).map_err(|_| Error::InvalidResponse)?;
        let save_unders = conn.read_u8()? == 1;
        let root_depth = conn.read_u8()?;
        let allowed_depths_lenght = conn.read_u8()?;
        let allowed_depths =
            conn.read_many(allowed_depths_lenght as usize, Depth::from_be_bytes)?;

        Ok(Self {
            root,
            default_colormat,
            white_pixel,
            black_pixel,
            current_input_masks,
            width_in_pixels,
            height_in_pixels,
            width_in_millimeters,
            height_in_millimeters,
            min_installed_maps,
            max_installed_maps,
            root_visual,
            backing_stores,
            save_unders,
            root_depth,
            allowed_depths,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Format {
    depth: u8,
    bits_per_pixel: u8,
    scanline_pad: u8,
}

impl Format {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let mut format = [0u8; 8];
        conn.read_exact(&mut format)?;
        Ok(Format {
            depth: format[0],
            bits_per_pixel: format[1],
            scanline_pad: format[2],
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct InitializeConnectionResponseSuccess {
    protocol_major_version: u16,
    protocol_minor_version: u16,
    release_number: u32,
    resource_id_base: u32,
    resource_id_mask: u32,
    motion_buffer_size: u32,
    maximum_request_length: u16,
    image_byte_order: u8,
    bitmap_format_byte_order: u8,
    bitmap_format_scanline_unit: u8,
    bitmap_format_scanline_pad: u8,
    min_keycode: u8,
    max_keycode: u8,
    vendor: Vec<u8>,
    pixmap_formats: Vec<Format>,
    screens: Vec<Screen>,
}

impl XResponse for InitializeConnectionResponseSuccess {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let _unused = conn.read_u8()?;
        let protocol_major_version = conn.read_u16_be()?;
        let protocol_minor_version = conn.read_u16_be()?;
        let _ = conn.read_u16_be()?;
        let release_number = conn.read_u32_be()?;
        let resource_id_base = conn.read_u32_be()?;
        let resource_id_mask = conn.read_u32_be()?;
        let motion_buffer_size = conn.read_u32_be()?;
        let vendor_length = conn.read_u16_be()?;
        let maximum_request_length = conn.read_u16_be()?;
        let screens_length = conn.read_u8()?;
        let formats_length = conn.read_u8()?;
        let image_byte_order = conn.read_u8()?;
        let bitmap_format_byte_order = conn.read_u8()?;
        let bitmap_format_scanline_unit = conn.read_u8()?;
        let bitmap_format_scanline_pad = conn.read_u8()?;
        let min_keycode = conn.read_u8()?;
        let max_keycode = conn.read_u8()?;
        let _unused = conn.read_u32_be()?;
        let vendor = conn.read_n_bytes(vendor_length as usize)?;
        let _pad = conn.read_n_bytes(pad(vendor_length as usize))?;
        let pixmap_formats = conn.read_many(formats_length as usize, Format::from_be_bytes)?;
        let screens = conn.read_many(screens_length as usize, Screen::from_be_bytes)?;

        Ok(Self {
            protocol_major_version,
            protocol_minor_version,
            release_number,
            resource_id_base,
            resource_id_mask,
            motion_buffer_size,
            maximum_request_length,
            image_byte_order,
            bitmap_format_byte_order,
            bitmap_format_scanline_unit,
            bitmap_format_scanline_pad,
            min_keycode,
            max_keycode,
            vendor,
            pixmap_formats,
            screens,
        })
    }
}

fn display_maybe_utf8(buf: &[u8]) -> String {
    if let Ok(utf8) = std::str::from_utf8(buf) {
        utf8.to_string()
    } else {
        format!("{:?}", buf)
    }
}

impl Display for InitializeConnectionResponseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Initialize Connection Response Refused: Protocol: {}.{}. Reason: {}",
            self.protocol_major_version,
            self.protocol_minor_version,
            display_maybe_utf8(&self.reason)
        )
    }
}

fn go() -> Result<(), Error> {
    let auth = XAuth::from_env()?;
    let display = DisplayVar::from_env()?;

    if &display.hostname != "" {
        eprintln!("Unsupported hostname: {}", display.hostname);
        return Ok(());
    }

    let socket_path = format!("/tmp/.X11-unix/X{}", display.display_sequence);
    let stream = UnixStream::connect(socket_path)?;
    let mut conn = XConnection::try_from(stream)?;

    let init = InitializeConnectionRequest {
        major_version: 11,
        minor_version: 0,
        authorization_protocol_name: auth.name,
        authorization_protocol_data: auth.data,
    };

    conn.send_request(&init)?;
    let response = conn.read_response::<InitializeConnectionResponse>()?;

    match &response {
        InitializeConnectionResponse::Refused(response) => {
            eprintln!("{}", response);
        }
        InitializeConnectionResponse::Success(response) => {
            eprintln!("{:#?}", response);
            eprintln!("Vendor: {}", display_maybe_utf8(&response.vendor));
        }
    }

    Ok(())
}

fn main() {
    go().unwrap();
}
