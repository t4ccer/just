use crate::{connection::XConnection, error::Error, SequenceNumber};
use std::{fmt, mem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SomeError {
    IdChoice(XIdChoiceError),
    Request(XRequestError),
    Value(XValueError),
    Window(XWindowError),
    Pixmap(XPixmapError),
    Atom(XAtomError),
    Cursor(XCursorError),
    Font(XFontError),
    Match(XMatchError),
    Drawable(XDrawableError),
    Access(XAccessError),
    Alloc(XAllocError),
    Colormap(XColormapError),
    GContext(XGContextError),
    IDChoice(XIDChoiceError),
    Name(XNameError),
    Length(XLengthError),
    Implementation(XImplementationError),
}

impl SomeError {
    /// Bytes must start after error code, i.e. from third byte counting from one
    pub fn from_le_bytes(conn: &mut XConnection, error_code: u8) -> Result<Self, Error> {
        let mut raw = [0u8; 32];
        raw[0] = 0;
        raw[1] = error_code;
        conn.read_exact(&mut raw[2..])?;
        let generic = XGenericError::from_le_bytes(raw);
        match error_code {
            1 => Ok(Self::Request(XRequestError { generic })),
            2 => Ok(Self::Value(XValueError { generic })),
            3 => Ok(Self::Window(XWindowError { generic })),
            4 => Ok(Self::Pixmap(XPixmapError { generic })),
            5 => Ok(Self::Atom(XAtomError { generic })),
            6 => Ok(Self::Cursor(XCursorError { generic })),
            7 => Ok(Self::Font(XFontError { generic })),
            8 => Ok(Self::Match(XMatchError { generic })),
            9 => Ok(Self::Drawable(XDrawableError { generic })),
            10 => Ok(Self::Access(XAccessError { generic })),
            11 => Ok(Self::Alloc(XAllocError { generic })),
            12 => Ok(Self::Colormap(XColormapError { generic })),
            13 => Ok(Self::GContext(XGContextError { generic })),
            14 => Ok(Self::IDChoice(XIDChoiceError { generic })),
            15 => Ok(Self::Name(XNameError { generic })),
            16 => Ok(Self::Length(XLengthError { generic })),
            17 => Ok(Self::Implementation(XImplementationError { generic })),
            invalid => Err(Error::UnknownErrorCode(invalid)),
        }
    }

    pub fn sequence_number(&self) -> SequenceNumber {
        match self {
            SomeError::IdChoice(err) => err.sequence_number(),
            SomeError::Request(err) => err.sequence_number(),
            SomeError::Value(err) => err.sequence_number(),
            SomeError::Window(err) => err.sequence_number(),
            SomeError::Pixmap(err) => err.sequence_number(),
            SomeError::Atom(err) => err.sequence_number(),
            SomeError::Cursor(err) => err.sequence_number(),
            SomeError::Font(err) => err.sequence_number(),
            SomeError::Match(err) => err.sequence_number(),
            SomeError::Drawable(err) => err.sequence_number(),
            SomeError::Access(err) => err.sequence_number(),
            SomeError::Alloc(err) => err.sequence_number(),
            SomeError::Colormap(err) => err.sequence_number(),
            SomeError::GContext(err) => err.sequence_number(),
            SomeError::IDChoice(err) => err.sequence_number(),
            SomeError::Name(err) => err.sequence_number(),
            SomeError::Length(err) => err.sequence_number(),
            SomeError::Implementation(err) => err.sequence_number(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct XGenericError {
    error: u8,
    code: u8,
    sequence_number: u16,
    generic_value: u32,
    minor_opcode: u16,
    major_opcode: u8,
    _pad: [u8; 21],
}

impl XGenericError {
    fn from_le_bytes(raw: [u8; 32]) -> Self {
        unsafe { mem::transmute(raw) }
    }
}

macro_rules! impl_x_error_base {
    ($name:ident, $($rest:tt)*) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $name {
            generic: XGenericError,
        }

        impl $name {
            $($rest)*
        }
    };
}

macro_rules! impl_x_error {
    ($name:ident) => {
        impl_x_error!($name,);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    .field("sequence_number", &self.generic.sequence_number)
                    .field("minor_opcode", &self.generic.minor_opcode)
                    .field("major_opcode", &self.generic.major_opcode)
                    .finish()
            }
        }
    };

    ($name:ident, $generic:ident) => {
        impl_x_error! { $name,
            pub fn $generic(&self) -> u32 {
                self.generic.generic_value
            }
        }

        impl fmt::Debug for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.debug_struct(stringify!($name))
                        .field("sequence_number", &self.generic.sequence_number)
                        .field("minor_opcode", &self.generic.minor_opcode)
                        .field("major_opcode", &self.generic.major_opcode)
                        .field(stringify!($generic), &self.generic.generic_value)
                        .finish()
                }
        }
    };

    ($name:ident, $($rest:tt)*) => {
        impl_x_error_base! { $name,
            pub fn sequence_number(&self) -> SequenceNumber {
                SequenceNumber {
                    value: self.generic.sequence_number
                }
            }

            pub fn minor_opcode(&self) -> u16 {
                self.generic.minor_opcode
            }

            pub fn major_opcode(&self) -> u8 {
                self.generic.major_opcode
            }

            $($rest)*
        }
    };
}

impl_x_error!(XIdChoiceError, bad_resource_id);
impl_x_error!(XRequestError);
impl_x_error!(XValueError, bad_value);
impl_x_error!(XWindowError, bad_resource_id);
impl_x_error!(XPixmapError, bad_resource_id);
impl_x_error!(XAtomError, bad_atom_id);
impl_x_error!(XCursorError, bad_resource_id);
impl_x_error!(XFontError, bad_resource_id);
impl_x_error!(XMatchError);
impl_x_error!(XDrawableError, bad_resource_id);
impl_x_error!(XAccessError);
impl_x_error!(XAllocError);
impl_x_error!(XColormapError, bad_resource_id);
impl_x_error!(XGContextError, bad_resource_id);
impl_x_error!(XIDChoiceError, bad_resource_id);
impl_x_error!(XNameError);
impl_x_error!(XLengthError);
impl_x_error!(XImplementationError);
