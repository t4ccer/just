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

#[macro_export]
macro_rules! bitmask {
    (#[repr($inner:ident)] $(#[$name_attr_post:meta])* bitmask $ty:ident { $( $(#[$field_attr:meta])* $key:ident = $value:literal,)* }) => {
        $(#[$name_attr_post])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $ty {
            value: $inner,
        }

        #[automatically_derived]
        impl ::std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                let fields: &[&'static str] = &[ $( stringify!($key),)* ];

                write!(f, "{:#x} (", self.value)?;
                for (idx, field) in [ $( Self::$key,)* ]
                    .into_iter()
                    .enumerate()
                    .filter(|(_, field)| self.has(*field)) {
                        if idx != 0 {
                            write!(f, " | ")?;
                        }
                        write!(f, "{}", fields[idx])?;
                    }
                write!(f, ")")?;
                Ok(())
            }
        }

        #[automatically_derived]
        impl $ty {
            pub const EMPTY_MASK: Self = Self { value: 0 };
            $($(#[$field_attr])* pub const $key: Self = Self { value: $value };)*
        }

        impl $ty {
            #[inline(always)]
            pub fn has(self, flag: Self) -> bool {
                (self.value & flag.value) == flag.value
            }

            #[inline(always)]
            pub fn raw(self) -> $inner {
                self.value
            }
        }

        #[automatically_derived]
        impl ::std::ops::BitOr for $ty {
            type Output = Self;

            #[inline(always)]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    value: self.value | rhs.value,
                }
            }
        }

        #[automatically_derived]
        impl ::std::ops::BitOrAssign for $ty {
            #[inline(always)]
            fn bitor_assign(&mut self, rhs: Self) {
                self.value |= rhs.value
            }
        }

        #[automatically_derived]
        impl ::std::ops::BitAnd for $ty {
            type Output = Self;

            #[inline(always)]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self {
                    value: self.value & rhs.value,
                }
            }
        }

        #[automatically_derived]
        impl ::std::convert::From<$ty> for $inner {
            #[inline(always)]
            fn from(val: $ty) -> Self {
                val.value
            }
        }

        #[automatically_derived]
        impl ::std::convert::From<$inner> for $ty {
            #[inline(always)]
            fn from(value: $inner) -> Self {
                Self { value }
            }
        }

        #[automatically_derived]
        impl $crate::FromLeBytes for $ty {
            #[inline]
            fn from_le_bytes(conn: &mut $crate::connection::XConnection) -> Result<Self, $crate::error::Error> {
                let value: $inner = $crate::FromLeBytes::from_le_bytes(conn)?;
                Ok(Self {value})
            }
        }
    };
    ($($stuff:tt)*) => {
        compile_error!("Bitmask must contain #[repr(type)] at the very top (even above doc comments)");
    }
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

/// Create an enum with basic implementations like going from underlying representation to the enum
macro_rules! impl_enum {
    (#[repr($inner:ident)] $(#[$name_attr_post:meta])*  enum $name:ident { $( $(#[$field_attr:meta])* $key:ident = $value:literal,)* }) => {
        $(#[$name_attr_post])*
        #[repr($inner)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $($(#[$field_attr])* $key = $value,)*
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
        impl crate::FromLeBytes for $name {
            #[inline]
            fn from_le_bytes(conn: &mut crate::XConnection) -> Result<Self, crate::error::Error> {
                let inner: $inner = crate::FromLeBytes::from_le_bytes(conn)?;
                Self::try_from(inner)
                    .map_err(|invalid| crate::error::Error::InvalidEnum(stringify!(Self), invalid as u64))
            }
        }

        #[automatically_derived]
        #[allow(dead_code)]
        impl $name {
            #[inline]
            fn to_le_bytes(self) -> [u8; <$inner as crate::utils::BytesSize>::SIZE] {
                (self as $inner).to_le_bytes()
            }
        }
    };
    ($($stuff:tt)*) => {
        compile_error!("Enum must contain #[repr(type)] at the very top (even above doc comments)");
    }
}

pub(crate) use impl_enum;

macro_rules! impl_resource_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct $name(crate::ResourceId);

        #[allow(dead_code)]
        impl $name {
            pub fn id(self) -> crate::ResourceId {
                self.0
            }

            pub fn to_le_bytes(self) -> [u8; 4] {
                let raw: u32 = self.into();
                raw.to_le_bytes()
            }

            pub fn unchecked_from(value: u32) -> Self {
                Self(crate::ResourceId { value })
            }
        }

        impl From<$name> for u32 {
            fn from(value: $name) -> u32 {
                value.0.value()
            }
        }

        impl From<u32> for $name {
            fn from(value: u32) -> Self {
                Self(crate::ResourceId::from(value))
            }
        }

        impl From<crate::ResourceId> for $name {
            fn from(value: crate::ResourceId) -> Self {
                Self(value)
            }
        }

        impl crate::FromLeBytes for $name {
            fn from_le_bytes(conn: &mut crate::XConnection) -> Result<Self, crate::error::Error> {
                let inner: u32 = crate::FromLeBytes::from_le_bytes(conn)?;
                Ok(Self::from(inner))
            }
        }

        crate::requests::impl_value!($name into);
    };
}

pub(crate) use impl_resource_id;
