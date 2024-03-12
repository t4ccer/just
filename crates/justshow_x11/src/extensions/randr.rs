pub mod replies;
pub mod requests;

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
