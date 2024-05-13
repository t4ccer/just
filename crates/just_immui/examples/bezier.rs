use just_immui::{
    draw::{
        background, circle, inside_circle, inside_rectangle, rectangle, thin_dashed_line, thin_line,
    },
    Color, Context, Pointer, Result, UiId,
};

const BLUE: Color = Color::from_raw(0xff4eb4fa);
const RED: Color = Color::from_raw(0xfff92672);
const BLACK: Color = Color::from_raw(0xff222222);
const WHITE: Color = Color::from_raw(0xffdddddd);
const GRAY: Color = Color::from_raw(0x666666);

fn draw(ui: &mut Context, state: &mut State) {
    background(ui, BLACK);

    thin_dashed_line(
        ui,
        state.start_position.x,
        state.start_position.y,
        state.middle_position.x,
        state.middle_position.y,
        RED,
    );

    thin_dashed_line(
        ui,
        state.middle_position.x,
        state.middle_position.y,
        state.end_position.x,
        state.end_position.y,
        RED,
    );

    for t in 1..state.trace_lines.value {
        let t = t as f32 / state.trace_lines.value as f32;

        let (x1, y1) = linear_interpolation(
            state.start_position.x,
            state.start_position.y,
            state.middle_position.x,
            state.middle_position.y,
            t,
        );

        let (x2, y2) = linear_interpolation(
            state.middle_position.x,
            state.middle_position.y,
            state.end_position.x,
            state.end_position.y,
            t,
        );

        thin_line(ui, x1, y1, x2, y2, BLUE);
    }

    endpoint(ui, &mut state.start_position);
    endpoint(ui, &mut state.middle_position);
    endpoint(ui, &mut state.end_position);

    slider(ui, &mut state.trace_lines, 30, 30);
}

fn ui() -> Result<()> {
    let mut state = State {
        start_position: Endpoint::new(100, 100),
        middle_position: Endpoint::new(300, 400),
        end_position: Endpoint::new(600, 150),
        trace_lines: Slider {
            min: 0,
            max: 50,
            value: 25,
        },
    };
    let mut ui = Context::new("Bezier")?;

    // Run UI at 60 FPS
    ui.fps_limited_loop(60, |ui| draw(ui, &mut state))
}

fn slider(ui: &mut Context, state: &mut Slider, x: u32, y: u32) {
    // chosen arbitrarily
    let path_width = 180;
    let path_height = 6;

    let handle_width = 8;
    let handle_height = 20;

    rectangle(ui, x, y, path_width, path_height, GRAY);

    let handle_x = map_range(state.value, state.min, state.max, x, x + path_width);
    let handle_y = y - handle_height / 2 + path_height / 2;

    rectangle(ui, handle_x, handle_y, handle_width, handle_height, BLUE);

    let id = ui.next_id();
    let dragged = is_dragged(ui, id, |pointer| {
        inside_rectangle(
            handle_x,
            handle_y,
            handle_width,
            handle_height,
            pointer.x,
            pointer.y,
        )
    });
    if dragged {
        let px = (ui.pointer().x as i32).clamp(x as i32, x as i32 + path_width as i32) as u32;
        state.value = map_range(px, x, x + path_width, state.min, state.max);
    }
}

fn endpoint(ui: &mut Context, state: &mut Endpoint) {
    // chosen arbitrarily
    let r = 18;

    circle(ui, state.x, state.y, r, WHITE);
    circle(ui, state.x, state.y, r - 6, BLACK);

    let button_id = ui.next_id();
    let dragged = is_dragged(ui, button_id, |pointer| {
        inside_circle(state.x, state.y, r, pointer.x, pointer.y)
    });

    if dragged {
        // FIXME: Don't snap center to pointer (track prev and curr mouse pos)
        state.x = ui.pointer().x;
        state.y = ui.pointer().y;
    }
}

fn is_dragged(ui: &mut Context, button_id: UiId, is_over: impl FnOnce(&Pointer) -> bool) -> bool {
    if is_over(ui.pointer()) {
        if !ui.is_hot(button_id) && ui.pointer().is_pressed(1) {
            false
        } else {
            ui.make_hot(button_id);
            if ui.pointer().is_pressed(1) {
                ui.make_active(button_id);
                true
            } else {
                false
            }
        }
    } else {
        if ui.is_active(button_id) && ui.pointer().is_pressed(1) {
            true
        } else {
            ui.make_inactive(button_id);
            false
        }
    }
}

fn linear_interpolation(x1: u32, y1: u32, x2: u32, y2: u32, t: f32) -> (u32, u32) {
    let mut x = x1 as f32 + (x2 as f32 - x1 as f32) * t;
    if x < 0.0 {
        x = 0.0;
    }
    let x = x as u32;

    let mut y = y1 as f32 + (y2 as f32 - y1 as f32) * t;
    if y < 0.0 {
        y = 0.0;
    }
    let y = y as u32;

    (x, y)
}

struct State {
    start_position: Endpoint,
    middle_position: Endpoint,
    end_position: Endpoint,
    trace_lines: Slider,
}

struct Endpoint {
    x: u32,
    y: u32,
}

impl Endpoint {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y }
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
