use just_bdf::Glyph;

use crate::{Color, Context, Pointer, BYTES_PER_PIXEL};

#[inline]
pub fn background(ui: &mut Context, color: Color) {
    let (width, height) = ui.window_size();
    rectangle(ui, 0, 0, width, height, color)
}

#[inline(always)]
pub fn set_pixel(
    buf: &mut [u8],
    window_width: u32,
    #[allow(unused_variables)] window_height: u32,
    x: u32,
    y: u32,
    color: Color,
) {
    let offset = (window_width * y + x) as usize * BYTES_PER_PIXEL as usize;

    #[cfg(debug_assertions)]
    {
        if x >= window_width {
            panic!("Point out of range: x = {x} >= {window_width}, y = {y}")
        }
        if y >= window_height {
            panic!("Point out of range: x = {x}, y = {y} >= {window_height}")
        }
    }

    buf[offset + 0] = color.b;
    buf[offset + 1] = color.g;
    buf[offset + 2] = color.r;
}

#[inline]
pub fn rectangle(ui: &mut Context, x: u32, y: u32, width: u32, height: u32, color: Color) {
    let (window_width, window_height) = ui.window_size();
    let buf = ui.backend.buf_mut();

    for cy in y..(y + height).clamp(0, window_height) {
        for cx in x..(x + width).clamp(0, window_width) {
            set_pixel(buf, window_width, window_height, cx, cy, color);
        }
    }
}

#[inline]
pub fn distance_squared(x1: u32, y1: u32, x2: u32, y2: u32) -> u32 {
    let x_dist = (x1 as i32 - x2 as i32).abs() as u32;
    let y_dist = (y1 as i32 - y2 as i32).abs() as u32;
    x_dist * x_dist + y_dist * y_dist
}

#[inline]
pub fn inside_circle(ox: u32, oy: u32, r: u32, x: u32, y: u32) -> bool {
    distance_squared(ox, oy, x, y) <= r * r
}

#[inline]
pub fn inside_rectangle(rx: u32, ry: u32, width: u32, height: u32, x: u32, y: u32) -> bool {
    x >= rx && x <= width + rx && y >= ry && y <= height + ry
}

#[inline]
pub fn circle(ui: &mut Context, ox: u32, oy: u32, r: u32, color: Color) {
    let (window_width, window_height) = ui.window_size();
    let buf = ui.backend.buf_mut();

    let x = ox.saturating_sub(r);
    let y = oy.saturating_sub(r);

    for cy in y..(y + r * 2).clamp(0, window_height) {
        for cx in x..(x + r * 2).clamp(0, window_width) {
            if inside_circle(ox, oy, r, cx, cy) {
                set_pixel(buf, window_width, window_height, cx, cy, color);
            }
        }
    }
}

#[derive(Debug)]
pub struct LineIter {
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
    pub fn new(mut x1: u32, mut y1: u32, mut x2: u32, mut y2: u32) -> Self {
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
pub fn thin_line(ui: &mut Context, x1: u32, y1: u32, x2: u32, y2: u32, color: Color) {
    let (window_width, window_height) = ui.window_size();
    let buf = ui.backend.buf_mut();

    for (x, y) in LineIter::new(x1, y1, x2, y2) {
        if y >= window_height || x >= window_width {
            break;
        }

        set_pixel(buf, window_width, window_height, x, y, color);
    }
}

#[inline]
pub fn thin_dashed_line(ui: &mut Context, x1: u32, y1: u32, x2: u32, y2: u32, color: Color) {
    let (window_width, window_height) = ui.window_size();
    let buf = ui.backend.buf_mut();

    // chosen arbitrarily
    let dash_length: u32 = 10;
    let gap_length: u32 = 10;

    let mut n = 0;
    for (x, y) in LineIter::new(x1, y1, x2, y2) {
        if y >= window_height || x >= window_width {
            break;
        }

        if n < dash_length {
            set_pixel(buf, window_width, window_height, x, y, color);
        }
        n += 1;
        if n >= dash_length + gap_length {
            n = 0;
        }
    }
}

// TODO: Colour

#[inline]
pub fn text_bdf<'a>(
    ui: &mut Context,
    font: impl Fn(char) -> &'a Glyph,
    mut x: u32,
    y: u32,
    size: u32,
    text: &str,
) {
    for glyph in text.chars().map(font) {
        glyph_bdf(ui, x, y, size, glyph);
        x += size * glyph.bounding_box.width + size * 2;
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

fn glyph_bdf(ui: &mut Context, x: u32, y: u32, size: u32, glyph: &Glyph) {
    let padded_width = ((glyph.bounding_box.width + 7) / 8) * 8;
    let padded_height = ((glyph.bounding_box.height + 7) / 8) * 8;

    let x_off = padded_width as i32;
    let y_off = (padded_height - glyph.bounding_box.height) as i32;

    let total_x_offset = x as i32 + (x_off - glyph.bounding_box.x_off) * size as i32;
    let total_y_offset = y as i32 + (y_off - glyph.bounding_box.y_off) * size as i32;

    for gy in 0u32..glyph.bounding_box.height {
        for gx in 0u32..padded_width {
            let n = gy * padded_width + gx;
            let has_pixel = (glyph.bitmap[(n / 8) as usize] & (1 << (n % 8))) != 0;

            if has_pixel {
                rectangle(
                    ui,
                    (total_x_offset - (gx as i32 * size as i32)) as u32,
                    (total_y_offset + (gy as i32 * size as i32)) as u32,
                    size,
                    size,
                    Color::from_raw(0xdddddd),
                );
            }
        }
    }
}

pub struct Button {
    pub clicked: bool,
    pub pressed: bool,
    pub active: bool,
}

pub fn invisible_button(ui: &mut Context, in_bounds: impl FnOnce(&Pointer) -> bool) -> Button {
    let button_id = ui.next_id();

    if in_bounds(ui.pointer()) {
        if !ui.pointer().is_pressed(1) {
            ui.make_hot(button_id);
        }

        if ui.is_active(button_id) {
            if !ui.pointer().is_pressed(1) {
                ui.make_inactive(button_id);

                Button {
                    clicked: true,
                    pressed: false,
                    active: true,
                }
            } else {
                Button {
                    clicked: false,
                    pressed: true,
                    active: true,
                }
            }
        } else if ui.is_hot(button_id) {
            if ui.pointer().is_pressed(1) {
                ui.make_active(button_id);
            }

            Button {
                clicked: false,
                pressed: false,
                active: true,
            }
        } else {
            Button {
                clicked: false,
                pressed: false,
                active: false,
            }
        }
    } else {
        ui.make_inactive(button_id);
        Button {
            clicked: false,
            pressed: false,
            active: false,
        }
    }
}
