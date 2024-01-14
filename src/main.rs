use justshow::{error::Error, requests::GcParams, Rectangle, XDisplay};
use std::time::Duration;

pub fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;
    let window = display.create_window()?;
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
        let gc_settings = GcParams::new()
            .set_background(0x00000000)
            .set_foreground(0x00000000);
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

        let gc_settings = GcParams::new()
            .set_background(0xffffffff)
            .set_foreground(0xffffffff);
        gc.change(&mut display, gc_settings)?;
        window.draw_rectangle(&mut display, gc, rect)?;

        display.connection.flush()?;

        std::thread::sleep(Duration::from_millis(16));
    }
}

fn main() {
    go().unwrap();
}
