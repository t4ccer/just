use justshow_x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    requests::{self, WindowCreationAttributes},
    XDisplay,
};

pub fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;

    // TODO: dispaly.default_screen() as I'm not sure if `[0]` is correct
    let root = display.screens()[0].root;

    display.send_request(&requests::ChangeWindowAttributes {
        window: root,
        attributes: WindowCreationAttributes::new().set_event_mask(
            EventType::SUBSTRUCTURE_REDIRECT
                | EventType::SUBSTRUCTURE_NOTIFY
                | EventType::ENTER_WINDOW
                | EventType::LEAVE_WINDOW
                | EventType::STRUCTURE_NOTIFY
                | EventType::BUTTON_PRESS,
        ),
    })?;
    display.flush()?;
    // TODO: Handle error if some other client is already using SUBSTRUCTURE_REDIRECT

    'main_loop: loop {
        for error in display.errors() {
            dbg!(error);
        }

        for event in display.events()? {
            match event {
                SomeEvent::KeyPress(event) => match event.detail {
                    // q
                    24 => break 'main_loop,
                    // k
                    45 => {
                        std::process::Command::new("xterm").spawn()?;
                    }
                    _ => {}
                },
                _event => {}
            }
        }
    }

    Ok(())
}

fn main() {
    match go() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("justwindows: error: {}", err);
        }
    }
}
