// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

use just_canvas::{draw::inside_rectangle, Color, Result, Vector2};
use just_immui::{invisible_button, Button, Ui, UiId};

/// Main UI loop
fn draw(ui: &mut Ui, state: &mut State) {
    ui.background(Color::from_raw(0xff222222));

    ui.text(
        Vector2 { x: 50, y: 30 },
        3,
        "Hello, World!",
        Color::from_raw(0xffdddddd),
    );

    counter_button(
        ui,
        new_id(0),
        Vector2 { x: 50, y: 100 },
        &mut state.count_left,
    );

    let right_button = counter_button(
        ui,
        new_id(1),
        Vector2 { x: 200, y: 100 },
        &mut state.count_right,
    );
    // add additional action on click
    if right_button.got_released {
        println!("Right clicked!");
    }
}

fn ui() -> Result<()> {
    let mut state = State {
        count_left: 0,
        count_right: 0,
    };

    #[cfg(not(feature = "screenshot"))]
    {
        let mut ui = Ui::new("My Application")?;

        // run UI at 60 FPS
        ui.fps_limited_loop(60, |ui| draw(ui, &mut state))
    }
    #[cfg(feature = "screenshot")]
    {
        return just_immui::screenshot!("hello_world.png", state, Vector2 { x: 400, y: 200 });
    }
}

/// Persistent state between UI frames
struct State {
    count_left: u32,
    count_right: u32,
}

fn main() {
    ui().unwrap();
}

/// Button with click counter - custom widget composed from simpler ones
fn counter_button(ui: &mut Ui, id: UiId, position: Vector2<u32>, state: &mut u32) -> Button {
    let inactive_color = Color::from_raw(0xfff92672);
    let hot_color = Color::from_raw(0xff4eb4fa);
    let active_color = Color::from_raw(0xffa7e22e);

    let width = 120;
    let height = 40;
    let size = Vector2 {
        x: width,
        y: height,
    };
    let font_size = 2;

    let button = invisible_button(ui, id, |pointer| inside_rectangle(position, size, pointer));

    if button.got_hovered || button.got_released || button.got_pressed || button.got_unhovered {
        ui.set_dirty();
    }

    if button.got_released || button.is_pressed {
        ui.rectangle(position, size, active_color);
    } else if button.is_hovered {
        ui.rectangle(position, size, hot_color);
    } else {
        ui.rectangle(position, size, inactive_color);
    }

    if button.got_released {
        *state += 1;
    }
    let txt = format!("{}", *state);
    let text_size = ui.text_size(font_size, &txt);
    ui.text(
        Vector2 {
            x: position.x + (width / 2 - text_size.x / 2),
            y: position.y + (height / 2 - text_size.y / 2),
        },
        font_size,
        &txt,
        Color::from_raw(0xffdddddd),
    );

    button
}

fn new_id(id: u32) -> UiId {
    UiId {
        id,
        parent: 0,
        index: 0,
    }
}
