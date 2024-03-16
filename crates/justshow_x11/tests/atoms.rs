use std::str::FromStr;

use justshow_x11::{
    atoms::AtomId,
    replies::{self, String8},
    requests,
    xerror::SomeError,
    XDisplay,
};

#[test]
fn get_predefined_atom() {
    let mut display = XDisplay::open().unwrap();
    let request = requests::GetAtomName {
        atom: AtomId::SUPERSCRIPT_X,
    };
    let pending = display.send_request(&request).unwrap();
    let reply_or_error = display.await_pending_reply(pending).unwrap();
    let reply = reply_or_error.unwrap();

    assert_eq!(
        reply,
        replies::GetAtomName {
            name: String8::from_str("SUPERSCRIPT_X").unwrap()
        }
    )
}

#[test]
fn get_invalid_atom() {
    let mut display = XDisplay::open().unwrap();
    let request = requests::GetAtomName {
        atom: AtomId::from(0),
    };
    let pending = display.send_request(&request).unwrap();
    let reply_or_error = display.await_pending_reply(pending).unwrap();

    let err = reply_or_error.unwrap_err();
    if let SomeError::Atom(err) = err {
        assert_eq!(err.bad_atom_id(), 0);
    } else {
        panic!("Expected Atom error, instead got {:?}", err);
    }
}
