use justshow_x11::{error::Error, extensions::randr, replies, requests, XDisplay};

/// Send the same request twice and assert that replies are the same
/// This checks that reply decoder is not consuming too much/too little data
macro_rules! randr_double_request {
    ($mk_request:expr) => {{
        let mut display = XDisplay::open().unwrap();

        let extension = init(&mut display).unwrap();

        let request = $mk_request(&mut display);

        let pending_1 = display
            .send_extension_request(&request, extension.major_opcode)
            .unwrap();
        let pending_2 = display
            .send_extension_request(&request, extension.major_opcode)
            .unwrap();

        let reply_1 = display.await_pending_reply(pending_1).unwrap();
        let reply_2 = display.await_pending_reply(pending_2).unwrap();

        assert_eq!(reply_1, reply_2);
    }};
}

/// Initialize randr protocol
fn init(display: &mut XDisplay) -> Result<replies::QueryExtension, Error> {
    let extension = {
        let pending = display
            .send_request(&requests::QueryExtension {
                name: randr::EXTENSION_NAME.to_vec(),
            })
            .unwrap();
        display.flush().unwrap();
        display.await_pending_reply(pending).unwrap()
    };

    let pending_init = display.send_extension_request(
        &randr::requests::QueryVersion {
            major_version: randr::SUPPORTED_MAJOR,
            minor_version: randr::SUPPORTED_MINOR,
        },
        extension.major_opcode,
    )?;
    let init = display.await_pending_reply(pending_init)?;

    assert_eq!(init.major_version, randr::SUPPORTED_MAJOR);
    assert_eq!(init.minor_version, randr::SUPPORTED_MINOR);

    Ok(extension)
}

#[test]
fn query_version() {
    randr_double_request!(|_| randr::requests::QueryVersion {
        major_version: randr::SUPPORTED_MAJOR,
        minor_version: randr::SUPPORTED_MINOR,
    });
}

#[test]
fn get_monitor_info() {
    randr_double_request!(|display: &mut XDisplay| randr::requests::GetScreenInfo {
        window: display.screens()[0].root,
    });
}

#[test]
fn get_monitors() {
    randr_double_request!(|display: &mut XDisplay| randr::requests::GetMonitors {
        window: display.screens()[0].root,
        get_active: true,
    });

    randr_double_request!(|display: &mut XDisplay| randr::requests::GetMonitors {
        window: display.screens()[0].root,
        get_active: false,
    });
}

#[test]
fn get_get_screen_size_range() {
    randr_double_request!(
        |display: &mut XDisplay| randr::requests::GetScreenSizeRange {
            window: display.screens()[0].root,
        }
    );
}
