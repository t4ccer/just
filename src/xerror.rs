use crate::{connection::XConnectionRead, error::Error, XResponse};

#[derive(Debug, Clone, Copy)]
pub enum XError {
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

impl XResponse for XError {
    fn from_be_bytes(conn: &mut XConnectionRead) -> Result<Self, Error> {
        let mut raw = [0u8; 32];
        conn.read_exact(&mut raw)?;
        assert!(raw[0] == 0);
        let generic = XGenericError::from_be_bytes(&raw);
        match raw[1] {
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
}

#[derive(Debug, Clone, Copy)]
pub struct XGenericError {
    pub sequence_number: u16,
    pub generic_value: u32,
    pub minor_opcode: u16,
    pub major_opcode: u8,
}

impl XGenericError {
    fn from_be_bytes(raw: &[u8; 32]) -> Self {
        Self {
            sequence_number: u16::from_be_bytes([raw[2], raw[3]]),
            generic_value: u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]),
            minor_opcode: u16::from_be_bytes([raw[8], raw[9]]),
            major_opcode: raw[10],
        }
    }
}

macro_rules! impl_x_error {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name {
            pub generic: XGenericError,
        }
    };

    ($name:ident, $generic:ident) => {
        impl_x_error!($name);

        impl $name {
            pub fn $generic(&self) -> u32 {
                self.generic.generic_value
            }
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
