use std::time::Duration;

use just_canvas::{Canvas, Pointer, Result};

pub mod draw;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiId {
    pub id: u32,
    pub parent: u32,
    pub index: u32,
}

pub struct Ui {
    canvas: Canvas,
    hot: Option<UiId>,
    active: Option<UiId>,
}

impl Ui {
    #[inline]
    pub fn new(title: &str) -> Result<Self> {
        let canvas = Canvas::new(title)?;
        Ok(Self::with_canvas(canvas))
    }

    #[inline]
    pub fn with_canvas(canvas: Canvas) -> Self {
        Self {
            canvas,
            hot: None,
            active: None,
        }
    }

    #[inline]
    pub fn make_inactive(&mut self, id: UiId) {
        match self.hot {
            Some(hot) if hot == id => {
                self.hot = None;
            }
            _ => {}
        }

        match self.active {
            Some(active) if active == id => {
                self.active = None;
            }
            _ => {}
        }
    }

    #[inline]
    pub fn make_hot(&mut self, id: UiId) {
        match self.hot {
            None => {
                self.hot = Some(id);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn make_active(&mut self, id: UiId) {
        match self.hot {
            Some(hot) if hot == id => {
                self.active = Some(id);
            }
            _ => {}
        }
    }

    #[inline]
    pub fn is_hot(&self, id: UiId) -> bool {
        self.hot.is_some_and(|hot| hot == id)
    }

    #[inline]
    pub fn is_active(&self, id: UiId) -> bool {
        self.active.is_some_and(|active| active == id)
    }

    #[inline]
    pub fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer {
        self.canvas.pointer()
    }

    // TODO: Remove: deprecated
    pub fn fps_limited_loop<F>(&mut self, fps: u64, mut draw: F) -> Result<()>
    where
        F: FnMut(&mut Self),
    {
        while !self.canvas().should_close() {
            let frame_start = std::time::Instant::now();
            self.canvas().process_events()?;

            draw(self);
            self.canvas().flush()?;

            let frame_end = std::time::Instant::now();
            let frame_duration = frame_end - frame_start;
            let final_sleep = Duration::from_micros(1000000 / fps).checked_sub(frame_duration);
            if let Some(final_sleep) = final_sleep {
                std::thread::sleep(final_sleep);
            }
        }

        Ok(())
    }
}

#[cfg(feature = "screenshot")]
pub fn to_ppm(
    img: &[u8],
    size: just_canvas::Vector2<u32>,
    mut f: impl std::io::Write,
) -> std::io::Result<()> {
    writeln!(f, "P6")?;
    writeln!(f, "{} {}", size.x, size.y)?;
    writeln!(f, "255")?;

    for y in 0..(size.y as usize) {
        for x in 0..(size.x as usize) {
            f.write_all(&[img[(y * size.x as usize + x as usize) * 4 + 2]])?;
            f.write_all(&[img[(y * size.x as usize + x as usize) * 4 + 1]])?;
            f.write_all(&[img[(y * size.x as usize + x as usize) * 4 + 0]])?;
        }
    }

    f.flush()
}

#[cfg(feature = "screenshot")]
#[macro_export]
macro_rules! screenshot {
    ($file_name:literal, $state:expr, $size:expr) => {{
        use std::{
            io::Write,
            path::Path,
            process::{Command, Stdio},
        };

        let size = $size;
        let canvas =
            just_canvas::Canvas::with_backend_type("", just_canvas::BackendType::Bitmap { size })
                .unwrap();
        let mut ui = Ui::with_canvas(canvas);
        draw(&mut ui, &mut $state);

        let mut ppm = Vec::new();
        just_immui::to_ppm(ui.canvas().raw_buf(), size, &mut ppm).unwrap();

        let out_png_path = Path::new(file!()).with_file_name($file_name);
        let mut convert = Command::new("convert")
            .arg("-")
            .arg(format!("{}", out_png_path.display()))
            .stdin(Stdio::piped())
            .spawn()
            .expect("Could not spawn `convert`");

        let mut stdin = convert.stdin.take().unwrap();
        stdin.write_all(&ppm).unwrap();
        drop(stdin);
        convert.wait().unwrap();

        Ok(())
    }};
}
