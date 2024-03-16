use justshow_x11::{atoms::AtomId, requests, XDisplay};

#[test]
fn get_invalid_atom() {
    let mut display = XDisplay::open().unwrap();
    let request = requests::GetAtomName {
        atom: AtomId::from(0),
    };
    let pending = display.send_request(&request).unwrap();
    let reply_or_error = display.await_pending_reply(pending).unwrap();

    assert!(reply_or_error.is_err());
}
