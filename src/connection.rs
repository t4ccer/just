use crate::{error::Error, requests::XRequest, XResponse};
use std::{
    collections::{vec_deque::Drain, VecDeque},
    io::{self, BufWriter, Read, Write},
    os::unix::net::UnixStream,
    str::FromStr,
};

pub(crate) enum XConnectionReader {
    UnixStream(UnixStream),
}

impl Read for XConnectionReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            XConnectionReader::UnixStream(stream) => stream.read(buf),
        }
    }
}

pub struct XConnection {
    read_end: XConnectionReader,
    read_buf: VecDeque<u8>,
    write_end: BufWriter<Box<dyn Write>>,
}

impl TryFrom<UnixStream> for XConnection {
    type Error = Error;

    fn try_from(stream: UnixStream) -> Result<Self, Error> {
        let read_end = stream.try_clone()?;
        read_end.set_nonblocking(true)?;

        let write_end = stream;

        Ok(Self {
            read_end: XConnectionReader::UnixStream(read_end),
            write_end: BufWriter::new(Box::new(write_end)),
            read_buf: VecDeque::new(),
        })
    }
}

impl XConnection {
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

    pub(crate) fn read_be_u16(&mut self) -> Result<u16, Error> {
        let mut buf = self.drain(2)?;
        let mut ret: u16 = 0;
        ret += buf.next().unwrap() as u16;
        ret <<= 8;
        ret += buf.next().unwrap() as u16;

        debug_assert!(buf.next().is_none());

        Ok(ret)
    }

    pub(crate) fn read_be_i16(&mut self) -> Result<i16, Error> {
        let raw = self.read_be_u16()?;
        Ok(i16::from_be_bytes(raw.to_be_bytes()))
    }

    pub(crate) fn read_be_u32(&mut self) -> Result<u32, Error> {
        let mut buf = self.drain(4)?;
        let mut ret: u32 = 0;
        ret += buf.next().unwrap() as u32;
        ret <<= 8;
        ret += buf.next().unwrap() as u32;
        ret <<= 8;
        ret += buf.next().unwrap() as u32;
        ret <<= 8;
        ret += buf.next().unwrap() as u32;

        debug_assert!(buf.next().is_none());

        Ok(ret)
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
        for (idx, elem) in self.drain(buf.len())?.into_iter().enumerate() {
            buf[idx] = elem;
        }

        Ok(())
    }

    pub(crate) fn peek(&mut self, index: usize) -> Result<u8, Error> {
        self.ensure_buffer_size(1)?;
        Ok(*self.read_buf.get(index).unwrap())
    }

    pub(crate) fn send_request<R: XRequest>(&mut self, request: &R) -> Result<(), Error> {
        request.to_be_bytes(&mut self.write_end)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.write_end.flush()?;
        Ok(())
    }

    pub fn read_expected_response<T>(&mut self) -> Result<T, Error>
    where
        T: XResponse,
    {
        T::from_be_bytes(self)
    }

    pub fn from_env() -> Result<Self, Error> {
        let display = DisplayVar::from_env()?;

        if &display.hostname != "" {
            eprintln!("Unsupported hostname: {}", display.hostname);
            todo!()
        }

        let socket_path = format!("/tmp/.X11-unix/X{}", display.display_sequence);
        let stream = UnixStream::connect(socket_path)?;
        Self::try_from(stream)
    }

    /// `true` if read any new data
    fn fill_buf_nonblocking(&mut self) -> Result<bool, Error> {
        let mut buf = vec![0u8; 0x1000];
        match self.read_end.read(&mut buf) {
            Ok(n) => {
                self.read_buf.extend(&buf[0..n]);
                Ok(true)
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(false),
            Err(err) => Err(err)?,
        }
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
        let var = "DISPLAY";
        let value = std::env::var(var).map_err(|_| Error::NoEnv(var))?;
        Self::from_str(&value)
    }
}
