use crate::{utils::display_maybe_utf8, InitializeConnectionResponseRefused};
use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    InvalidXAuthFile(String),
    CouldNotReadXAuthFile(String, io::Error),
    InvalidDisplayEnv,
    InvalidResponse(&'static str),
    NoEnv(&'static str),
    IOError(io::Error),
    CouldNotOpenDisplay(InitializeConnectionResponseRefused),
    UnknownErrorCode(u8),
    CouldNotOpenUnixSocket(String, io::Error),
    CouldNotConnectTo(String),
    UnexpectedReply,
    InvalidEnum(&'static str, u64),
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
            Error::CouldNotReadXAuthFile(file_path, inner) => {
                write!(
                    f,
                    "Could not read Xauthority file '{}': {}",
                    file_path, inner
                )
            }
            Error::InvalidDisplayEnv => {
                write!(f, "Could not decode $DISPLAY environment variable")
            }
            Error::InvalidResponse(loc) => {
                write!(f, "Could not decode response from X server: {}", loc)
            }
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
            Error::UnexpectedReply => {
                write!(f, "Server sent reply in different format than expected")
            }
            Error::InvalidEnum(enum_name, invalid_value) => write!(
                f,
                "Server sent invalid enum '{}' value: {}",
                enum_name, invalid_value
            ),
        }
    }
}
