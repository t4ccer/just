pub mod bin_parse {
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

pub fn pad(e: usize) -> usize {
    (4 - (e % 4)) % 4
}

pub fn display_maybe_utf8(buf: &[u8]) -> String {
    if let Ok(utf8) = std::str::from_utf8(buf) {
        utf8.to_string()
    } else {
        format!("{:?}", buf)
    }
}
