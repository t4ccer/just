use crate::utils::impl_enum;

impl_enum! {
    #[repr(u16)]
    /// NOTE: rander extension calls it SUBPIXELORDER
    enum Subpixel {
        Unknown = 0,
        HorizontalRGB = 1,
        HorizontalBGR = 2,
        VerticalRGB = 3,
        VerticalBGR = 4,
        None = 5,
    }
}

#[derive(Debug)]
pub struct Fixed {
    inner: u32,
}

impl From<f32> for Fixed {
    fn from(value: f32) -> Self {
        Self {
            inner: (value * 65536.0) as u32,
        }
    }
}

impl From<Fixed> for f32 {
    fn from(value: Fixed) -> Self {
        value.inner as f32 / 65536.0
    }
}

#[derive(Debug)]
pub struct Transform {
    pub matrix: [[Fixed; 3]; 3],
}
