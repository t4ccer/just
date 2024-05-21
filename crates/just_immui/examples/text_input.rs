use just_canvas::{Result, Vector2};
use just_immui::{
    monokaish::{self, TextInput},
    Ui, UiId,
};

fn draw(ui: &mut Ui, left: &mut TextInput, right: &mut TextInput) {
    ui.background(monokaish::BLACK);

    left.draw(
        ui,
        UiId {
            id: 0,
            parent: 0,
            index: 0,
        },
        Vector2 { x: 100, y: 50 },
    );

    right.draw(
        ui,
        UiId {
            id: 1,
            parent: 0,
            index: 0,
        },
        Vector2 { x: 400, y: 50 },
    );
}

fn ui() -> Result<()> {
    let mut ui = Ui::new("Text input")?;
    ui.set_dirty();

    let mut left = TextInput {
        value: "Hello, World!".to_string(),
        cursor: 0,
    };
    let mut right = TextInput {
        value: "12.34".to_string(),
        cursor: 0,
    };
    ui.fps_limited_loop(60, |ui| draw(ui, &mut left, &mut right))?;
    Ok(())
}

fn main() {
    ui().unwrap();
}
