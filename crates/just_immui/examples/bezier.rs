use just_canvas::{
    draw::{inside_circle, inside_rectangle},
    Color, Result, Vector2,
};
use just_immui::{invisible_button, invisible_draggable, Ui, UiId};

const BLUE: Color = Color::from_raw(0xff4eb4fa);
const DARK_BLUE: Color = Color::from_raw(0xff0b629e);
const RED: Color = Color::from_raw(0xfff92672);
const BLACK: Color = Color::from_raw(0xff222222);
const WHITE: Color = Color::from_raw(0xffdddddd);
const GRAY: Color = Color::from_raw(0xff666666);
const GREEN: Color = Color::from_raw(0xffa7e22e);

fn draw(ui: &mut Ui, state: &mut State) {
    ui.background(BLACK);

    let panel_size = ui.window_size();
    ui.rectangle(
        Vector2 { x: 0, y: 95 },
        Vector2 {
            x: panel_size.x,
            y: 5,
        },
        GRAY,
    );

    if state.show_traces {
        ui.thin_dashed_line(state.start_point.position, state.middle_point.position, RED);
        ui.thin_dashed_line(state.middle_point.position, state.end_point.position, RED);

        for t in 1..state.trace_lines.value {
            let t = t as f32 / state.trace_lines.value as f32;

            let p1 =
                linear_interpolation(state.start_point.position, state.middle_point.position, t);
            let p2 = linear_interpolation(state.middle_point.position, state.end_point.position, t);

            ui.thin_line(p1, p2, GREEN);
        }
    }

    if !state.show_traces || state.trace_lines.value < 2 {
        bezier_curve(
            ui,
            state.start_point.position,
            state.middle_point.position,
            state.end_point.position,
            128,
            GREEN,
        );
    }

    endpoint(ui, new_id(0), &mut state.start_point);
    endpoint(ui, new_id(1), &mut state.middle_point);
    endpoint(ui, new_id(2), &mut state.end_point);

    ui.thin_dashed_line(Vector2 { x: 30, y: 32 }, Vector2 { x: 95, y: 32 }, RED);
    if state.show_traces {
        slider(
            ui,
            new_id(3),
            &mut state.trace_lines,
            Vector2 { x: 30, y: 65 },
        );
    }
    checkbox(
        ui,
        new_id(4),
        &mut state.show_traces,
        Vector2 { x: 100, y: 20 },
    );
}

