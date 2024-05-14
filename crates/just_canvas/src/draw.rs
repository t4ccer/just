use crate::{Canvas, Color, Vector2, BYTES_PER_PIXEL};
use just_bdf::Glyph;

#[inline(always)]
pub fn set_pixel(buf: &mut [u8], window_size: Vector2<u32>, x: u32, y: u32, color: Color) {
    let offset = (window_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;

    #[cfg(debug_assertions)]
    {
        if x >= window_size.x {
            panic!("Point out of range: x = {x} >= {}, y = {y}", window_size.x)
        }
        if y >= window_size.y {
            panic!("Point out of range: x = {x}, y = {y} >= {}", window_size.y)
        }
    }

    buf[offset + 0] = color.b;
    buf[offset + 1] = color.g;
    buf[offset + 2] = color.r;
}

#[inline]
pub fn rectangle(canvas: &mut Canvas, position: Vector2<u32>, size: Vector2<u32>, color: Color) {
    let window_size = canvas.window_size();
    let buf = canvas.raw_buf_mut();

    for cy in position.y..(position.y + size.y).clamp(0, window_size.y) {
        for cx in position.x..(position.x + size.x).clamp(0, window_size.x) {
            set_pixel(buf, window_size, cx, cy, color);
        }
    }
}

#[inline]
pub fn distance_squared(p1: Vector2<u32>, p2: Vector2<u32>) -> u32 {
    let x_dist = (p1.x as i32 - p2.x as i32).abs() as u32;
    let y_dist = (p1.y as i32 - p2.y as i32).abs() as u32;
    x_dist * x_dist + y_dist * y_dist
}

#[inline]
pub fn inside_circle(center: Vector2<u32>, r: u32, point: Vector2<u32>) -> bool {
    distance_squared(center, point) <= r * r
}

#[inline]
pub fn inside_rectangle(position: Vector2<u32>, size: Vector2<u32>, point: Vector2<u32>) -> bool {
    point.x >= position.x
        && point.x <= position.x + size.x
        && point.y >= position.y
        && point.y <= size.y + position.y
}

#[inline]
pub fn circle(ui: &mut Canvas, center: Vector2<u32>, r: u32, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    let x = center.x.saturating_sub(r);
    let y = center.y.saturating_sub(r);

    for cy in y..(y + r * 2).clamp(0, window_size.y) {
        for cx in x..(x + r * 2).clamp(0, window_size.x) {
            let point = Vector2 { x: cx, y: cy };
            if inside_circle(center, r, point) {
                set_pixel(buf, window_size, cx, cy, color);
            }
        }
    }
}

#[derive(Debug)]
struct LineIter {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32,
    dx: i32,
    dy: i32,
    curr_x: u32,
    curr_y: u32,
}

impl LineIter {
    fn new(start: Vector2<u32>, end: Vector2<u32>) -> Self {
        let Vector2 {
            x: mut x1,
            y: mut y1,
        } = start;

        let Vector2 {
            x: mut x2,
            y: mut y2,
        } = end;

        let dx = x2 as i32 - x1 as i32;
        let dy = y2 as i32 - y1 as i32;

        if dx.abs() > dy.abs() {
            if x1 > x2 {
                core::mem::swap(&mut x1, &mut x2);
                core::mem::swap(&mut y1, &mut y2);
            }
        } else {
            if y1 > y2 {
                core::mem::swap(&mut x1, &mut x2);
                core::mem::swap(&mut y1, &mut y2);
            }
        }

        Self {
            x1,
            y1,
            x2,
            y2,
            dx,
            dy,
            curr_x: x1,
            curr_y: y1,
        }
    }
}

impl Iterator for LineIter {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.dx == 0 && self.dy == 0 {
            return None;
        }

        if self.dx.abs() > self.dy.abs() {
            if self.curr_x > self.x2 {
                return None;
            }

            let y =
                (self.dy * (self.curr_x as i32 - self.x1 as i32) / self.dx + self.y1 as i32) as u32;
            let res = (self.curr_x, y);

            self.curr_x += 1;

            Some(res)
        } else {
            if self.curr_y > self.y2 {
                return None;
            }

            let x =
                (self.dx * (self.curr_y as i32 - self.y1 as i32) / self.dy + self.x1 as i32) as u32;
            let res = (x, self.curr_y);

            self.curr_y += 1;

            Some(res)
        }
    }
}

#[inline]
pub fn thin_line(ui: &mut Canvas, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    for (x, y) in LineIter::new(start, end) {
        if y >= window_size.y || x >= window_size.x {
            continue;
        }

        set_pixel(buf, window_size, x, y, color);
    }
}

#[inline]
pub fn thin_dashed_line(ui: &mut Canvas, start: Vector2<u32>, end: Vector2<u32>, color: Color) {
    let window_size = ui.window_size();
    let buf = ui.raw_buf_mut();

    // chosen arbitrarily
    let dash_length: u32 = 10;
    let gap_length: u32 = 10;

    let mut n = 0;
    for (x, y) in LineIter::new(start, end) {
        if y >= window_size.y || x >= window_size.x {
            continue;
        }

        if n < dash_length {
            set_pixel(buf, window_size, x, y, color);
        }
        n += 1;
        if n >= dash_length + gap_length {
            n = 0;
        }
    }
}

// TODO: text_bdf_bounding_box

#[inline]
pub fn text_bdf_width<'a>(font: impl Fn(char) -> &'a Glyph, size: u32, text: &str) -> u32 {
    let mut x = 0;
    for glyph in text.chars().map(font) {
        x += size * glyph.bounding_box.width + size * 2;
    }
    x
}

pub fn glyph_bdf(ui: &mut Canvas, position: Vector2<u32>, size: u32, glyph: &Glyph, color: Color) {
    let padded_width = ((glyph.bounding_box.width + 7) / 8) * 8;
    let padded_height = ((glyph.bounding_box.height + 7) / 8) * 8;

    let x_off = padded_width as i32;
    let y_off = (padded_height - glyph.bounding_box.height) as i32 - 1;

    let total_x_offset = position.x as i32 + (x_off - glyph.bounding_box.x_off) * size as i32;
    let total_y_offset = position.y as i32 + (y_off - glyph.bounding_box.y_off) * size as i32;

    for gy in 0u32..glyph.bounding_box.height {
        for gx in 0u32..padded_width {
            let n = gy * padded_width + gx;
            let has_pixel = (glyph.bitmap[(n / 8) as usize] & (1 << (n % 8))) != 0;

            if has_pixel {
                rectangle(
                    ui,
                    Vector2 {
                        x: (total_x_offset - (gx as i32 * size as i32)) as u32,
                        y: (total_y_offset + (gy as i32 * size as i32)) as u32,
                    },
                    Vector2 { x: size, y: size },
                    color,
                );
            }
        }
    }
}
