use crate::{backend::Backend, Event, Result, Vector2, BYTES_PER_PIXEL};

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
    fn flush_window(&mut self) -> Result<()> {
        Ok(())
    }

    fn events(&mut self) -> Result<Vec<Event>> {
        Ok(Vec::new())
    }

    // TODO: Move to Vector2
    fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        let mut buf =
            vec![0u8; new_width as usize * new_height as usize * BYTES_PER_PIXEL as usize];
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let new_offset = (new_width * y + x) as usize;
                let old_offset = (self.size.x * y + x) as usize;
                buf[new_offset] = self.buf[old_offset];
            }
        }

        self.size = Vector2 {
            x: new_height,
            y: new_height,
        };
        self.buf = buf;
        Ok(())
    }

    // TODO: Move to Vector2
    fn size(&self) -> (u32, u32) {
        (self.size.x, self.size.y)
    }

    fn buf_mut(&mut self) -> &mut [u8] {
        &mut self.buf
    }
}
