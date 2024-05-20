use just_canvas::{draw::inside_rectangle, Color, Vector2};

use crate::{invisible_draggable, Ui, UiId};

pub const BLACK: Color = Color::from_raw(0xff222222);
pub const DARK_GRAY: Color = Color::from_raw(0xff333333);
pub const GRAY: Color = Color::from_raw(0xff666666);
pub const LIGHT_GRAY: Color = Color::from_raw(0xffaaaaaa);
pub const WHITE: Color = Color::from_raw(0xffdddddd);
pub const ORANGE: Color = Color::from_raw(0xfffd971f);
pub const YELLOW: Color = Color::from_raw(0xffe6db74);
pub const PURPLE: Color = Color::from_raw(0xff9d65ff);
pub const BLUE: Color = Color::from_raw(0xff4eb4fa);
pub const DARK_BLUE: Color = Color::from_raw(0xff007fc1);
pub const RED: Color = Color::from_raw(0xfff92672);
pub const RED_DULL: Color = Color::from_raw(0xffc15d81);
pub const GREEN: Color = Color::from_raw(0xffa7e22e);
pub const GREEN_DULL: Color = Color::from_raw(0xff98b758);

macro_rules! map_range {
    ($input: expr, $input_start:expr, $input_end: expr, $output_start:expr, $output_end:expr, $(,)?) => {
        (($output_end as f32 - $output_start as f32) / ($input_end as f32 - $input_start as f32)
            * ($input as f32 - $input_start as f32)
            + $output_start as f32)
            .clamp($output_start as f32, $output_end as f32)
    };
}

pub struct Slider {
    pub min: u32,
    pub max: u32,
    pub value: u32,
}

impl Slider {
    pub fn draw(&mut self, ui: &mut Ui, id: UiId, position: Vector2<i32>, slider_length: u32) {
        // chosen arbitrarily
        let size = Vector2 {
            x: slider_length,
            y: 6,
        };
        let handle_size = Vector2 { x: 8, y: 20 };

        ui.rectangle(position, size, GRAY);

        let handle_position = Vector2 {
            x: map_range!(
                self.value,
                self.min,
                self.max,
                position.x,
                position.x + size.x as i32,
            ) as i32,
            y: position.y - handle_size.y as i32 / 2 + size.y as i32 / 2,
        };

        ui.rectangle(handle_position, handle_size, BLUE);

        let dragged = invisible_draggable(ui, id, |pointer| {
            inside_rectangle(
                position,
                Vector2 {
                    x: size.x,
                    y: handle_size.y as u32,
                },
                pointer.as_i32(),
            )
        });
        if dragged {
            let px = (ui.pointer_position().x as i32)
                .clamp(position.x as i32, position.x as i32 + size.x as i32)
                as u32;
            self.value = map_range!(
                px,
                position.x,
                position.x + size.x as i32,
                self.min,
                self.max,
            ) as u32;
            ui.set_dirty();
        }
    }
}
