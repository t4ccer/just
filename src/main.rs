use justshow::{error::Error, requests::GcParams, Rectangle, XDisplay};
use std::time::Duration;

pub fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;
    let window = display.create_simple_window()?;
    window.map(&mut display)?;

    let gc = window.create_gc(&mut display, GcParams::new())?;

    let mut y = 0;

    let background = Rectangle {
        x: 0,
        y: 0,
        width: 0x1000,
        height: 0x1000,
    };

    loop {
        // Background
        let gc_settings = GcParams::new().set_foreground(0x222222);
        gc.change(&mut display, gc_settings)?;
        window.draw_rectangle(&mut display, gc, background)?;

        // Left
        let rect = Rectangle {
            x: 75,
            y,
            width: 100,
            height: 200,
        };
        y = (y + 5) % 1000;

        let gc_settings = GcParams::new().set_foreground(0x4eb4fa);
        gc.change(&mut display, gc_settings)?;
        window.draw_rectangle(&mut display, gc, rect)?;

        display.connection.flush()?;

        // let window_params = window.get_attributes(&mut display)?;
        // dbg!(window_params);

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn main() {
    go().unwrap();
}
