use std::io;

#[derive(Debug)]
pub enum Error {
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
