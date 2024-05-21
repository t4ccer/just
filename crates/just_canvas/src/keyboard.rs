use just_x11::keysym::KeySym;

/// Not a character
#[derive(Debug)]
pub enum SpecialKeyboardButton {
    // TTY function keys
    BackSpace,
    Tab,
    Linefeed,
    Clear,
    Return,
    Pause,
    ScrollLock,
    SysReq,
    Escape,
    Delete,

    // International & multi-key character composition
    MultiKey,
    Codeinput,
    SingleCandidate,
    MultipleCandidate,
    PreviousCandidate,

    // TODO: Japanese keyboard support

    // Cursor control & motion
    Home,
    Left,
    Up,
    Right,
    Down,
    Prior,
    PageUp,
    Next,
    PageDown,
    End,
    Begin,

    // Misc functions
    Select,
    Print,
    Execute,
    Insert,
    Undo,
    Redo,
    Menu,
    Find,
    Cancel,
    Help,
    Break,
    ModeSwitch,
    ScriptSwitch,
    NumLock,

    // Auxiliary functions
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
    F32,
    F33,
    F34,
    F35,

    // Modifiers
    ShiftL,
    ShiftR,
    ControlL,
    ControlR,
    CapsLock,
    ShiftLock,
    MetaL,
    MetaR,
    AltL,
    AltR,
    SuperL,
    SuperR,
    HyperL,
    HyperR,
    // TODO: Keypad functions
}

#[derive(Debug)]
pub enum KeyboardButton {
    Special(SpecialKeyboardButton),
    Unicode(char),
}

impl TryFrom<KeySym> for KeyboardButton {
    type Error = KeySym;

