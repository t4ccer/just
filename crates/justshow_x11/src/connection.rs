use crate::{
    error::Error,
    requests::{XExtensionRequest, XRequest},
};
use std::{
    collections::{vec_deque::Drain, VecDeque},
    fmt::Display,
    io::{self, BufWriter, Read, Write},
    os::unix::net::UnixStream,
    str::FromStr,
};

pub(crate) enum XConnectionReader {
    UnixStream(UnixStream),
    #[cfg(test)]
    Empty,
}

impl Read for XConnectionReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            XConnectionReader::UnixStream(stream) => stream.read(buf),
            #[cfg(test)]
            XConnectionReader::Empty => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF")),
        }
    }
}

// We need non-blocking socket for reading so we have this wrapper to block on writes
struct BlockingWriter<W> {
    inner: W,
}

impl<W> BlockingWriter<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }

    fn do_blocking<T>(&mut self, f: impl Fn(&mut Self) -> io::Result<T>) -> io::Result<T> {
        loop {
            match f(self) {
                Ok(ok) => return Ok(ok),
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                Err(err) => return Err(err),
            }
        }
    }
}

impl<W> Write for BlockingWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.do_blocking(|w| w.inner.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.do_blocking(|w| w.inner.flush())
    }
}

/// Connection to the X server
pub struct XConnection {
    read_end: XConnectionReader,
    read_buf: VecDeque<u8>,

    /// Shared temporary buffer used to read data from `read_end` connection before pushing them
    /// to `read_buf`
    fill_buf: Box<[u8]>,

    write_end: BlockingWriter<BufWriter<Box<dyn Write>>>,
}

// Arbitrarly chosen
const FILL_BUFF_SIZE: usize = 0x1000;

impl TryFrom<UnixStream> for XConnection {
    type Error = Error;

    fn try_from(stream: UnixStream) -> Result<Self, Error> {
        stream.set_nonblocking(true)?;

        let read_end = stream.try_clone()?;
        let write_end = stream;

        Ok(Self {
            read_end: XConnectionReader::UnixStream(read_end),
            write_end: BlockingWriter::new(BufWriter::new(Box::new(write_end))),
            read_buf: VecDeque::new(),
            fill_buf: vec![0u8; FILL_BUFF_SIZE].into_boxed_slice(),
        })
    }
}

pub(crate) enum ConnectionKind {
    UnixStream,
}

impl XConnection {
    #[cfg(test)]
    /// Create dummy connection with some pre-filled data, not connected to anything
    pub fn dummy(data: VecDeque<u8>) -> Self {
        Self {
            read_end: XConnectionReader::Empty,
            read_buf: data,
            fill_buf: vec![].into_boxed_slice(),
            write_end: BlockingWriter::new(BufWriter::new(Box::new(std::io::empty()))),
        }
    }

    pub(crate) fn has_unconsumed_data(&self) -> bool {
        !self.read_buf.is_empty()
    }

    pub(crate) fn kind(&self) -> ConnectionKind {
        match self.read_end {
            XConnectionReader::UnixStream(_) => ConnectionKind::UnixStream,
            #[cfg(test)]
            XConnectionReader::Empty => unimplemented!(),
        }
    }

    fn ensure_buffer_size(&mut self, size: usize) -> Result<(), Error> {
        while self.read_buf.len() < size {
            self.fill_buf_nonblocking()?;
        }
        Ok(())
    }

