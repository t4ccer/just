use crate::x11::InitializeConnectionResponseRefused;
use std::{fmt::Display, io};

use super::utils::display_maybe_utf8;

#[derive(Debug)]
pub enum Error {
    InvalidXAuthFile(String),
    InvalidDisplayEnv,
    InvalidResponse,
    NoEnv(&'static str),
    IOError(io::Error),
    CouldNotOpenDisplay(InitializeConnectionResponseRefused),
    UnknownErrorCode(u8),
    CouldNotOpenUnixSocket(String, io::Error),
    CouldNotConnectTo(String),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidXAuthFile(file_path) => {
                write!(f, "Could not decode Xauthority file '{}'", file_path)
            }
            Error::InvalidDisplayEnv => {
                write!(f, "Could not decode $DISPLAY environment variable")
            }
            Error::InvalidResponse => write!(f, "Could not decode response from X server"),
            Error::NoEnv(env_var) => write!(f, "Environment variable '{}' is not set", env_var),
            Error::IOError(inner) => write!(f, "Unexpected IO error: {}", inner),
            Error::CouldNotOpenDisplay(response) => write!(
                f,
                "Could not open connection to the server: {}",
                display_maybe_utf8(&response.reason)
            ),
            Error::UnknownErrorCode(error_code) => write!(
                f,
                "Client received invalid error code '{}' from X server",
                error_code
            ),
            Error::CouldNotOpenUnixSocket(socket_path, inner) => {
                write!(f, "Could not open unix socket '{}': {}", socket_path, inner)
            }
            Error::CouldNotConnectTo(display) => {
                write!(f, "Could not connect to display '{}'", display)
            }
        }
    }
}
