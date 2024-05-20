// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

use just_canvas::{
    draw::{inside_circle, inside_rectangle},
    Color, Result, Vector2,
};
use just_immui::{
    invisible_button, invisible_draggable,
    monokaish::{self, Slider},
    Ui, UiId,
};

fn draw(ui: &mut Ui, state: &mut State) {
    ui.background(monokaish::BLACK);

    let view = ui.current_view();
    let top_bar_height = 100;
    let pad = 25;
    ui.with_view(
        Vector2 {
            x: pad,
            y: top_bar_height + pad,
        },
        Vector2 {
            x: view.size.x - pad * 2,
            y: view.size.y - pad * 2 - top_bar_height,
        },
        |ui| {
            editable_bezier(
                ui,
                &mut state.curve,
                state.show_traces,
                state.trace_lines_slider.value,
            );
        },
    );

    ui.with_view(
        // view.position,
        Vector2 { x: 0, y: 0 },
        Vector2 {
            x: view.size.x,
            y: top_bar_height,
        },
        |ui| top_bar(ui, state),
    );
}

fn ui() -> Result<()> {
    let mut state = State {
        curve: Bezier {
            start_point: Endpoint::new(100, 100),
            middle_point: Endpoint::new(300, 400),
            end_point: Endpoint::new(600, 150),
        },
        trace_lines_slider: Slider {
            min: 0,
            max: 50,
            value: 25,
        },
        show_traces: true,
    };

    #[cfg(not(feature = "screenshot"))]
    {
        let mut ui = Ui::new("Bezier")?;

        // Run UI at 60 FPS
        ui.fps_limited_loop(60, |ui| draw(ui, &mut state))
    }

    #[cfg(feature = "screenshot")]
    {
        return just_immui::screenshot!("bezier.png", state, Vector2 { x: 800, y: 600 });
    }
}

fn editable_bezier(ui: &mut Ui, state: &mut Bezier, show_traces: bool, trace_lines: u32) {
    if show_traces {
        ui.thin_dashed_line(
            state.start_point.position,
            state.middle_point.position,
            monokaish::RED,
        );
        ui.thin_dashed_line(
            state.middle_point.position,
            state.end_point.position,
            monokaish::RED,
        );

        for t in 1..trace_lines {
            let t = t as f32 / trace_lines as f32;

            let p1 = Vector2::linear_interpolation(
                state.start_point.position,
                state.middle_point.position,
                t,
            );
            let p2 = Vector2::linear_interpolation(
                state.middle_point.position,
                state.end_point.position,
                t,
            );

            ui.thin_line(p1, p2, monokaish::GREEN);
        }
    }

    if !show_traces || trace_lines < 2 {
        bezier_curve(
            ui,
            state.start_point.position,
            state.middle_point.position,
            state.end_point.position,
            128,
            monokaish::GREEN,
        );
    }

    endpoint(ui, new_id(0), &mut state.start_point);
    endpoint(ui, new_id(1), &mut state.middle_point);
    endpoint(ui, new_id(2), &mut state.end_point);
}

fn top_bar(ui: &mut Ui, state: &mut State) {
    let view = ui.current_view();

    let bottom_line_weigth = 3;

    ui.thin_dashed_line(
        Vector2 { x: 30, y: 32 },
        Vector2 { x: 95, y: 32 },
        monokaish::RED,
    );

    if state.show_traces {
        state
            .trace_lines_slider
            .draw(ui, new_id(3), Vector2 { x: 30, y: 65 }, 180);
    }

    checkbox(
        ui,
        new_id(4),
        &mut state.show_traces,
        Vector2 { x: 100, y: 20 },
    );

    ui.rectangle(
        Vector2 {
            x: 0,
            y: view.size.y as i32 - bottom_line_weigth as i32,
        },
        Vector2 {
            x: view.size.x,
            y: bottom_line_weigth,
        },
        monokaish::GRAY,
    );
}

fn checkbox(ui: &mut Ui, id: UiId, state: &mut bool, position: Vector2<i32>) {
    let size = Vector2 { x: 24, y: 24 };
    let pad = 3u32;

    let mut color = if *state {
        monokaish::BLUE
    } else {
        monokaish::BLACK
    };

    let button = invisible_button(ui, id, |cursor| {
        inside_rectangle(position, size, cursor.as_i32())
    });

    if button.got_hovered || button.got_released || button.got_pressed || button.got_unhovered {
        ui.set_dirty();
    }

    if button.is_pressed {
        color = monokaish::DARK_BLUE;
    }
    if button.got_released {
        *state = !*state;
        color = if *state {
            monokaish::BLUE
        } else {
            monokaish::BLACK
        };
    }

    ui.rectangle(position, size, monokaish::GRAY);
    ui.rectangle(
        Vector2 {
            x: position.x + pad as i32,
            y: position.y + pad as i32,
        },
        Vector2 {
            x: size.x - pad * 2,
            y: size.y - pad * 2,
        },
        color,
    );
}

fn endpoint(ui: &mut Ui, id: UiId, state: &mut Endpoint) {
    // chosen arbitrarily
    let r = 18;

    let view = ui.current_view();

    if ui.resized() {
        state.position = state
            .position
            .clamp(Vector2::<i32>::zero(), view.size.as_i32());
        ui.set_dirty();
    }

    ui.circle(state.position, r, monokaish::WHITE);
    ui.circle(state.position, r - 5, monokaish::BLACK);
    ui.circle(state.position, r - 12, monokaish::BLUE);

    let dragged = invisible_draggable(ui, id, |pointer| {
        inside_circle(state.position, r, pointer.as_i32())
    });

    let pointer = ui.pointer_position();

    if dragged {
        match state.previous_mouse {
            None => state.previous_mouse = Some(pointer),
            Some(prev_pointer) => {
                let new_position = Vector2 {
                    x: state.position.x as i32 + pointer.x as i32 - prev_pointer.x as i32,
                    y: state.position.y as i32 + pointer.y as i32 - prev_pointer.y as i32,
                }
                .clamp(Vector2::<i32>::zero(), view.size.as_i32())
                .as_u32();

                state.position = new_position.as_i32();
                state.previous_mouse = Some(pointer);
                ui.set_dirty();
            }
        }
    } else {
        state.previous_mouse = Some(pointer);
    }
}

fn bezier_curve(
    ui: &mut Ui,
    start: Vector2<i32>,
    middle: Vector2<i32>,
    end: Vector2<i32>,
    resolution: u32,
    color: Color,
) {
    let mut prev = start;
    for i in 0..resolution {
        let t = (i as f32 + 1.0) / resolution as f32;
        let next = Vector2::linear_interpolation(
            Vector2::linear_interpolation(start, middle, t),
            Vector2::linear_interpolation(middle, end, t),
            t,
        );
        ui.thin_line(prev, next, color);
        prev = next;
    }
}

struct State {
    curve: Bezier,
    trace_lines_slider: Slider,
    show_traces: bool,
}

struct Bezier {
    start_point: Endpoint,
    middle_point: Endpoint,
    end_point: Endpoint,
}

struct Endpoint {
    position: Vector2<i32>,
    previous_mouse: Option<Vector2<u32>>,
}

impl Endpoint {
    fn new(x: i32, y: i32) -> Self {
        Self {
            position: Vector2 { x, y },
            previous_mouse: None,
        }
    }
}

fn main() {
    ui().unwrap();
}

fn new_id(id: u32) -> UiId {
    UiId {
        id,
        parent: 0,
        index: 0,
    }
}
