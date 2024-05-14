use crate::{backend::Backend, Event, Result, Vector2, BYTES_PER_PIXEL};
use core::cmp;

pub(crate) struct BitmapBackend {
    size: Vector2<u32>,
    buf: Vec<u8>,
}

impl BitmapBackend {
    pub(crate) fn new(size: Vector2<u32>) -> Self {
        Self {
            size,
            buf: vec![0u8; size.x as usize * size.y as usize * BYTES_PER_PIXEL as usize],
        }
    }
}

impl Backend for BitmapBackend {
    #[inline]
    fn flush_window(&mut self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn events(&mut self) -> Result<Vec<Event>> {
        Ok(Vec::new())
    }

    fn resize(&mut self, new_size: Vector2<u32>) -> Result<()> {
        let old_buf = self.buf.clone();
        let old_size = self.size;
        let mut new_buf =
            vec![0u8; new_size.x as usize * new_size.y as usize * BYTES_PER_PIXEL as usize];

        for y in 0..cmp::min(new_size.y, old_size.y) {
            for x in 0..cmp::min(new_size.x, old_size.x) {
                let new_offset = (new_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;
                let old_offset = (old_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;
                new_buf[new_offset] = old_buf[old_offset];
                new_buf[new_offset + 1] = old_buf[old_offset + 1];
                new_buf[new_offset + 2] = old_buf[old_offset + 2];
                new_buf[new_offset + 3] = old_buf[old_offset + 3];
            }
        }

        self.size = new_size;
        self.buf = new_buf;
        Ok(())
    }

    #[inline]
    fn size(&self) -> Vector2<u32> {
        self.size
    }

    #[inline]
    fn buf_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }

    #[inline]
    fn buf(&self) -> &[u8] {
        &self.buf
    }
}
