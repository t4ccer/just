use crate::{backend::Backend, Event, Result, Vector2};

#[derive(Debug)]
pub enum Error {
    TriedToResize,
}

pub(crate) struct SharedBitmapBackend<'buf> {
    size: Vector2<u32>,
    buf: &'buf mut [u8],
}

impl<'buf> SharedBitmapBackend<'buf> {
    pub(crate) fn _new(size: Vector2<u32>, buf: &'buf mut [u8]) -> Self {
        Self { size, buf }
    }
}

impl<'buf> Backend for SharedBitmapBackend<'buf> {
    #[inline]
    fn flush_window(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn events(&mut self) -> Result<Vec<Event>> {
        Ok(Vec::new())
    }

    fn resize(&mut self, _new_size: Vector2<u32>) -> Result<()> {
        Err(crate::CanvasError::SharedBitmapError(Error::TriedToResize))
    }

    #[inline]
    fn size(&self) -> Vector2<u32> {
        self.size
    }

    #[inline]
    fn buf_mut(&mut self) -> &mut [u8] {
        self.buf
    }

    #[inline]
    fn buf(&self) -> &[u8] {
        self.buf
    }
}
