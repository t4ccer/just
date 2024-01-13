use crate::{error::Error, utils::bin_parse};
use std::{fs, io::Read};

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

        (raw.len() == 0).then_some(Self {
            family,
            address,
            seat,
            name,
            data,
        })
    }

    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<std::path::Path>,
    {
        let mut auth_file = fs::File::open(path)?;
        let mut auth_raw = Vec::new();
        auth_file.read_to_end(&mut auth_raw)?;
        XAuth::from_bytes(&auth_raw).ok_or(Error::InvalidXAuth)
    }

    pub fn from_env() -> Result<Self, Error> {
        let var = "XAUTHORITY";
        let file_path = std::env::var(var).map_err(|_| Error::NoEnv(var))?;
        Self::from_file(file_path)
    }
}
