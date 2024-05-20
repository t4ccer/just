use crate::{Event, Result, Vector2};

pub(crate) mod owned_bitmap;
pub(crate) mod shared_bitmap;
pub(crate) mod x11_mit_shm;

pub(crate) trait Backend {
    fn flush_window(&mut self) -> Result<()>;

    fn events(&mut self) -> Result<Vec<Event>>;

    fn resize(&mut self, size: Vector2<u32>) -> Result<()>;

    fn size(&self) -> Vector2<u32>;

    fn buf_mut(&mut self) -> &mut [u8];

    fn buf(&self) -> &[u8];
}
