use crate::{error::Error, utils::bin_parse};
use std::{fmt::Display, fs, io::Read};

#[allow(dead_code)]
#[derive(Debug)]
pub struct XAuth {
    pub family: u16,
    pub address: Vec<u8>,
    pub seat: Vec<u8>,
    pub name: Vec<u8>,
    pub data: Vec<u8>,
}

impl XAuth {
    pub fn from_bytes(raw: &[u8]) -> Option<Self> {
        let (family, raw) = bin_parse::u16_be(raw)?;
        let (address, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (seat, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (name, raw) = bin_parse::sized_u16_be_vec(raw)?;
        let (data, raw) = bin_parse::sized_u16_be_vec(raw)?;

        (raw.is_empty()).then_some(Self {
            family,
            address,
            seat,
            name,
            data,
        })
    }

    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path> + Display + Clone,
    {
        let mut auth_file = fs::File::open(path.clone())
            .map_err(|err| (Error::CouldNotReadXAuthFile(path.to_string(), err)))?;
        let mut auth_raw = Vec::new();
        auth_file.read_to_end(&mut auth_raw)?;
        XAuth::from_bytes(&auth_raw).ok_or(Error::InvalidXAuthFile(path.to_string()))
    }

    fn home_path() -> Option<String> {
        let var = "HOME";
        let home = std::env::var(var).ok()?;
        Some(format!("{}/.Xauthority", home))
    }

    pub fn from_env() -> Result<Self, Error> {
        let var = "XAUTHORITY";
        let file_path = std::env::var(var).map_err(|_| Error::NoEnv(var));
        match file_path {
            Ok(file_path) => Self::from_file(file_path),
            Err(_) => {
                let file_path = Self::home_path().ok_or(Error::NoEnv(var))?;
                Self::from_file(file_path)
            }
        }
    }
}