    pub(crate) fn drain(&mut self, len: usize) -> Result<Drain<'_, u8>, Error> {
        self.ensure_buffer_size(len)?;
        Ok(self.read_buf.drain(0..len))
    }

    pub(crate) fn read_n_bytes(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        Ok(self.drain(len)?.collect())
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8, Error> {
        self.ensure_buffer_size(1)?;
        Ok(self.read_buf.pop_front().unwrap())
    }

    pub(crate) fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(self.read_u8()? != 0)
    }

    pub(crate) fn read_le_u16(&mut self) -> Result<u16, Error> {
        let mut buf = self.drain(2)?;
        let b1 = buf.next().unwrap();
        let b2 = buf.next().unwrap();

        debug_assert!(buf.next().is_none());

        Ok(u16::from_le_bytes([b1, b2]))
    }

    pub(crate) fn read_le_i16(&mut self) -> Result<i16, Error> {
        let raw = self.read_le_u16()?;
        Ok(i16::from_le_bytes(raw.to_le_bytes()))
    }

    pub(crate) fn read_le_u32(&mut self) -> Result<u32, Error> {
        let mut buf = self.drain(4)?;
        let b1 = buf.next().unwrap();
        let b2 = buf.next().unwrap();
        let b3 = buf.next().unwrap();
        let b4 = buf.next().unwrap();

        debug_assert!(buf.next().is_none());

        Ok(u32::from_le_bytes([b1, b2, b3, b4]))
    }

    pub(crate) fn read_le_i32(&mut self) -> Result<i32, Error> {
        let raw = self.read_le_u32()?;
        Ok(i32::from_le_bytes(raw.to_le_bytes()))
    }

    pub(crate) fn read_many<T, E>(
        &mut self,
        len: usize,
        parser: impl Fn(&mut Self) -> Result<T, E>,
    ) -> Result<Vec<T>, E> {
        (0..len)
            .map(|_| parser(self))
            .collect::<Result<Vec<T>, E>>()
    }

    pub(crate) fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        for (idx, elem) in self.drain(buf.len())?.enumerate() {
            buf[idx] = elem;
        }

        Ok(())
    }

    pub(crate) fn peek(&mut self, index: usize) -> Result<u8, Error> {
        self.ensure_buffer_size(index)?;
        Ok(*self.read_buf.get(index).unwrap())
    }

    pub(crate) fn send_request<R: XRequest>(&mut self, request: &R) -> Result<(), Error> {
        request.to_le_bytes(&mut self.write_end)?;
        Ok(())
    }

    pub(crate) fn send_extension_request<R: XExtensionRequest>(
        &mut self,
        request: &R,
        major_opcode: u8,
    ) -> Result<(), Error> {
        self.write_end.write_all(&major_opcode.to_le_bytes())?;
        request.to_le_bytes(&mut self.write_end)?;
        Ok(())
    }

    /// Open a connection with details from `$DISPLAY` environment variable
    pub fn open() -> Result<Self, Error> {
        let display = DisplayVar::from_env()?;
        Self::with_display(display)
    }

    pub fn with_display(display: DisplayVar) -> Result<Self, Error> {
        if !display.hostname.is_empty() {
            return Err(Error::CouldNotConnectTo(display.to_string()));
        }

        // TODO: Use display.screen for something
        assert_eq!(
            display.screen, None,
            "Display screen is not implemented yet"
        );

        let socket_path = format!("/tmp/.X11-unix/X{}", display.display_sequence);
        let stream = UnixStream::connect(&socket_path)
            .map_err(|err| Error::CouldNotOpenUnixSocket(socket_path, err))?;
        Self::try_from(stream)
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.write_end.flush()?;
        Ok(())
    }

    /// `true` if read any new data
    pub(crate) fn fill_buf_nonblocking(&mut self) -> Result<bool, Error> {
        match self.read_end.read(&mut self.fill_buf) {
            Ok(n) => {
                self.read_buf.extend(&self.fill_buf[0..n]);
                Ok(true)
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(err) => Err(err)?,
        }
    }
}

#[derive(Debug)]
pub struct DisplayVar {
    pub hostname: String,
    pub display_sequence: u32,
    pub screen: Option<u32>,
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

impl Display for DisplayVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.hostname, self.display_sequence)?;
        if let Some(screen) = self.screen {
            write!(f, ".{}", screen)?;
        }
        Ok(())
    }
}

impl DisplayVar {
    /// Read and parse `$DISPLAY` environment variable
    pub fn from_env() -> Result<Self, Error> {
        let var = "DISPLAY";
        let value = std::env::var(var).map_err(|_| Error::NoEnv(var))?;
        Self::from_str(&value)
    }
}
