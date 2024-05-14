use bdf::BdfCharMap;
use just_canvas::{draw, Canvas, Color, Pointer, Result, Vector2};
use std::{cmp, time::Duration};

mod bdf;

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
    font_char_map: BdfCharMap,
    dirty: bool,
    dirty_next: bool,
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
            font_char_map: BdfCharMap::ib8x8u(),
            dirty: true,
            dirty_next: true,
        }
    }

    #[inline]
    /// Return `true` if element was hot or active and was changed to inactive
    pub fn make_inactive(&mut self, id: UiId) -> bool {
        let mut res = false;

        match self.hot {
            Some(hot) if hot == id => {
                self.hot = None;
                res = true;
            }
            _ => {}
        }

        match self.active {
            Some(active) if active == id => {
                self.active = None;
                res = true;
            }
            _ => {}
        }

        res
    }

    #[inline]
    /// Return true if no other element was hot and this one was made hot
    pub fn make_hot(&mut self, id: UiId) -> bool {
        match self.hot {
            None => {
                self.hot = Some(id);
                true
            }
            _ => false,
        }
    }

    #[inline]
    /// Return true if current element was hot and was made active
    pub fn make_active(&mut self, id: UiId) -> bool {
        match self.hot {
            Some(hot) if hot == id => {
                self.active = Some(id);
                true
            }
            _ => false,
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
    pub fn is_dirty(&self) -> bool {
        self.canvas.resized() || self.dirty
    }

    #[inline]
    pub fn set_dirty(&mut self) {
        self.dirty_next = true
    }

    #[inline]
    pub(crate) fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    #[inline]
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    #[inline]
    pub fn window_size(&self) -> Vector2<u32> {
        self.canvas.window_size()
    }

    #[inline]
    pub fn background(&mut self, color: Color) {
        let window_size = self.canvas.window_size();
        self.rectangle(Vector2 { x: 0, y: 0 }, window_size, color)
    }

    #[inline]
    pub fn rectangle(&mut self, position: Vector2<u32>, size: Vector2<u32>, color: Color) {
        if !self.dirty {
            return;
        }

        draw::rectangle(&mut self.canvas, position, size, color);
    }

    #[inline]
    pub fn thin_line(&mut self, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
        if !self.dirty {
            return;
        }

        draw::thin_line(&mut self.canvas, start, end, color);
    }

    #[inline]
    pub fn thin_dashed_line(&mut self, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
        if !self.dirty {
            return;
        }

        draw::thin_dashed_line(&mut self.canvas, start, end, color);
    }

    #[inline]
    pub fn circle(&mut self, center: Vector2<u32>, r: u32, color: Color) {
        if !self.dirty {
            return;
        }

        draw::circle(&mut self.canvas, center, r, color);
    }

    #[inline]
    pub fn pointer(&self) -> &Pointer {
        self.canvas.pointer()
    }

    #[inline]
    pub fn resized(&self) -> bool {
        self.canvas.resized()
    }

    pub fn fps_limited_loop<F>(&mut self, fps: u64, mut draw: F) -> Result<()>
    where
        F: FnMut(&mut Self),
    {
        while !self.canvas_mut().should_close() {
            let frame_start = std::time::Instant::now();
            self.canvas_mut().process_events()?;

            draw(self);

            if self.dirty {
                self.canvas_mut().flush()?;
            }

            self.dirty = false;
            if self.dirty_next {
                self.dirty = true;
            }
            self.dirty_next = false;

            let frame_end = std::time::Instant::now();
            let frame_duration = frame_end - frame_start;
            let final_sleep = Duration::from_micros(1000000 / fps).checked_sub(frame_duration);
            if let Some(final_sleep) = final_sleep {
                std::thread::sleep(final_sleep);
            }
        }

        Ok(())
    }

    pub fn text(&mut self, mut position: Vector2<u32>, size: u32, text: &str, color: Color) {
        let canvas = &mut self.canvas;
        let char_map = &self.font_char_map;
        for glyph in text.chars().map(|c| char_map.get(c)) {
            draw::glyph_bdf(canvas, position, size, glyph, color);
            position.x += size * glyph.bounding_box.width + size * 2;
        }
    }

    pub fn text_size(&self, font_size: u32, text: &str) -> Vector2<u32> {
        let mut size = Vector2::<u32>::zero();
        let char_map = &self.font_char_map;
        for glyph in text.chars().map(|c| char_map.get(c)) {
            size.x += font_size * glyph.bounding_box.width + font_size * 2;
            size.y = cmp::max(size.y, font_size * glyph.bounding_box.height);
        }
        if !text.is_empty() {
            size.x -= font_size * 2;
        }
        size
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

        let mut ppm = Vec::<u8>::new();
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

#[derive(Debug)]
pub struct Button {
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub got_pressed: bool,
    pub got_hovered: bool,
    pub got_unhovered: bool,
    pub got_released: bool,
}

pub fn invisible_button(
    ui: &mut Ui,
    id: UiId,
    in_bounds: impl FnOnce(Vector2<u32>) -> bool,
) -> Button {
    let mut button = Button {
        is_hovered: false,
        is_pressed: false,
        got_pressed: false,
        got_hovered: false,
        got_unhovered: false,
        got_released: false,
    };

    if in_bounds(ui.pointer().position) {
        if ui.is_hot(id) {
            button.got_hovered = false;
            button.is_hovered = true;
        } else {
            button.got_hovered = ui.make_hot(id);
            button.is_hovered = button.got_hovered;
        }

        if ui.pointer().is_pressed(1) && ui.is_hot(id) && !ui.is_active(id) {
            button.got_pressed = ui.make_active(id);
            button.is_pressed = button.got_pressed;
        } else if ui.pointer().is_pressed(1) && ui.is_active(id) {
            button.is_pressed = true;
        } else if !ui.pointer().is_pressed(1) && ui.is_active(id) {
            ui.make_inactive(id);
            button.got_released = true;
        }
    } else {
        button.got_unhovered = ui.make_inactive(id);
    }

    button
}

pub fn invisible_draggable(
    ui: &mut Ui,
    id: UiId,
    in_bounds: impl FnOnce(Vector2<u32>) -> bool,
) -> bool {
    if in_bounds(ui.pointer().position) {
        if !ui.is_hot(id) && ui.pointer().is_pressed(1) {
            false
        } else {
            ui.make_hot(id);
            if ui.pointer().is_pressed(1) {
                ui.make_active(id);
                true
            } else {
                false
            }
        }
    } else {
        if ui.is_active(id) && ui.pointer().is_pressed(1) {
            true
        } else {
            ui.make_inactive(id);
            false
        }
    }
}