fn ui() -> Result<()> {
    let mut state = State {
        start_point: Endpoint::new(100, 200),
        middle_point: Endpoint::new(300, 500),
        end_point: Endpoint::new(600, 250),
        trace_lines: Slider {
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

fn checkbox(ui: &mut Ui, id: UiId, state: &mut bool, position: Vector2<u32>) {
    let size_len = 24;
    let size = Vector2 {
        x: size_len,
        y: size_len,
    };
    let pad = 3;

    let mut color = if *state { BLUE } else { BLACK };

    let button = invisible_button(ui, id, |cursor| inside_rectangle(position, size, cursor));

    if button.got_hovered || button.got_released || button.got_pressed || button.got_unhovered {
        ui.set_dirty();
    }

    if button.is_pressed {
        color = DARK_BLUE;
    }
    if button.got_released {
        *state = !*state;
        color = if *state { BLUE } else { BLACK };
    }

    ui.rectangle(position, size, GRAY);
    ui.rectangle(
        Vector2 {
            x: position.x + pad,
            y: position.y + pad,
        },
        Vector2 {
            x: size_len - pad * 2,
            y: size_len - pad * 2,
        },
        color,
    );
}

fn slider(ui: &mut Ui, id: UiId, state: &mut Slider, position: Vector2<u32>) {
    // chosen arbitrarily
    let size = Vector2 { x: 180, y: 6 };
    let handle_size = Vector2 { x: 8, y: 20 };

    ui.rectangle(position, size, GRAY);

    let handle_position = Vector2 {
        x: map_range(
            state.value,
            state.min,
            state.max,
            position.x,
            position.x + size.x,
        ),
        y: position.y - handle_size.y / 2 + size.y / 2,
    };

    ui.rectangle(handle_position, handle_size, BLUE);

    let dragged = invisible_draggable(ui, id, |pointer| {
        inside_rectangle(
            position,
            Vector2 {
                x: size.x,
                y: handle_size.y,
            },
            pointer,
        )
    });
    if dragged {
        let px = (ui.pointer().position.x as i32)
            .clamp(position.x as i32, position.x as i32 + size.x as i32) as u32;
        state.value = map_range(px, position.x, position.x + size.x, state.min, state.max);
        ui.set_dirty();
    }
}

fn endpoint(ui: &mut Ui, id: UiId, state: &mut Endpoint) {
    // chosen arbitrarily
    let r = 18;

    let window_size = ui.window_size();

    if ui.resized() {
        state.position = state.position.clamp(Vector2 { x: 0, y: 0 }, window_size);
        ui.set_dirty();
    }

    ui.circle(state.position, r, WHITE);
    ui.circle(state.position, r - 5, BLACK);
    ui.circle(state.position, r - 12, BLUE);

    let dragged = invisible_draggable(ui, id, |pointer| inside_circle(state.position, r, pointer));

    let pointer = ui.pointer().position;

    if dragged {
        match state.previous_mouse {
            None => state.previous_mouse = Some(pointer),
            Some(prev_pointer) => {
                let new_position = Vector2 {
                    x: state.position.x as i32 + pointer.x as i32 - prev_pointer.x as i32,
                    y: state.position.y as i32 + pointer.y as i32 - prev_pointer.y as i32,
                }
                .clamp(Vector2 { x: 0, y: 0 }, window_size.as_i32())
                .as_u32();

                state.position = new_position;
                state.previous_mouse = Some(pointer);
                ui.set_dirty();
            }
        }
    } else {
        state.previous_mouse = Some(pointer)
    }
}

fn linear_interpolation(p1: Vector2<u32>, p2: Vector2<u32>, t: f32) -> Vector2<u32> {
    let mut x = p1.x as f32 + (p2.x as f32 - p1.x as f32) * t;
    if x < 0.0 {
        x = 0.0;
    }
    let x = x as u32;

    let mut y = p1.y as f32 + (p2.y as f32 - p1.y as f32) * t;
    if y < 0.0 {
        y = 0.0;
    }
    let y = y as u32;

    Vector2 { x, y }
}

fn bezier_curve(
    ui: &mut Ui,
    start: Vector2<u32>,
    middle: Vector2<u32>,
    end: Vector2<u32>,
    resolution: u32,
    color: Color,
) {
    let mut prev = start;
    for i in 0..resolution {
        let t = (i as f32 + 1.0) / resolution as f32;
        let next = linear_interpolation(
            linear_interpolation(start, middle, t),
            linear_interpolation(middle, end, t),
            t,
        );
        ui.thin_line(prev, next, color);
        prev = next;
    }
}

struct State {
    start_point: Endpoint,
    middle_point: Endpoint,
    end_point: Endpoint,
    trace_lines: Slider,
    show_traces: bool,
}

struct Endpoint {
    position: Vector2<u32>,
    previous_mouse: Option<Vector2<u32>>,
}

impl Endpoint {
    fn new(x: u32, y: u32) -> Self {
        Self {
            position: Vector2 { x, y },
            previous_mouse: None,
        }
    }
}

struct Slider {
    min: u32,
    max: u32,
    value: u32,
}

fn map_range(
    input: u32,
    input_start: u32,
    input_end: u32,
    output_start: u32,
    output_end: u32,
) -> u32 {
    ((output_end as f32 - output_start as f32) / (input_end as f32 - input_start as f32)
        * (input as f32 - input_start as f32)
        + output_start as f32)
        .clamp(output_start as f32, output_end as f32) as u32
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