    fn try_from(value: KeySym) -> Result<Self, Self::Error> {
        match value {
            KeySym::VOID_SYMBOL | KeySym::NO_SYMBOL => Err(value),

            // TTY function keys
            KeySym::BackSpace => Ok(Self::Special(SpecialKeyboardButton::BackSpace)),
            KeySym::Tab => Ok(Self::Special(SpecialKeyboardButton::Tab)),
            KeySym::Linefeed => Ok(Self::Special(SpecialKeyboardButton::Linefeed)),
            KeySym::Clear => Ok(Self::Special(SpecialKeyboardButton::Clear)),
            KeySym::Return => Ok(Self::Special(SpecialKeyboardButton::Return)),
            KeySym::Pause => Ok(Self::Special(SpecialKeyboardButton::Pause)),
            KeySym::Scroll_Lock => Ok(Self::Special(SpecialKeyboardButton::ScrollLock)),
            KeySym::Sys_Req => Ok(Self::Special(SpecialKeyboardButton::SysReq)),
            KeySym::Escape => Ok(Self::Special(SpecialKeyboardButton::Escape)),
            KeySym::Delete => Ok(Self::Special(SpecialKeyboardButton::Delete)),

            // International & multi-key character composition
            KeySym::Multi_key => Ok(Self::Special(SpecialKeyboardButton::MultiKey)),
            KeySym::Codeinput => Ok(Self::Special(SpecialKeyboardButton::Codeinput)),
            KeySym::SingleCandidate => Ok(Self::Special(SpecialKeyboardButton::SingleCandidate)),
            KeySym::MultipleCandidate => {
                Ok(Self::Special(SpecialKeyboardButton::MultipleCandidate))
            }
            KeySym::PreviousCandidate => {
                Ok(Self::Special(SpecialKeyboardButton::PreviousCandidate))
            }

            // TODO: Japanese keyboard support

            // Cursor control & motion
            KeySym::Home => Ok(Self::Special(SpecialKeyboardButton::Home)),
            KeySym::Left => Ok(Self::Special(SpecialKeyboardButton::Left)),
            KeySym::Up => Ok(Self::Special(SpecialKeyboardButton::Up)),
            KeySym::Right => Ok(Self::Special(SpecialKeyboardButton::Right)),
            KeySym::Down => Ok(Self::Special(SpecialKeyboardButton::Down)),
            KeySym::Prior => Ok(Self::Special(SpecialKeyboardButton::Prior)),
            KeySym::Next => Ok(Self::Special(SpecialKeyboardButton::Next)),
            KeySym::End => Ok(Self::Special(SpecialKeyboardButton::End)),
            KeySym::Begin => Ok(Self::Special(SpecialKeyboardButton::Begin)),

            // Misc functions
            KeySym::Select => Ok(Self::Special(SpecialKeyboardButton::Select)),
            KeySym::Print => Ok(Self::Special(SpecialKeyboardButton::Print)),
            KeySym::Execute => Ok(Self::Special(SpecialKeyboardButton::Execute)),
            KeySym::Insert => Ok(Self::Special(SpecialKeyboardButton::Insert)),
            KeySym::Undo => Ok(Self::Special(SpecialKeyboardButton::Undo)),
            KeySym::Redo => Ok(Self::Special(SpecialKeyboardButton::Redo)),
            KeySym::Menu => Ok(Self::Special(SpecialKeyboardButton::Menu)),
            KeySym::Find => Ok(Self::Special(SpecialKeyboardButton::Find)),
            KeySym::Cancel => Ok(Self::Special(SpecialKeyboardButton::Cancel)),
            KeySym::Help => Ok(Self::Special(SpecialKeyboardButton::Help)),
            KeySym::Break => Ok(Self::Special(SpecialKeyboardButton::Break)),
            KeySym::Mode_switch => Ok(Self::Special(SpecialKeyboardButton::ModeSwitch)),
            KeySym::Num_Lock => Ok(Self::Special(SpecialKeyboardButton::NumLock)),

            // Auxiliary functions
            KeySym::F1 => Ok(Self::Special(SpecialKeyboardButton::F1)),
            KeySym::F2 => Ok(Self::Special(SpecialKeyboardButton::F2)),
            KeySym::F3 => Ok(Self::Special(SpecialKeyboardButton::F3)),
            KeySym::F4 => Ok(Self::Special(SpecialKeyboardButton::F4)),
            KeySym::F5 => Ok(Self::Special(SpecialKeyboardButton::F5)),
            KeySym::F6 => Ok(Self::Special(SpecialKeyboardButton::F6)),
            KeySym::F7 => Ok(Self::Special(SpecialKeyboardButton::F7)),
            KeySym::F8 => Ok(Self::Special(SpecialKeyboardButton::F8)),
            KeySym::F9 => Ok(Self::Special(SpecialKeyboardButton::F9)),
            KeySym::F10 => Ok(Self::Special(SpecialKeyboardButton::F10)),
            KeySym::F11 => Ok(Self::Special(SpecialKeyboardButton::F11)),
            KeySym::F12 => Ok(Self::Special(SpecialKeyboardButton::F12)),
            KeySym::F13 => Ok(Self::Special(SpecialKeyboardButton::F13)),
            KeySym::F14 => Ok(Self::Special(SpecialKeyboardButton::F14)),
            KeySym::F15 => Ok(Self::Special(SpecialKeyboardButton::F15)),
            KeySym::F16 => Ok(Self::Special(SpecialKeyboardButton::F16)),
            KeySym::F17 => Ok(Self::Special(SpecialKeyboardButton::F17)),
            KeySym::F18 => Ok(Self::Special(SpecialKeyboardButton::F18)),
            KeySym::F19 => Ok(Self::Special(SpecialKeyboardButton::F19)),
            KeySym::F20 => Ok(Self::Special(SpecialKeyboardButton::F20)),
            KeySym::F21 => Ok(Self::Special(SpecialKeyboardButton::F21)),
            KeySym::F22 => Ok(Self::Special(SpecialKeyboardButton::F22)),
            KeySym::F23 => Ok(Self::Special(SpecialKeyboardButton::F23)),
            KeySym::F24 => Ok(Self::Special(SpecialKeyboardButton::F24)),
            KeySym::F25 => Ok(Self::Special(SpecialKeyboardButton::F25)),
            KeySym::F26 => Ok(Self::Special(SpecialKeyboardButton::F26)),
            KeySym::F27 => Ok(Self::Special(SpecialKeyboardButton::F27)),
            KeySym::F28 => Ok(Self::Special(SpecialKeyboardButton::F28)),
            KeySym::F29 => Ok(Self::Special(SpecialKeyboardButton::F29)),
            KeySym::F30 => Ok(Self::Special(SpecialKeyboardButton::F30)),
            KeySym::F31 => Ok(Self::Special(SpecialKeyboardButton::F31)),
            KeySym::F32 => Ok(Self::Special(SpecialKeyboardButton::F32)),
            KeySym::F33 => Ok(Self::Special(SpecialKeyboardButton::F33)),
            KeySym::F34 => Ok(Self::Special(SpecialKeyboardButton::F34)),
            KeySym::F35 => Ok(Self::Special(SpecialKeyboardButton::F35)),

            // Modifiers
            KeySym::Shift_L => Ok(Self::Special(SpecialKeyboardButton::ShiftL)),
            KeySym::Shift_R => Ok(Self::Special(SpecialKeyboardButton::ShiftR)),
            KeySym::Control_L => Ok(Self::Special(SpecialKeyboardButton::ControlL)),
            KeySym::Control_R => Ok(Self::Special(SpecialKeyboardButton::ControlR)),
            KeySym::Caps_Lock => Ok(Self::Special(SpecialKeyboardButton::CapsLock)),
            KeySym::Shift_Lock => Ok(Self::Special(SpecialKeyboardButton::ShiftLock)),
            KeySym::Meta_L => Ok(Self::Special(SpecialKeyboardButton::MetaL)),
            KeySym::Meta_R => Ok(Self::Special(SpecialKeyboardButton::MetaR)),
            KeySym::Alt_L => Ok(Self::Special(SpecialKeyboardButton::AltL)),
            KeySym::Alt_R => Ok(Self::Special(SpecialKeyboardButton::AltR)),
            KeySym::Super_L => Ok(Self::Special(SpecialKeyboardButton::SuperL)),
            KeySym::Super_R => Ok(Self::Special(SpecialKeyboardButton::SuperR)),
            KeySym::Hyper_L => Ok(Self::Special(SpecialKeyboardButton::HyperL)),
            KeySym::Hyper_R => Ok(Self::Special(SpecialKeyboardButton::HyperR)),

            // Everything else is unicode (or invalid but we just ignore)
            _ => char::from_u32(value.inner).ok_or(value).map(Self::Unicode),
        }
    }
}
