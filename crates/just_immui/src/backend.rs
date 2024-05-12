use crate::{Event, Result};

pub(crate) mod x11_mit_shm;

pub(crate) trait Backend {
    fn new(title: &str) -> Result<Self>
    where
        Self: Sized;

    fn flush_window(&mut self) -> Result<()>;

    fn events(&mut self) -> Result<Vec<Event>>;

    fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()>;

    fn size(&self) -> (u32, u32);

    fn buf_mut(&mut self) -> &mut [u8];
}
