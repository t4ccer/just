pub(crate) mod bin_parse {
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

pub(crate) fn pad(e: usize) -> usize {
    (4 - (e % 4)) % 4
}

pub(crate) fn display_maybe_utf8(buf: &[u8]) -> String {
    if let Ok(utf8) = std::str::from_utf8(buf) {
        utf8.to_string()
    } else {
        format!("{:?}", buf)
    }
}

macro_rules! bitmask {
    (#[repr($inner:ident)] bitmask $ty:ident { $($key:ident = $value:literal,)* }) => {
        #[repr(transparent)]
        pub struct $ty {
            value: $inner,
        }

        #[automatically_derived]
        impl $ty {
            $(pub const $key: Self = Self { value: $value };)*
        }

        #[automatically_derived]
        impl ::std::ops::BitOr for $ty {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    value: self.value | rhs.value,
                }
            }
        }

        #[automatically_derived]
        impl ::std::ops::BitOrAssign for $ty {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.value |= rhs.value
            }
        }

        #[automatically_derived]
        impl ::std::convert::From<$ty> for $inner {
            #[inline]
            fn from(val: $ty) -> Self {
                val.value
            }
        }
    };
}
pub(crate) use bitmask;

pub(crate) trait BytesSize {
    const SIZE: usize;
}

impl BytesSize for u8 {
    const SIZE: usize = 1;
}

impl BytesSize for u16 {
    const SIZE: usize = 2;
}

impl BytesSize for u32 {
    const SIZE: usize = 4;
}

// TODO: Support comments on enum constructors
/// Create an enum with basic implementations like going from underlying representation to the enum
macro_rules! impl_enum {
    (#[repr($inner:ident)] enum $name:ident { $($key:ident = $value:literal,)* }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr($inner)]
        pub enum $name {
            $($key = $value,)*
        }

        #[automatically_derived]
        impl ::std::convert::TryFrom<$inner> for $name {
            type Error = $inner;

            #[inline]
            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok(Self::$key),)*
                    _ => Err(value),
                }
            }
        }

        #[automatically_derived]
        impl $name {
            #[inline]
            fn to_le_bytes(self) -> [u8; <$inner as crate::utils::BytesSize>::SIZE] {
                (self as $inner).to_le_bytes()
            }
        }
    };
}

pub(crate) use impl_enum;
