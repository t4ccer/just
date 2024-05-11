use just_immui::{Color, Context, Result};

pub struct Button {
    pub clicked: bool,
    pub active: bool,
}

fn button(ui: &mut Context, x: u32, y: u32, width: u32, height: u32) -> Button {
    let button_id = ui.next_id();

    let inactive_color = Color::from_raw(0xf92672);
    let hot_color = Color::from_raw(0x4eb4fa);
    let active_color = Color::from_raw(0xa7e22e);

    if ui.pointer().x >= x
        && ui.pointer().x <= width + x
        && ui.pointer().y >= y
        && ui.pointer().y <= height + y
    {
        if !ui.pointer().is_pressed(1) {
            ui.make_hot(button_id);
        }

        if ui.is_active(button_id) {
            if !ui.pointer().is_pressed(1) {
                ui.make_inactive(button_id);

                // We're coloring here with `hot_color` because mouse is still over button
                // so it'll flicker on next frame
                ui.rectangle(x, y, width, height, hot_color);

                Button {
                    clicked: true,
                    active: true,
                }
            } else {
                ui.rectangle(x, y, width, height, active_color);
                Button {
                    clicked: false,
                    active: true,
                }
            }
        } else if ui.is_hot(button_id) {
            if ui.pointer().is_pressed(1) {
                ui.make_active(button_id);
            }

            ui.rectangle(x, y, width, height, hot_color);
            Button {
                clicked: false,
                active: true,
            }
        } else {
            ui.rectangle(x, y, width, height, inactive_color);
            Button {
                clicked: false,
                active: false,
            }
        }
    } else {
        ui.make_inactive(button_id);
        ui.rectangle(x, y, width, height, inactive_color);
        Button {
            clicked: false,
            active: false,
        }
    }
}

fn draw(ui: &mut Context) {
    ui.background(Color::from_raw(0x222222));

    if button(ui, 50, 100, 120, 40).clicked {
        println!("Left!");
    }

    if button(ui, 50 + 150, 100, 120, 40).clicked {
        println!("Middle!");
    }

    if button(ui, 50 + 300, 100, 120, 40).clicked {
        println!("Right!");
    }
}

fn ui() -> Result<()> {
    let mut ui = Context::new("My Application")?;
    ui.fps_limited_loop(60, draw)?;
    Ok(())
}

fn main() {
    ui().unwrap();
}
