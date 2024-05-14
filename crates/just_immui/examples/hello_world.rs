use just_bdf::{Font, Glyph};
use just_canvas::{Color, Result, Vector2};
use just_immui::{
    draw::{
        background, inside_rectangle, invisible_button, rectangle, text_bdf, text_bdf_width, Button,
    },
    Ui, UiId,
};
use std::collections::HashMap;

/// Main UI loop
fn draw(ui: &mut Ui, state: &mut State) {
    let get_char = |c| state.char_map.get(c);

    background(ui.canvas(), Color::from_raw(0x222222));
    text_bdf(
        ui.canvas(),
        get_char,
        Vector2 { x: 50, y: 30 },
        3,
        "Hello, World!",
    );
    counter_button(
        ui,
        new_id(0),
        Vector2 { x: 50, y: 100 },
        &mut state.count_left,
        &get_char,
    );

    let right_button = counter_button(
        ui,
        new_id(1),
        Vector2 { x: 200, y: 100 },
        &mut state.count_right,
        &get_char,
    );
    // add additional action on click
    if right_button.clicked {
        println!("Right clicked!");
    }
}

fn ui() -> Result<()> {
    // load font compile time
    let font = just_bdf::parse(include_str!("ib8x8u.bdf")).unwrap();

    let char_map = BdfCharMap::new(font);

    let mut state = State {
        char_map,
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
    char_map: BdfCharMap,
    count_left: u32,
    count_right: u32,
}

fn main() {
    ui().unwrap();
}

/// Button with click counter - custom widget composed from simpler ones
fn counter_button<'a>(
    ui: &mut Ui,
    id: UiId,
    position: Vector2<u32>,
    state: &mut u32,
    font: impl Fn(char) -> &'a Glyph,
) -> Button {
    let inactive_color = Color::from_raw(0xf92672);
    let hot_color = Color::from_raw(0x4eb4fa);
    let active_color = Color::from_raw(0xa7e22e);

    let width = 120;
    let height = 40;
    let size = Vector2 {
        x: width,
        y: height,
    };
    let font_size = 2;
    let font_height = 8;

    let button = invisible_button(ui, id, |pointer| inside_rectangle(position, size, pointer));
    if button.clicked || button.pressed {
        rectangle(ui.canvas(), position, size, active_color);
    } else if button.active {
        rectangle(ui.canvas(), position, size, hot_color);
    } else {
        rectangle(ui.canvas(), position, size, inactive_color);
    }

    if button.clicked {
        *state += 1;
    }
    let txt = format!("{}", *state);
    let text_width = text_bdf_width(&font, 2, &txt);
    text_bdf(
        ui.canvas(),
        &font,
        Vector2 {
            x: position.x + (width / 2 - text_width / 2),
            y: position.y + (height / 2 - (font_size * font_height) / 2),
        },
        font_size,
        &txt,
    );

    button
}

// ad-hoc font handling, will be moved _somewhere_

struct BdfCharMap {
    glyphs: Vec<Glyph>,
    ascii: [usize; 128],
    map: HashMap<u32, usize>,
    default: usize,
}

impl BdfCharMap {
    pub fn new(font: Font) -> Self {
        let default = font.glyphs.len() - 1;
        let mut char_map = BdfCharMap {
            glyphs: font.glyphs,
            map: HashMap::new(),
            ascii: [default; 128],
            default,
        };

        for (idx, g) in char_map.glyphs.iter().enumerate() {
            match g.encoding {
                just_bdf::Encoding::AdobeStandard(enc) => {
                    if enc < 128 {
                        char_map.ascii[enc as usize] = idx;
                    }
                    char_map.map.insert(enc, idx);
                }
                _ => {}
            }
        }
        char_map
    }

    pub fn get(&self, c: char) -> &Glyph {
        let k = c as u32;
        if k < 128 {
            &self.glyphs[self.ascii[k as usize]]
        } else {
            let idx = self.map.get(&k).unwrap_or(&self.default);
            &self.glyphs[*idx]
        }
    }
}

fn new_id(id: u32) -> UiId {
    UiId {
        id,
        parent: 0,
        index: 0,
    }
}
