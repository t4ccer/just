use just_bdf::{Font, Glyph};
use just_immui::{
    background, invisible_button, rectangle, text_bdf, text_bdf_len, Button, Color, Context, Result,
};
use std::collections::HashMap;

/// Main UI loop
fn draw(ui: &mut Context, state: &mut State) {
    let get_char = |c| state.char_map.get(c);

    background(ui, Color::from_raw(0x222222));
    text_bdf(ui, get_char, 50, 30, 2, "Hello, World!");
    counter_button(ui, 50, 100, &mut state.count_left, &get_char);

    let right_button = counter_button(ui, 200, 100, &mut state.count_right, &get_char);
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
    let mut ui = Context::new("My Application")?;

    // run UI at 60 FPS
    ui.fps_limited_loop(60, |ui| draw(ui, &mut state))
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
    ui: &mut Context,
    x: u32,
    y: u32,
    state: &mut u32,
    font: impl Fn(char) -> &'a Glyph,
) -> Button {
    let inactive_color = Color::from_raw(0xf92672);
    let hot_color = Color::from_raw(0x4eb4fa);
    let active_color = Color::from_raw(0xa7e22e);

    let width = 120;
    let height = 40;

    let button = invisible_button(ui, x, y, width, height);
    if button.clicked || button.pressed {
        rectangle(ui, x, y, width, height, active_color);
    } else if button.active {
        rectangle(ui, x, y, width, height, hot_color);
    } else {
        rectangle(ui, x, y, width, height, inactive_color);
    }

    if button.clicked {
        *state += 1;
    }
    let txt = format!("{}", *state);
    let len = text_bdf_len(&font, 2, &txt);
    text_bdf(ui, &font, x + (width / 2 - len / 2), width - 8, 2, &txt);

    button
}

// ad-hoc font handling, will be moved _somewhere_

struct BdfCharMap {
    map: HashMap<u32, Glyph>,
    default: Glyph,
}

impl BdfCharMap {
    pub fn new(font: Font) -> Self {
        let mut char_map = BdfCharMap {
            map: HashMap::new(),
            default: font.glyphs.last().unwrap().clone(),
        };
        for g in font.glyphs {
            match g.encoding {
                just_bdf::Encoding::AdobeStandard(enc) => {
                    char_map.map.insert(enc, g);
                }
                _ => {}
            }
        }
        char_map
    }

    pub fn get(&self, c: char) -> &Glyph {
        self.map.get(&(c as u32)).unwrap_or(&self.default)
    }
}
