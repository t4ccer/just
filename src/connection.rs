use crate::{error::Error, requests::XRequest, XResponse};
use std::{
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    os::unix::net::UnixStream,
    str::FromStr,
};

pub struct XConnectionRead {
    pub(crate) inner: BufReader<Box<dyn Read>>,
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

pub struct XConnection {
    pub(crate) read_end: XConnectionRead,
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
