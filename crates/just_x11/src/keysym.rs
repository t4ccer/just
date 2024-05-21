#![allow(non_upper_case_globals)]

use crate::{connection::XConnection, error::Error, FromLeBytes};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct KeySym {
    pub inner: u32,
}

impl std::fmt::Debug for KeySym {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Move to `field_with` when stable
        write!(f, "KeySym(0x{:08x})", self.inner)
    }
}

// NOTE: I don't like this but are required to make things in justshow_x11_simple::keys easier

impl Add for KeySym {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner,
        }
    }
}

impl Sub for KeySym {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner - rhs.inner,
        }
    }
}

impl AddAssign for KeySym {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner
    }
}

impl SubAssign for KeySym {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner
    }
}

impl FromLeBytes for KeySym {
    fn from_le_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let inner = conn.read_le_u32()?;
        Ok(Self { inner })
    }
}

impl KeySym {
    #[inline(always)]
    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
        self.inner.to_le_bytes()
    }

    pub const NO_SYMBOL: Self = Self { inner: 0x00000000 };
    pub const VOID_SYMBOL: Self = Self { inner: 0x00ffffff };

    // from xorgproto/include/X11/keysymdef.h

    // TODO: Follow upper/lower case convention

    /// Back space, back char
    pub const BackSpace: Self = Self { inner: 0xff08 };

    pub const Tab: Self = Self { inner: 0xff09 };

    /// Linefeed, LF
    pub const Linefeed: Self = Self { inner: 0xff0a };

    pub const Clear: Self = Self { inner: 0xff0b };

    /// Return, enter
    pub const Return: Self = Self { inner: 0xff0d };

    /// Pause, hold
    pub const Pause: Self = Self { inner: 0xff13 };

    pub const Scroll_Lock: Self = Self { inner: 0xff14 };

    pub const Sys_Req: Self = Self { inner: 0xff15 };

    pub const Escape: Self = Self { inner: 0xff1b };

    /// Delete, rubout
    pub const Delete: Self = Self { inner: 0xffff };

    /// Multi-key character compose
    pub const Multi_key: Self = Self { inner: 0xff20 };

    pub const Codeinput: Self = Self { inner: 0xff37 };

    pub const SingleCandidate: Self = Self { inner: 0xff3c };

    pub const MultipleCandidate: Self = Self { inner: 0xff3d };

    pub const PreviousCandidate: Self = Self { inner: 0xff3e };

    /// Kanji, Kanji convert
    pub const Kanji: Self = Self { inner: 0xff21 };

    /// Cancel Conversion
    pub const Muhenkan: Self = Self { inner: 0xff22 };

    /// Start/Stop Conversion
    pub const Henkan_Mode: Self = Self { inner: 0xff23 };

    /// Alias for Henkan_Mode
    pub const Henkan: Self = Self { inner: 0xff23 };

    /// to Romaji
    pub const Romaji: Self = Self { inner: 0xff24 };

    /// to Hiragana
    pub const Hiragana: Self = Self { inner: 0xff25 };

    /// to Katakana
    pub const Katakana: Self = Self { inner: 0xff26 };

    /// Hiragana/Katakana toggle
    pub const Hiragana_Katakana: Self = Self { inner: 0xff27 };

    /// to Zenkaku
    pub const Zenkaku: Self = Self { inner: 0xff28 };

    /// to Hankaku
    pub const Hankaku: Self = Self { inner: 0xff29 };

    /// Zenkaku/Hankaku toggle
    pub const Zenkaku_Hankaku: Self = Self { inner: 0xff2a };

    /// Add to Dictionary
    pub const Touroku: Self = Self { inner: 0xff2b };

    /// Delete from Dictionary
    pub const Massyo: Self = Self { inner: 0xff2c };

    /// Kana Lock
    pub const Kana_Lock: Self = Self { inner: 0xff2d };

    /// Kana Shift
    pub const Kana_Shift: Self = Self { inner: 0xff2e };

    /// Alphanumeric Shift
    pub const Eisu_Shift: Self = Self { inner: 0xff2f };

    /// Alphanumeric toggle
    pub const Eisu_toggle: Self = Self { inner: 0xff30 };

    /// Codeinput
    pub const Kanji_Bangou: Self = Self { inner: 0xff37 };

    /// Multiple/All Candidate(s)
    pub const Zen_Koho: Self = Self { inner: 0xff3d };

    /// Previous Candidate
    pub const Mae_Koho: Self = Self { inner: 0xff3e };

    pub const Home: Self = Self { inner: 0xff50 };

    /// Move left, left arrow
    pub const Left: Self = Self { inner: 0xff51 };

    /// Move up, up arrow
    pub const Up: Self = Self { inner: 0xff52 };

    /// Move right, right arrow
    pub const Right: Self = Self { inner: 0xff53 };

    /// Move down, down arrow
    pub const Down: Self = Self { inner: 0xff54 };

    /// Prior, previous
    pub const Prior: Self = Self { inner: 0xff55 };

    pub const Page_Up: Self = Self { inner: 0xff55 };

    /// Next
    pub const Next: Self = Self { inner: 0xff56 };

    pub const Page_Down: Self = Self { inner: 0xff56 };

    /// EOL
    pub const End: Self = Self { inner: 0xff57 };

    /// BOL
    pub const Begin: Self = Self { inner: 0xff58 };

    /// Select, mark
    pub const Select: Self = Self { inner: 0xff60 };

    pub const Print: Self = Self { inner: 0xff61 };

    /// Execute, run, do
    pub const Execute: Self = Self { inner: 0xff62 };

    /// Insert, insert here
    pub const Insert: Self = Self { inner: 0xff63 };

    pub const Undo: Self = Self { inner: 0xff65 };

    /// Redo, again
    pub const Redo: Self = Self { inner: 0xff66 };

    pub const Menu: Self = Self { inner: 0xff67 };

    /// Find, search
    pub const Find: Self = Self { inner: 0xff68 };

    /// Cancel, stop, abort, exit
    pub const Cancel: Self = Self { inner: 0xff69 };

    /// Help
    pub const Help: Self = Self { inner: 0xff6a };

    pub const Break: Self = Self { inner: 0xff6b };

    /// Character set switch
    pub const Mode_switch: Self = Self { inner: 0xff7e };

    /// Alias for mode_switch
    pub const script_switch: Self = Self { inner: 0xff7e };

    pub const Num_Lock: Self = Self { inner: 0xff7f };

    /// Space
    pub const KP_Space: Self = Self { inner: 0xff80 };

    pub const KP_Tab: Self = Self { inner: 0xff89 };

    /// Enter
    pub const KP_Enter: Self = Self { inner: 0xff8d };

    /// PF1, KP_A, ...
    pub const KP_F1: Self = Self { inner: 0xff91 };

    pub const KP_F2: Self = Self { inner: 0xff92 };

    pub const KP_F3: Self = Self { inner: 0xff93 };

    pub const KP_F4: Self = Self { inner: 0xff94 };

    pub const KP_Home: Self = Self { inner: 0xff95 };

    pub const KP_Left: Self = Self { inner: 0xff96 };

    pub const KP_Up: Self = Self { inner: 0xff97 };

    pub const KP_Right: Self = Self { inner: 0xff98 };

    pub const KP_Down: Self = Self { inner: 0xff99 };

    pub const KP_Prior: Self = Self { inner: 0xff9a };

    pub const KP_Page_Up: Self = Self { inner: 0xff9a };

    pub const KP_Next: Self = Self { inner: 0xff9b };

    pub const KP_Page_Down: Self = Self { inner: 0xff9b };

    pub const KP_End: Self = Self { inner: 0xff9c };

    pub const KP_Begin: Self = Self { inner: 0xff9d };

    pub const KP_Insert: Self = Self { inner: 0xff9e };

    pub const KP_Delete: Self = Self { inner: 0xff9f };

    /// Equals
    pub const KP_Equal: Self = Self { inner: 0xffbd };

    pub const KP_Multiply: Self = Self { inner: 0xffaa };

    pub const KP_Add: Self = Self { inner: 0xffab };

    /// Separator, often comma
    pub const KP_Separator: Self = Self { inner: 0xffac };

    pub const KP_Subtract: Self = Self { inner: 0xffad };

    pub const KP_Decimal: Self = Self { inner: 0xffae };

    pub const KP_Divide: Self = Self { inner: 0xffaf };

    pub const KP_0: Self = Self { inner: 0xffb0 };

    pub const KP_1: Self = Self { inner: 0xffb1 };

    pub const KP_2: Self = Self { inner: 0xffb2 };

    pub const KP_3: Self = Self { inner: 0xffb3 };

    pub const KP_4: Self = Self { inner: 0xffb4 };

    pub const KP_5: Self = Self { inner: 0xffb5 };

    pub const KP_6: Self = Self { inner: 0xffb6 };

    pub const KP_7: Self = Self { inner: 0xffb7 };

    pub const KP_8: Self = Self { inner: 0xffb8 };

    pub const KP_9: Self = Self { inner: 0xffb9 };

    pub const F1: Self = Self { inner: 0xffbe };

    pub const F2: Self = Self { inner: 0xffbf };

    pub const F3: Self = Self { inner: 0xffc0 };

    pub const F4: Self = Self { inner: 0xffc1 };

    pub const F5: Self = Self { inner: 0xffc2 };

    pub const F6: Self = Self { inner: 0xffc3 };

    pub const F7: Self = Self { inner: 0xffc4 };

    pub const F8: Self = Self { inner: 0xffc5 };

    pub const F9: Self = Self { inner: 0xffc6 };

    pub const F10: Self = Self { inner: 0xffc7 };

    pub const F11: Self = Self { inner: 0xffc8 };

    pub const L1: Self = Self { inner: 0xffc8 };

    pub const F12: Self = Self { inner: 0xffc9 };

    pub const L2: Self = Self { inner: 0xffc9 };

    pub const F13: Self = Self { inner: 0xffca };

    pub const L3: Self = Self { inner: 0xffca };

    pub const F14: Self = Self { inner: 0xffcb };

    pub const L4: Self = Self { inner: 0xffcb };

    pub const F15: Self = Self { inner: 0xffcc };

    pub const L5: Self = Self { inner: 0xffcc };

    pub const F16: Self = Self { inner: 0xffcd };

    pub const L6: Self = Self { inner: 0xffcd };

    pub const F17: Self = Self { inner: 0xffce };

    pub const L7: Self = Self { inner: 0xffce };

    pub const F18: Self = Self { inner: 0xffcf };

    pub const L8: Self = Self { inner: 0xffcf };

    pub const F19: Self = Self { inner: 0xffd0 };

    pub const L9: Self = Self { inner: 0xffd0 };

    pub const F20: Self = Self { inner: 0xffd1 };

    pub const L10: Self = Self { inner: 0xffd1 };

    pub const F21: Self = Self { inner: 0xffd2 };

    pub const R1: Self = Self { inner: 0xffd2 };

    pub const F22: Self = Self { inner: 0xffd3 };

    pub const R2: Self = Self { inner: 0xffd3 };

    pub const F23: Self = Self { inner: 0xffd4 };

    pub const R3: Self = Self { inner: 0xffd4 };

    pub const F24: Self = Self { inner: 0xffd5 };

    pub const R4: Self = Self { inner: 0xffd5 };

    pub const F25: Self = Self { inner: 0xffd6 };

    pub const R5: Self = Self { inner: 0xffd6 };

    pub const F26: Self = Self { inner: 0xffd7 };

    pub const R6: Self = Self { inner: 0xffd7 };

    pub const F27: Self = Self { inner: 0xffd8 };

    pub const R7: Self = Self { inner: 0xffd8 };

    pub const F28: Self = Self { inner: 0xffd9 };

    pub const R8: Self = Self { inner: 0xffd9 };

    pub const F29: Self = Self { inner: 0xffda };

    pub const R9: Self = Self { inner: 0xffda };

    pub const F30: Self = Self { inner: 0xffdb };

    pub const R10: Self = Self { inner: 0xffdb };

    pub const F31: Self = Self { inner: 0xffdc };

    pub const R11: Self = Self { inner: 0xffdc };

    pub const F32: Self = Self { inner: 0xffdd };

    pub const R12: Self = Self { inner: 0xffdd };

    pub const F33: Self = Self { inner: 0xffde };

    pub const R13: Self = Self { inner: 0xffde };

    pub const F34: Self = Self { inner: 0xffdf };

    pub const R14: Self = Self { inner: 0xffdf };

    pub const F35: Self = Self { inner: 0xffe0 };

    pub const R15: Self = Self { inner: 0xffe0 };

    /// Left shift
    pub const Shift_L: Self = Self { inner: 0xffe1 };

    /// Right shift
    pub const Shift_R: Self = Self { inner: 0xffe2 };

    /// Left control
    pub const Control_L: Self = Self { inner: 0xffe3 };

    /// Right control
    pub const Control_R: Self = Self { inner: 0xffe4 };

    /// Caps lock
    pub const Caps_Lock: Self = Self { inner: 0xffe5 };

    /// Shift lock
    pub const Shift_Lock: Self = Self { inner: 0xffe6 };

    /// Left meta
    pub const Meta_L: Self = Self { inner: 0xffe7 };

    /// Right meta
    pub const Meta_R: Self = Self { inner: 0xffe8 };

    /// Left alt
    pub const Alt_L: Self = Self { inner: 0xffe9 };

    /// Right alt
    pub const Alt_R: Self = Self { inner: 0xffea };

    /// Left super
    pub const Super_L: Self = Self { inner: 0xffeb };

    /// Right super
    pub const Super_R: Self = Self { inner: 0xffec };

    /// Left hyper
    pub const Hyper_L: Self = Self { inner: 0xffed };

    /// Right hyper
    pub const Hyper_R: Self = Self { inner: 0xffee };

    pub const ISO_Lock: Self = Self { inner: 0xfe01 };

    pub const ISO_Level2_Latch: Self = Self { inner: 0xfe02 };

    pub const ISO_Level3_Shift: Self = Self { inner: 0xfe03 };

    pub const ISO_Level3_Latch: Self = Self { inner: 0xfe04 };

    pub const ISO_Level3_Lock: Self = Self { inner: 0xfe05 };

    pub const ISO_Level5_Shift: Self = Self { inner: 0xfe11 };

    pub const ISO_Level5_Latch: Self = Self { inner: 0xfe12 };

    pub const ISO_Level5_Lock: Self = Self { inner: 0xfe13 };

    /// Alias for mode_switch
    pub const ISO_Group_Shift: Self = Self { inner: 0xff7e };

    pub const ISO_Group_Latch: Self = Self { inner: 0xfe06 };

    pub const ISO_Group_Lock: Self = Self { inner: 0xfe07 };

    pub const ISO_Next_Group: Self = Self { inner: 0xfe08 };

    pub const ISO_Next_Group_Lock: Self = Self { inner: 0xfe09 };

    pub const ISO_Prev_Group: Self = Self { inner: 0xfe0a };

    pub const ISO_Prev_Group_Lock: Self = Self { inner: 0xfe0b };

    pub const ISO_First_Group: Self = Self { inner: 0xfe0c };

    pub const ISO_First_Group_Lock: Self = Self { inner: 0xfe0d };

    pub const ISO_Last_Group: Self = Self { inner: 0xfe0e };

    pub const ISO_Last_Group_Lock: Self = Self { inner: 0xfe0f };

    pub const ISO_Left_Tab: Self = Self { inner: 0xfe20 };

    pub const ISO_Move_Line_Up: Self = Self { inner: 0xfe21 };

    pub const ISO_Move_Line_Down: Self = Self { inner: 0xfe22 };

    pub const ISO_Partial_Line_Up: Self = Self { inner: 0xfe23 };

    pub const ISO_Partial_Line_Down: Self = Self { inner: 0xfe24 };

    pub const ISO_Partial_Space_Left: Self = Self { inner: 0xfe25 };

    pub const ISO_Partial_Space_Right: Self = Self { inner: 0xfe26 };

    pub const ISO_Set_Margin_Left: Self = Self { inner: 0xfe27 };

    pub const ISO_Set_Margin_Right: Self = Self { inner: 0xfe28 };

    pub const ISO_Release_Margin_Left: Self = Self { inner: 0xfe29 };

    pub const ISO_Release_Margin_Right: Self = Self { inner: 0xfe2a };

    pub const ISO_Release_Both_Margins: Self = Self { inner: 0xfe2b };

    pub const ISO_Fast_Cursor_Left: Self = Self { inner: 0xfe2c };

    pub const ISO_Fast_Cursor_Right: Self = Self { inner: 0xfe2d };

    pub const ISO_Fast_Cursor_Up: Self = Self { inner: 0xfe2e };

    pub const ISO_Fast_Cursor_Down: Self = Self { inner: 0xfe2f };

    pub const ISO_Continuous_Underline: Self = Self { inner: 0xfe30 };

    pub const ISO_Discontinuous_Underline: Self = Self { inner: 0xfe31 };

    pub const ISO_Emphasize: Self = Self { inner: 0xfe32 };

    pub const ISO_Center_Object: Self = Self { inner: 0xfe33 };

    pub const ISO_Enter: Self = Self { inner: 0xfe34 };

    pub const dead_grave: Self = Self { inner: 0xfe50 };

    pub const dead_acute: Self = Self { inner: 0xfe51 };

    pub const dead_circumflex: Self = Self { inner: 0xfe52 };

    pub const dead_tilde: Self = Self { inner: 0xfe53 };

    pub const dead_macron: Self = Self { inner: 0xfe54 };

    pub const dead_breve: Self = Self { inner: 0xfe55 };

    pub const dead_abovedot: Self = Self { inner: 0xfe56 };

    pub const dead_diaeresis: Self = Self { inner: 0xfe57 };

    pub const dead_abovering: Self = Self { inner: 0xfe58 };

    pub const dead_doubleacute: Self = Self { inner: 0xfe59 };

    pub const dead_caron: Self = Self { inner: 0xfe5a };

    pub const dead_cedilla: Self = Self { inner: 0xfe5b };

    pub const dead_ogonek: Self = Self { inner: 0xfe5c };

    pub const dead_iota: Self = Self { inner: 0xfe5d };

    pub const dead_voiced_sound: Self = Self { inner: 0xfe5e };

    pub const dead_semivoiced_sound: Self = Self { inner: 0xfe5f };

    pub const dead_belowdot: Self = Self { inner: 0xfe60 };

    pub const dead_hook: Self = Self { inner: 0xfe61 };

    pub const dead_horn: Self = Self { inner: 0xfe62 };

    pub const First_Virtual_Screen: Self = Self { inner: 0xfed0 };

    pub const Prev_Virtual_Screen: Self = Self { inner: 0xfed1 };

    pub const Next_Virtual_Screen: Self = Self { inner: 0xfed2 };

    pub const Last_Virtual_Screen: Self = Self { inner: 0xfed4 };

    pub const Terminate_Server: Self = Self { inner: 0xfed5 };

    pub const AccessX_Enable: Self = Self { inner: 0xfe70 };

    pub const AccessX_Feedback_Enable: Self = Self { inner: 0xfe71 };

    pub const RepeatKeys_Enable: Self = Self { inner: 0xfe72 };

    pub const SlowKeys_Enable: Self = Self { inner: 0xfe73 };

    pub const BounceKeys_Enable: Self = Self { inner: 0xfe74 };

    pub const StickyKeys_Enable: Self = Self { inner: 0xfe75 };

    pub const MouseKeys_Enable: Self = Self { inner: 0xfe76 };

    pub const MouseKeys_Accel_Enable: Self = Self { inner: 0xfe77 };

    pub const Overlay1_Enable: Self = Self { inner: 0xfe78 };

    pub const Overlay2_Enable: Self = Self { inner: 0xfe79 };

    pub const AudibleBell_Enable: Self = Self { inner: 0xfe7a };

    pub const Pointer_Left: Self = Self { inner: 0xfee0 };

    pub const Pointer_Right: Self = Self { inner: 0xfee1 };

    pub const Pointer_Up: Self = Self { inner: 0xfee2 };

    pub const Pointer_Down: Self = Self { inner: 0xfee3 };

    pub const Pointer_UpLeft: Self = Self { inner: 0xfee4 };

    pub const Pointer_UpRight: Self = Self { inner: 0xfee5 };

    pub const Pointer_DownLeft: Self = Self { inner: 0xfee6 };

    pub const Pointer_DownRight: Self = Self { inner: 0xfee7 };

    pub const Pointer_Button_Dflt: Self = Self { inner: 0xfee8 };

    pub const Pointer_Button1: Self = Self { inner: 0xfee9 };

    pub const Pointer_Button2: Self = Self { inner: 0xfeea };

    pub const Pointer_Button3: Self = Self { inner: 0xfeeb };

    pub const Pointer_Button4: Self = Self { inner: 0xfeec };

    pub const Pointer_Button5: Self = Self { inner: 0xfeed };

    pub const Pointer_DblClick_Dflt: Self = Self { inner: 0xfeee };

    pub const Pointer_DblClick1: Self = Self { inner: 0xfeef };

    pub const Pointer_DblClick2: Self = Self { inner: 0xfef0 };

    pub const Pointer_DblClick3: Self = Self { inner: 0xfef1 };

    pub const Pointer_DblClick4: Self = Self { inner: 0xfef2 };

    pub const Pointer_DblClick5: Self = Self { inner: 0xfef3 };

    pub const Pointer_Drag_Dflt: Self = Self { inner: 0xfef4 };

    pub const Pointer_Drag1: Self = Self { inner: 0xfef5 };

    pub const Pointer_Drag2: Self = Self { inner: 0xfef6 };

    pub const Pointer_Drag3: Self = Self { inner: 0xfef7 };

    pub const Pointer_Drag4: Self = Self { inner: 0xfef8 };

    pub const Pointer_Drag5: Self = Self { inner: 0xfefd };

    pub const Pointer_EnableKeys: Self = Self { inner: 0xfef9 };

    pub const Pointer_Accelerate: Self = Self { inner: 0xfefa };

    pub const Pointer_DfltBtnNext: Self = Self { inner: 0xfefb };

    pub const Pointer_DfltBtnPrev: Self = Self { inner: 0xfefc };

    pub const IBM_3270_Duplicate: Self = Self { inner: 0xfd01 };

    pub const IBM_3270_FieldMark: Self = Self { inner: 0xfd02 };

    pub const IBM_3270_Right2: Self = Self { inner: 0xfd03 };

    pub const IBM_3270_Left2: Self = Self { inner: 0xfd04 };

    pub const IBM_3270_BackTab: Self = Self { inner: 0xfd05 };

    pub const IBM_3270_EraseEOF: Self = Self { inner: 0xfd06 };

    pub const IBM_3270_EraseInput: Self = Self { inner: 0xfd07 };

    pub const IBM_3270_Reset: Self = Self { inner: 0xfd08 };

    pub const IBM_3270_Quit: Self = Self { inner: 0xfd09 };

    pub const IBM_3270_PA1: Self = Self { inner: 0xfd0a };

    pub const IBM_3270_PA2: Self = Self { inner: 0xfd0b };

    pub const IBM_3270_PA3: Self = Self { inner: 0xfd0c };

    pub const IBM_3270_Test: Self = Self { inner: 0xfd0d };

    pub const IBM_3270_Attn: Self = Self { inner: 0xfd0e };

    pub const IBM_3270_CursorBlink: Self = Self { inner: 0xfd0f };

    pub const IBM_3270_AltCursor: Self = Self { inner: 0xfd10 };

    pub const IBM_3270_KeyClick: Self = Self { inner: 0xfd11 };

    pub const IBM_3270_Jump: Self = Self { inner: 0xfd12 };

    pub const IBM_3270_Ident: Self = Self { inner: 0xfd13 };

    pub const IBM_3270_Rule: Self = Self { inner: 0xfd14 };

    pub const IBM_3270_Copy: Self = Self { inner: 0xfd15 };

    pub const IBM_3270_Play: Self = Self { inner: 0xfd16 };

    pub const IBM_3270_Setup: Self = Self { inner: 0xfd17 };

    pub const IBM_3270_Record: Self = Self { inner: 0xfd18 };

    pub const IBM_3270_ChangeScreen: Self = Self { inner: 0xfd19 };

    pub const IBM_3270_DeleteWord: Self = Self { inner: 0xfd1a };

    pub const IBM_3270_ExSelect: Self = Self { inner: 0xfd1b };

    pub const IBM_3270_CursorSelect: Self = Self { inner: 0xfd1c };

    pub const IBM_3270_PrintScreen: Self = Self { inner: 0xfd1d };

    pub const IBM_3270_Enter: Self = Self { inner: 0xfd1e };

    /// U+0020 SPACE
    pub const space: Self = Self { inner: 0x0020 };

    /// U+0021 EXCLAMATION MARK
    pub const exclam: Self = Self { inner: 0x0021 };

    /// U+0022 QUOTATION MARK
    pub const quotedbl: Self = Self { inner: 0x0022 };

    /// U+0023 NUMBER SIGN
    pub const numbersign: Self = Self { inner: 0x0023 };

    /// U+0024 DOLLAR SIGN
    pub const dollar: Self = Self { inner: 0x0024 };

    /// U+0025 PERCENT SIGN
    pub const percent: Self = Self { inner: 0x0025 };

    /// U+0026 AMPERSAND
    pub const ampersand: Self = Self { inner: 0x0026 };

    /// U+0027 APOSTROPHE
    pub const apostrophe: Self = Self { inner: 0x0027 };

    /// deprecated
    pub const quoteright: Self = Self { inner: 0x0027 };

    /// U+0028 LEFT PARENTHESIS
    pub const parenleft: Self = Self { inner: 0x0028 };

    /// U+0029 RIGHT PARENTHESIS
    pub const parenright: Self = Self { inner: 0x0029 };

    /// U+002A ASTERISK
    pub const asterisk: Self = Self { inner: 0x002a };

    /// U+002B PLUS SIGN
    pub const plus: Self = Self { inner: 0x002b };

    /// U+002C COMMA
    pub const comma: Self = Self { inner: 0x002c };

    /// U+002D HYPHEN-MINUS
    pub const minus: Self = Self { inner: 0x002d };

    /// U+002E FULL STOP
    pub const period: Self = Self { inner: 0x002e };

    /// U+002F SOLIDUS
    pub const slash: Self = Self { inner: 0x002f };

    /// U+0030 DIGIT ZERO
    pub const DIGIT_0: Self = Self { inner: 0x0030 };

    /// U+0031 DIGIT ONE
    pub const DIGIT_1: Self = Self { inner: 0x0031 };

    /// U+0032 DIGIT TWO
    pub const DIGIT_2: Self = Self { inner: 0x0032 };

    /// U+0033 DIGIT THREE
    pub const DIGIT_3: Self = Self { inner: 0x0033 };

    /// U+0034 DIGIT FOUR
    pub const DIGIT_4: Self = Self { inner: 0x0034 };

    /// U+0035 DIGIT FIVE
    pub const DIGIT_5: Self = Self { inner: 0x0035 };

    /// U+0036 DIGIT SIX
    pub const DIGIT_6: Self = Self { inner: 0x0036 };

    /// U+0037 DIGIT SEVEN
    pub const DIGIT_7: Self = Self { inner: 0x0037 };

    /// U+0038 DIGIT EIGHT
    pub const DIGIT_8: Self = Self { inner: 0x0038 };

    /// U+0039 DIGIT NINE
    pub const DIGIT_9: Self = Self { inner: 0x0039 };

    /// U+003A COLON
    pub const colon: Self = Self { inner: 0x003a };

    /// U+003B SEMICOLON
    pub const semicolon: Self = Self { inner: 0x003b };

    /// U+003C LESS-THAN SIGN
    pub const less: Self = Self { inner: 0x003c };

    /// U+003D EQUALS SIGN
    pub const equal: Self = Self { inner: 0x003d };

    /// U+003E GREATER-THAN SIGN
    pub const greater: Self = Self { inner: 0x003e };

    /// U+003F QUESTION MARK
    pub const question: Self = Self { inner: 0x003f };

    /// U+0040 COMMERCIAL AT
    pub const at: Self = Self { inner: 0x0040 };

    /// U+0041 LATIN CAPITAL LETTER A
    pub const A: Self = Self { inner: 0x0041 };

    /// U+0042 LATIN CAPITAL LETTER B
    pub const B: Self = Self { inner: 0x0042 };

    /// U+0043 LATIN CAPITAL LETTER C
    pub const C: Self = Self { inner: 0x0043 };

    /// U+0044 LATIN CAPITAL LETTER D
    pub const D: Self = Self { inner: 0x0044 };

    /// U+0045 LATIN CAPITAL LETTER E
    pub const E: Self = Self { inner: 0x0045 };

    /// U+0046 LATIN CAPITAL LETTER F
    pub const F: Self = Self { inner: 0x0046 };

    /// U+0047 LATIN CAPITAL LETTER G
    pub const G: Self = Self { inner: 0x0047 };

    /// U+0048 LATIN CAPITAL LETTER H
    pub const H: Self = Self { inner: 0x0048 };

    /// U+0049 LATIN CAPITAL LETTER I
    pub const I: Self = Self { inner: 0x0049 };

    /// U+004A LATIN CAPITAL LETTER J
    pub const J: Self = Self { inner: 0x004a };

    /// U+004B LATIN CAPITAL LETTER K
    pub const K: Self = Self { inner: 0x004b };

    /// U+004C LATIN CAPITAL LETTER L
    pub const L: Self = Self { inner: 0x004c };

    /// U+004D LATIN CAPITAL LETTER M
    pub const M: Self = Self { inner: 0x004d };

    /// U+004E LATIN CAPITAL LETTER N
    pub const N: Self = Self { inner: 0x004e };

    /// U+004F LATIN CAPITAL LETTER O
    pub const O: Self = Self { inner: 0x004f };

    /// U+0050 LATIN CAPITAL LETTER P
    pub const P: Self = Self { inner: 0x0050 };

    /// U+0051 LATIN CAPITAL LETTER Q
    pub const Q: Self = Self { inner: 0x0051 };

    /// U+0052 LATIN CAPITAL LETTER R
    pub const R: Self = Self { inner: 0x0052 };

    /// U+0053 LATIN CAPITAL LETTER S
    pub const S: Self = Self { inner: 0x0053 };

    /// U+0054 LATIN CAPITAL LETTER T
    pub const T: Self = Self { inner: 0x0054 };

    /// U+0055 LATIN CAPITAL LETTER U
    pub const U: Self = Self { inner: 0x0055 };

    /// U+0056 LATIN CAPITAL LETTER V
    pub const V: Self = Self { inner: 0x0056 };

    /// U+0057 LATIN CAPITAL LETTER W
    pub const W: Self = Self { inner: 0x0057 };

    /// U+0058 LATIN CAPITAL LETTER X
    pub const X: Self = Self { inner: 0x0058 };

    /// U+0059 LATIN CAPITAL LETTER Y
    pub const Y: Self = Self { inner: 0x0059 };

    /// U+005A LATIN CAPITAL LETTER Z
    pub const Z: Self = Self { inner: 0x005a };

    /// U+005B LEFT SQUARE BRACKET
    pub const bracketleft: Self = Self { inner: 0x005b };

    /// U+005C REVERSE SOLIDUS
    pub const backslash: Self = Self { inner: 0x005c };

    /// U+005D RIGHT SQUARE BRACKET
    pub const bracketright: Self = Self { inner: 0x005d };

    /// U+005E CIRCUMFLEX ACCENT
    pub const asciicircum: Self = Self { inner: 0x005e };

    /// U+005F LOW LINE
    pub const underscore: Self = Self { inner: 0x005f };

    /// U+0060 GRAVE ACCENT
    pub const grave: Self = Self { inner: 0x0060 };

    /// deprecated
    pub const quoteleft: Self = Self { inner: 0x0060 };

    /// U+0061 LATIN SMALL LETTER A
    pub const a: Self = Self { inner: 0x0061 };

    /// U+0062 LATIN SMALL LETTER B
    pub const b: Self = Self { inner: 0x0062 };

    /// U+0063 LATIN SMALL LETTER C
    pub const c: Self = Self { inner: 0x0063 };

    /// U+0064 LATIN SMALL LETTER D
    pub const d: Self = Self { inner: 0x0064 };

    /// U+0065 LATIN SMALL LETTER E
    pub const e: Self = Self { inner: 0x0065 };

    /// U+0066 LATIN SMALL LETTER F
    pub const f: Self = Self { inner: 0x0066 };

    /// U+0067 LATIN SMALL LETTER G
    pub const g: Self = Self { inner: 0x0067 };

    /// U+0068 LATIN SMALL LETTER H
    pub const h: Self = Self { inner: 0x0068 };

    /// U+0069 LATIN SMALL LETTER I
    pub const i: Self = Self { inner: 0x0069 };

    /// U+006A LATIN SMALL LETTER J
    pub const j: Self = Self { inner: 0x006a };

    /// U+006B LATIN SMALL LETTER K
    pub const k: Self = Self { inner: 0x006b };

    /// U+006C LATIN SMALL LETTER L
    pub const l: Self = Self { inner: 0x006c };

    /// U+006D LATIN SMALL LETTER M
    pub const m: Self = Self { inner: 0x006d };

    /// U+006E LATIN SMALL LETTER N
    pub const n: Self = Self { inner: 0x006e };

    /// U+006F LATIN SMALL LETTER O
    pub const o: Self = Self { inner: 0x006f };

    /// U+0070 LATIN SMALL LETTER P
    pub const p: Self = Self { inner: 0x0070 };

    /// U+0071 LATIN SMALL LETTER Q
    pub const q: Self = Self { inner: 0x0071 };

    /// U+0072 LATIN SMALL LETTER R
    pub const r: Self = Self { inner: 0x0072 };

    /// U+0073 LATIN SMALL LETTER S
    pub const s: Self = Self { inner: 0x0073 };

    /// U+0074 LATIN SMALL LETTER T
    pub const t: Self = Self { inner: 0x0074 };

    /// U+0075 LATIN SMALL LETTER U
    pub const u: Self = Self { inner: 0x0075 };

    /// U+0076 LATIN SMALL LETTER V
    pub const v: Self = Self { inner: 0x0076 };

    /// U+0077 LATIN SMALL LETTER W
    pub const w: Self = Self { inner: 0x0077 };

    /// U+0078 LATIN SMALL LETTER X
    pub const x: Self = Self { inner: 0x0078 };

    /// U+0079 LATIN SMALL LETTER Y
    pub const y: Self = Self { inner: 0x0079 };

    /// U+007A LATIN SMALL LETTER Z
    pub const z: Self = Self { inner: 0x007a };

    /// U+007B LEFT CURLY BRACKET
    pub const braceleft: Self = Self { inner: 0x007b };

    /// U+007C VERTICAL LINE
    pub const bar: Self = Self { inner: 0x007c };

    /// U+007D RIGHT CURLY BRACKET
    pub const braceright: Self = Self { inner: 0x007d };

    /// U+007E TILDE
    pub const asciitilde: Self = Self { inner: 0x007e };

    /// U+00A0 NO-BREAK SPACE
    pub const nobreakspace: Self = Self { inner: 0x00a0 };

    /// U+00A1 INVERTED EXCLAMATION MARK
    pub const exclamdown: Self = Self { inner: 0x00a1 };

    /// U+00A2 CENT SIGN
    pub const cent: Self = Self { inner: 0x00a2 };

    /// U+00A3 POUND SIGN
    pub const sterling: Self = Self { inner: 0x00a3 };

    /// U+00A4 CURRENCY SIGN
    pub const currency: Self = Self { inner: 0x00a4 };

    /// U+00A5 YEN SIGN
    pub const yen: Self = Self { inner: 0x00a5 };

    /// U+00A6 BROKEN BAR
    pub const brokenbar: Self = Self { inner: 0x00a6 };

    /// U+00A7 SECTION SIGN
    pub const section: Self = Self { inner: 0x00a7 };

    /// U+00A8 DIAERESIS
    pub const diaeresis: Self = Self { inner: 0x00a8 };

    /// U+00A9 COPYRIGHT SIGN
    pub const copyright: Self = Self { inner: 0x00a9 };

    /// U+00AA FEMININE ORDINAL INDICATOR
    pub const ordfeminine: Self = Self { inner: 0x00aa };

    /// U+00AB LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    pub const guillemotleft: Self = Self { inner: 0x00ab };

    /// U+00AC NOT SIGN
    pub const notsign: Self = Self { inner: 0x00ac };

    /// U+00AD SOFT HYPHEN
    pub const hyphen: Self = Self { inner: 0x00ad };

    /// U+00AE REGISTERED SIGN
    pub const registered: Self = Self { inner: 0x00ae };

    /// U+00AF MACRON
    pub const macron: Self = Self { inner: 0x00af };

    /// U+00B0 DEGREE SIGN
    pub const degree: Self = Self { inner: 0x00b0 };

    /// U+00B1 PLUS-MINUS SIGN
    pub const plusminus: Self = Self { inner: 0x00b1 };

    /// U+00B2 SUPERSCRIPT TWO
    pub const twosuperior: Self = Self { inner: 0x00b2 };

    /// U+00B3 SUPERSCRIPT THREE
    pub const threesuperior: Self = Self { inner: 0x00b3 };

    /// U+00B4 ACUTE ACCENT
    pub const acute: Self = Self { inner: 0x00b4 };

    /// U+00B5 MICRO SIGN
    pub const mu: Self = Self { inner: 0x00b5 };

    /// U+00B6 PILCROW SIGN
    pub const paragraph: Self = Self { inner: 0x00b6 };

    /// U+00B7 MIDDLE DOT
    pub const periodcentered: Self = Self { inner: 0x00b7 };

    /// U+00B8 CEDILLA
    pub const cedilla: Self = Self { inner: 0x00b8 };

    /// U+00B9 SUPERSCRIPT ONE
    pub const onesuperior: Self = Self { inner: 0x00b9 };

    /// U+00BA MASCULINE ORDINAL INDICATOR
    pub const masculine: Self = Self { inner: 0x00ba };

    /// U+00BB RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    pub const guillemotright: Self = Self { inner: 0x00bb };

    /// U+00BC VULGAR FRACTION ONE QUARTER
    pub const onequarter: Self = Self { inner: 0x00bc };

    /// U+00BD VULGAR FRACTION ONE HALF
    pub const onehalf: Self = Self { inner: 0x00bd };

    /// U+00BE VULGAR FRACTION THREE QUARTERS
    pub const threequarters: Self = Self { inner: 0x00be };

    /// U+00BF INVERTED QUESTION MARK
    pub const questiondown: Self = Self { inner: 0x00bf };

    /// U+00C0 LATIN CAPITAL LETTER A WITH GRAVE
    pub const Agrave: Self = Self { inner: 0x00c0 };

    /// U+00C1 LATIN CAPITAL LETTER A WITH ACUTE
    pub const Aacute: Self = Self { inner: 0x00c1 };

    /// U+00C2 LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    pub const Acircumflex: Self = Self { inner: 0x00c2 };

    /// U+00C3 LATIN CAPITAL LETTER A WITH TILDE
    pub const Atilde: Self = Self { inner: 0x00c3 };

    /// U+00C4 LATIN CAPITAL LETTER A WITH DIAERESIS
    pub const Adiaeresis: Self = Self { inner: 0x00c4 };

    /// U+00C5 LATIN CAPITAL LETTER A WITH RING ABOVE
    pub const Aring: Self = Self { inner: 0x00c5 };

    /// U+00C6 LATIN CAPITAL LETTER AE
    pub const AE: Self = Self { inner: 0x00c6 };

    /// U+00C7 LATIN CAPITAL LETTER C WITH CEDILLA
    pub const Ccedilla: Self = Self { inner: 0x00c7 };

    /// U+00C8 LATIN CAPITAL LETTER E WITH GRAVE
    pub const Egrave: Self = Self { inner: 0x00c8 };

    /// U+00C9 LATIN CAPITAL LETTER E WITH ACUTE
    pub const Eacute: Self = Self { inner: 0x00c9 };

    /// U+00CA LATIN CAPITAL LETTER E WITH CIRCUMFLEX
    pub const Ecircumflex: Self = Self { inner: 0x00ca };

    /// U+00CB LATIN CAPITAL LETTER E WITH DIAERESIS
    pub const Ediaeresis: Self = Self { inner: 0x00cb };

    /// U+00CC LATIN CAPITAL LETTER I WITH GRAVE
    pub const Igrave: Self = Self { inner: 0x00cc };

    /// U+00CD LATIN CAPITAL LETTER I WITH ACUTE
    pub const Iacute: Self = Self { inner: 0x00cd };

    /// U+00CE LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    pub const Icircumflex: Self = Self { inner: 0x00ce };

    /// U+00CF LATIN CAPITAL LETTER I WITH DIAERESIS
    pub const Idiaeresis: Self = Self { inner: 0x00cf };

    /// U+00D0 LATIN CAPITAL LETTER ETH
    pub const ETH: Self = Self { inner: 0x00d0 };

    /// deprecated
    pub const Eth: Self = Self { inner: 0x00d0 };

    /// U+00D1 LATIN CAPITAL LETTER N WITH TILDE
    pub const Ntilde: Self = Self { inner: 0x00d1 };

    /// U+00D2 LATIN CAPITAL LETTER O WITH GRAVE
    pub const Ograve: Self = Self { inner: 0x00d2 };

    /// U+00D3 LATIN CAPITAL LETTER O WITH ACUTE
    pub const Oacute: Self = Self { inner: 0x00d3 };

    /// U+00D4 LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    pub const Ocircumflex: Self = Self { inner: 0x00d4 };

    /// U+00D5 LATIN CAPITAL LETTER O WITH TILDE
    pub const Otilde: Self = Self { inner: 0x00d5 };

    /// U+00D6 LATIN CAPITAL LETTER O WITH DIAERESIS
    pub const Odiaeresis: Self = Self { inner: 0x00d6 };

    /// U+00D7 MULTIPLICATION SIGN
    pub const multiply: Self = Self { inner: 0x00d7 };

    /// U+00D8 LATIN CAPITAL LETTER O WITH STROKE
    pub const Oslash: Self = Self { inner: 0x00d8 };

    /// U+00D8 LATIN CAPITAL LETTER O WITH STROKE
    pub const Ooblique: Self = Self { inner: 0x00d8 };

    /// U+00D9 LATIN CAPITAL LETTER U WITH GRAVE
    pub const Ugrave: Self = Self { inner: 0x00d9 };

    /// U+00DA LATIN CAPITAL LETTER U WITH ACUTE
    pub const Uacute: Self = Self { inner: 0x00da };

    /// U+00DB LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    pub const Ucircumflex: Self = Self { inner: 0x00db };

    /// U+00DC LATIN CAPITAL LETTER U WITH DIAERESIS
    pub const Udiaeresis: Self = Self { inner: 0x00dc };

    /// U+00DD LATIN CAPITAL LETTER Y WITH ACUTE
    pub const Yacute: Self = Self { inner: 0x00dd };

    /// U+00DE LATIN CAPITAL LETTER THORN
    pub const THORN: Self = Self { inner: 0x00de };

    /// deprecated
    pub const Thorn: Self = Self { inner: 0x00de };

    /// U+00DF LATIN SMALL LETTER SHARP S
    pub const ssharp: Self = Self { inner: 0x00df };

    /// U+00E0 LATIN SMALL LETTER A WITH GRAVE
    pub const agrave: Self = Self { inner: 0x00e0 };

    /// U+00E1 LATIN SMALL LETTER A WITH ACUTE
    pub const aacute: Self = Self { inner: 0x00e1 };

    /// U+00E2 LATIN SMALL LETTER A WITH CIRCUMFLEX
    pub const acircumflex: Self = Self { inner: 0x00e2 };

    /// U+00E3 LATIN SMALL LETTER A WITH TILDE
    pub const atilde: Self = Self { inner: 0x00e3 };

    /// U+00E4 LATIN SMALL LETTER A WITH DIAERESIS
    pub const adiaeresis: Self = Self { inner: 0x00e4 };

    /// U+00E5 LATIN SMALL LETTER A WITH RING ABOVE
    pub const aring: Self = Self { inner: 0x00e5 };

    /// U+00E6 LATIN SMALL LETTER AE
    pub const ae: Self = Self { inner: 0x00e6 };

    /// U+00E7 LATIN SMALL LETTER C WITH CEDILLA
    pub const ccedilla: Self = Self { inner: 0x00e7 };

    /// U+00E8 LATIN SMALL LETTER E WITH GRAVE
    pub const egrave: Self = Self { inner: 0x00e8 };

    /// U+00E9 LATIN SMALL LETTER E WITH ACUTE
    pub const eacute: Self = Self { inner: 0x00e9 };

    /// U+00EA LATIN SMALL LETTER E WITH CIRCUMFLEX
    pub const ecircumflex: Self = Self { inner: 0x00ea };

    /// U+00EB LATIN SMALL LETTER E WITH DIAERESIS
    pub const ediaeresis: Self = Self { inner: 0x00eb };

    /// U+00EC LATIN SMALL LETTER I WITH GRAVE
    pub const igrave: Self = Self { inner: 0x00ec };

    /// U+00ED LATIN SMALL LETTER I WITH ACUTE
    pub const iacute: Self = Self { inner: 0x00ed };

    /// U+00EE LATIN SMALL LETTER I WITH CIRCUMFLEX
    pub const icircumflex: Self = Self { inner: 0x00ee };

    /// U+00EF LATIN SMALL LETTER I WITH DIAERESIS
    pub const idiaeresis: Self = Self { inner: 0x00ef };

    /// U+00F0 LATIN SMALL LETTER ETH
    pub const eth: Self = Self { inner: 0x00f0 };

    /// U+00F1 LATIN SMALL LETTER N WITH TILDE
    pub const ntilde: Self = Self { inner: 0x00f1 };

    /// U+00F2 LATIN SMALL LETTER O WITH GRAVE
    pub const ograve: Self = Self { inner: 0x00f2 };

    /// U+00F3 LATIN SMALL LETTER O WITH ACUTE
    pub const oacute: Self = Self { inner: 0x00f3 };

    /// U+00F4 LATIN SMALL LETTER O WITH CIRCUMFLEX
    pub const ocircumflex: Self = Self { inner: 0x00f4 };

    /// U+00F5 LATIN SMALL LETTER O WITH TILDE
    pub const otilde: Self = Self { inner: 0x00f5 };

    /// U+00F6 LATIN SMALL LETTER O WITH DIAERESIS
    pub const odiaeresis: Self = Self { inner: 0x00f6 };

    /// U+00F7 DIVISION SIGN
    pub const division: Self = Self { inner: 0x00f7 };

    /// U+00F8 LATIN SMALL LETTER O WITH STROKE
    pub const oslash: Self = Self { inner: 0x00f8 };

    /// U+00F8 LATIN SMALL LETTER O WITH STROKE
    pub const ooblique: Self = Self { inner: 0x00f8 };

    /// U+00F9 LATIN SMALL LETTER U WITH GRAVE
    pub const ugrave: Self = Self { inner: 0x00f9 };

    /// U+00FA LATIN SMALL LETTER U WITH ACUTE
    pub const uacute: Self = Self { inner: 0x00fa };

    /// U+00FB LATIN SMALL LETTER U WITH CIRCUMFLEX
    pub const ucircumflex: Self = Self { inner: 0x00fb };

    /// U+00FC LATIN SMALL LETTER U WITH DIAERESIS
    pub const udiaeresis: Self = Self { inner: 0x00fc };

    /// U+00FD LATIN SMALL LETTER Y WITH ACUTE
    pub const yacute: Self = Self { inner: 0x00fd };

    /// U+00FE LATIN SMALL LETTER THORN
    pub const thorn: Self = Self { inner: 0x00fe };

    /// U+00FF LATIN SMALL LETTER Y WITH DIAERESIS
    pub const ydiaeresis: Self = Self { inner: 0x00ff };

    /// U+0104 LATIN CAPITAL LETTER A WITH OGONEK
    pub const Aogonek: Self = Self { inner: 0x01a1 };

    /// U+02D8 BREVE
    pub const breve: Self = Self { inner: 0x01a2 };

    /// U+0141 LATIN CAPITAL LETTER L WITH STROKE
    pub const Lstroke: Self = Self { inner: 0x01a3 };

    /// U+013D LATIN CAPITAL LETTER L WITH CARON
    pub const Lcaron: Self = Self { inner: 0x01a5 };

    /// U+015A LATIN CAPITAL LETTER S WITH ACUTE
    pub const Sacute: Self = Self { inner: 0x01a6 };

    /// U+0160 LATIN CAPITAL LETTER S WITH CARON
    pub const Scaron: Self = Self { inner: 0x01a9 };

    /// U+015E LATIN CAPITAL LETTER S WITH CEDILLA
    pub const Scedilla: Self = Self { inner: 0x01aa };

    /// U+0164 LATIN CAPITAL LETTER T WITH CARON
    pub const Tcaron: Self = Self { inner: 0x01ab };

    /// U+0179 LATIN CAPITAL LETTER Z WITH ACUTE
    pub const Zacute: Self = Self { inner: 0x01ac };

    /// U+017D LATIN CAPITAL LETTER Z WITH CARON
    pub const Zcaron: Self = Self { inner: 0x01ae };

    /// U+017B LATIN CAPITAL LETTER Z WITH DOT ABOVE
    pub const Zabovedot: Self = Self { inner: 0x01af };

    /// U+0105 LATIN SMALL LETTER A WITH OGONEK
    pub const aogonek: Self = Self { inner: 0x01b1 };

    /// U+02DB OGONEK
    pub const ogonek: Self = Self { inner: 0x01b2 };

    /// U+0142 LATIN SMALL LETTER L WITH STROKE
    pub const lstroke: Self = Self { inner: 0x01b3 };

    /// U+013E LATIN SMALL LETTER L WITH CARON
    pub const lcaron: Self = Self { inner: 0x01b5 };

    /// U+015B LATIN SMALL LETTER S WITH ACUTE
    pub const sacute: Self = Self { inner: 0x01b6 };

    /// U+02C7 CARON
    pub const caron: Self = Self { inner: 0x01b7 };

    /// U+0161 LATIN SMALL LETTER S WITH CARON
    pub const scaron: Self = Self { inner: 0x01b9 };

    /// U+015F LATIN SMALL LETTER S WITH CEDILLA
    pub const scedilla: Self = Self { inner: 0x01ba };

    /// U+0165 LATIN SMALL LETTER T WITH CARON
    pub const tcaron: Self = Self { inner: 0x01bb };

    /// U+017A LATIN SMALL LETTER Z WITH ACUTE
    pub const zacute: Self = Self { inner: 0x01bc };

    /// U+02DD DOUBLE ACUTE ACCENT
    pub const doubleacute: Self = Self { inner: 0x01bd };

    /// U+017E LATIN SMALL LETTER Z WITH CARON
    pub const zcaron: Self = Self { inner: 0x01be };

    /// U+017C LATIN SMALL LETTER Z WITH DOT ABOVE
    pub const zabovedot: Self = Self { inner: 0x01bf };

    /// U+0154 LATIN CAPITAL LETTER R WITH ACUTE
    pub const Racute: Self = Self { inner: 0x01c0 };

    /// U+0102 LATIN CAPITAL LETTER A WITH BREVE
    pub const Abreve: Self = Self { inner: 0x01c3 };

    /// U+0139 LATIN CAPITAL LETTER L WITH ACUTE
    pub const Lacute: Self = Self { inner: 0x01c5 };

    /// U+0106 LATIN CAPITAL LETTER C WITH ACUTE
    pub const Cacute: Self = Self { inner: 0x01c6 };

    /// U+010C LATIN CAPITAL LETTER C WITH CARON
    pub const Ccaron: Self = Self { inner: 0x01c8 };

    /// U+0118 LATIN CAPITAL LETTER E WITH OGONEK
    pub const Eogonek: Self = Self { inner: 0x01ca };

    /// U+011A LATIN CAPITAL LETTER E WITH CARON
    pub const Ecaron: Self = Self { inner: 0x01cc };

    /// U+010E LATIN CAPITAL LETTER D WITH CARON
    pub const Dcaron: Self = Self { inner: 0x01cf };

    /// U+0110 LATIN CAPITAL LETTER D WITH STROKE
    pub const Dstroke: Self = Self { inner: 0x01d0 };

    /// U+0143 LATIN CAPITAL LETTER N WITH ACUTE
    pub const Nacute: Self = Self { inner: 0x01d1 };

    /// U+0147 LATIN CAPITAL LETTER N WITH CARON
    pub const Ncaron: Self = Self { inner: 0x01d2 };

    /// U+0150 LATIN CAPITAL LETTER O WITH DOUBLE ACUTE
    pub const Odoubleacute: Self = Self { inner: 0x01d5 };

    /// U+0158 LATIN CAPITAL LETTER R WITH CARON
    pub const Rcaron: Self = Self { inner: 0x01d8 };

    /// U+016E LATIN CAPITAL LETTER U WITH RING ABOVE
    pub const Uring: Self = Self { inner: 0x01d9 };

    /// U+0170 LATIN CAPITAL LETTER U WITH DOUBLE ACUTE
    pub const Udoubleacute: Self = Self { inner: 0x01db };

    /// U+0162 LATIN CAPITAL LETTER T WITH CEDILLA
    pub const Tcedilla: Self = Self { inner: 0x01de };

    /// U+0155 LATIN SMALL LETTER R WITH ACUTE
    pub const racute: Self = Self { inner: 0x01e0 };

    /// U+0103 LATIN SMALL LETTER A WITH BREVE
    pub const abreve: Self = Self { inner: 0x01e3 };

    /// U+013A LATIN SMALL LETTER L WITH ACUTE
    pub const lacute: Self = Self { inner: 0x01e5 };

    /// U+0107 LATIN SMALL LETTER C WITH ACUTE
    pub const cacute: Self = Self { inner: 0x01e6 };

    /// U+010D LATIN SMALL LETTER C WITH CARON
    pub const ccaron: Self = Self { inner: 0x01e8 };

    /// U+0119 LATIN SMALL LETTER E WITH OGONEK
    pub const eogonek: Self = Self { inner: 0x01ea };

    /// U+011B LATIN SMALL LETTER E WITH CARON
    pub const ecaron: Self = Self { inner: 0x01ec };

    /// U+010F LATIN SMALL LETTER D WITH CARON
    pub const dcaron: Self = Self { inner: 0x01ef };

    /// U+0111 LATIN SMALL LETTER D WITH STROKE
    pub const dstroke: Self = Self { inner: 0x01f0 };

    /// U+0144 LATIN SMALL LETTER N WITH ACUTE
    pub const nacute: Self = Self { inner: 0x01f1 };

    /// U+0148 LATIN SMALL LETTER N WITH CARON
    pub const ncaron: Self = Self { inner: 0x01f2 };

    /// U+0151 LATIN SMALL LETTER O WITH DOUBLE ACUTE
    pub const odoubleacute: Self = Self { inner: 0x01f5 };

    /// U+0171 LATIN SMALL LETTER U WITH DOUBLE ACUTE
    pub const udoubleacute: Self = Self { inner: 0x01fb };

    /// U+0159 LATIN SMALL LETTER R WITH CARON
    pub const rcaron: Self = Self { inner: 0x01f8 };

    /// U+016F LATIN SMALL LETTER U WITH RING ABOVE
    pub const uring: Self = Self { inner: 0x01f9 };

    /// U+0163 LATIN SMALL LETTER T WITH CEDILLA
    pub const tcedilla: Self = Self { inner: 0x01fe };

    /// U+02D9 DOT ABOVE
    pub const abovedot: Self = Self { inner: 0x01ff };

    /// U+0126 LATIN CAPITAL LETTER H WITH STROKE
    pub const Hstroke: Self = Self { inner: 0x02a1 };

    /// U+0124 LATIN CAPITAL LETTER H WITH CIRCUMFLEX
    pub const Hcircumflex: Self = Self { inner: 0x02a6 };

    /// U+0130 LATIN CAPITAL LETTER I WITH DOT ABOVE
    pub const Iabovedot: Self = Self { inner: 0x02a9 };

    /// U+011E LATIN CAPITAL LETTER G WITH BREVE
    pub const Gbreve: Self = Self { inner: 0x02ab };

    /// U+0134 LATIN CAPITAL LETTER J WITH CIRCUMFLEX
    pub const Jcircumflex: Self = Self { inner: 0x02ac };

    /// U+0127 LATIN SMALL LETTER H WITH STROKE
    pub const hstroke: Self = Self { inner: 0x02b1 };

    /// U+0125 LATIN SMALL LETTER H WITH CIRCUMFLEX
    pub const hcircumflex: Self = Self { inner: 0x02b6 };

    /// U+0131 LATIN SMALL LETTER DOTLESS I
    pub const idotless: Self = Self { inner: 0x02b9 };

    /// U+011F LATIN SMALL LETTER G WITH BREVE
    pub const gbreve: Self = Self { inner: 0x02bb };

    /// U+0135 LATIN SMALL LETTER J WITH CIRCUMFLEX
    pub const jcircumflex: Self = Self { inner: 0x02bc };

    /// U+010A LATIN CAPITAL LETTER C WITH DOT ABOVE
    pub const Cabovedot: Self = Self { inner: 0x02c5 };

    /// U+0108 LATIN CAPITAL LETTER C WITH CIRCUMFLEX
    pub const Ccircumflex: Self = Self { inner: 0x02c6 };

    /// U+0120 LATIN CAPITAL LETTER G WITH DOT ABOVE
    pub const Gabovedot: Self = Self { inner: 0x02d5 };

    /// U+011C LATIN CAPITAL LETTER G WITH CIRCUMFLEX
    pub const Gcircumflex: Self = Self { inner: 0x02d8 };

    /// U+016C LATIN CAPITAL LETTER U WITH BREVE
    pub const Ubreve: Self = Self { inner: 0x02dd };

    /// U+015C LATIN CAPITAL LETTER S WITH CIRCUMFLEX
    pub const Scircumflex: Self = Self { inner: 0x02de };

    /// U+010B LATIN SMALL LETTER C WITH DOT ABOVE
    pub const cabovedot: Self = Self { inner: 0x02e5 };

    /// U+0109 LATIN SMALL LETTER C WITH CIRCUMFLEX
    pub const ccircumflex: Self = Self { inner: 0x02e6 };

    /// U+0121 LATIN SMALL LETTER G WITH DOT ABOVE
    pub const gabovedot: Self = Self { inner: 0x02f5 };

    /// U+011D LATIN SMALL LETTER G WITH CIRCUMFLEX
    pub const gcircumflex: Self = Self { inner: 0x02f8 };

    /// U+016D LATIN SMALL LETTER U WITH BREVE
    pub const ubreve: Self = Self { inner: 0x02fd };

    /// U+015D LATIN SMALL LETTER S WITH CIRCUMFLEX
    pub const scircumflex: Self = Self { inner: 0x02fe };

    /// U+0138 LATIN SMALL LETTER KRA
    pub const kra: Self = Self { inner: 0x03a2 };

    /// deprecated
    pub const kappa: Self = Self { inner: 0x03a2 };

    /// U+0156 LATIN CAPITAL LETTER R WITH CEDILLA
    pub const Rcedilla: Self = Self { inner: 0x03a3 };

    /// U+0128 LATIN CAPITAL LETTER I WITH TILDE
    pub const Itilde: Self = Self { inner: 0x03a5 };

    /// U+013B LATIN CAPITAL LETTER L WITH CEDILLA
    pub const Lcedilla: Self = Self { inner: 0x03a6 };

    /// U+0112 LATIN CAPITAL LETTER E WITH MACRON
    pub const Emacron: Self = Self { inner: 0x03aa };

    /// U+0122 LATIN CAPITAL LETTER G WITH CEDILLA
    pub const Gcedilla: Self = Self { inner: 0x03ab };

    /// U+0166 LATIN CAPITAL LETTER T WITH STROKE
    pub const Tslash: Self = Self { inner: 0x03ac };

    /// U+0157 LATIN SMALL LETTER R WITH CEDILLA
    pub const rcedilla: Self = Self { inner: 0x03b3 };

    /// U+0129 LATIN SMALL LETTER I WITH TILDE
    pub const itilde: Self = Self { inner: 0x03b5 };

    /// U+013C LATIN SMALL LETTER L WITH CEDILLA
    pub const lcedilla: Self = Self { inner: 0x03b6 };

    /// U+0113 LATIN SMALL LETTER E WITH MACRON
    pub const emacron: Self = Self { inner: 0x03ba };

    /// U+0123 LATIN SMALL LETTER G WITH CEDILLA
    pub const gcedilla: Self = Self { inner: 0x03bb };

    /// U+0167 LATIN SMALL LETTER T WITH STROKE
    pub const tslash: Self = Self { inner: 0x03bc };

    /// U+014A LATIN CAPITAL LETTER ENG
    pub const ENG: Self = Self { inner: 0x03bd };

    /// U+014B LATIN SMALL LETTER ENG
    pub const eng: Self = Self { inner: 0x03bf };

    /// U+0100 LATIN CAPITAL LETTER A WITH MACRON
    pub const Amacron: Self = Self { inner: 0x03c0 };

    /// U+012E LATIN CAPITAL LETTER I WITH OGONEK
    pub const Iogonek: Self = Self { inner: 0x03c7 };

    /// U+0116 LATIN CAPITAL LETTER E WITH DOT ABOVE
    pub const Eabovedot: Self = Self { inner: 0x03cc };

    /// U+012A LATIN CAPITAL LETTER I WITH MACRON
    pub const Imacron: Self = Self { inner: 0x03cf };

    /// U+0145 LATIN CAPITAL LETTER N WITH CEDILLA
    pub const Ncedilla: Self = Self { inner: 0x03d1 };

    /// U+014C LATIN CAPITAL LETTER O WITH MACRON
    pub const Omacron: Self = Self { inner: 0x03d2 };

    /// U+0136 LATIN CAPITAL LETTER K WITH CEDILLA
    pub const Kcedilla: Self = Self { inner: 0x03d3 };

    /// U+0172 LATIN CAPITAL LETTER U WITH OGONEK
    pub const Uogonek: Self = Self { inner: 0x03d9 };

    /// U+0168 LATIN CAPITAL LETTER U WITH TILDE
    pub const Utilde: Self = Self { inner: 0x03dd };

    /// U+016A LATIN CAPITAL LETTER U WITH MACRON
    pub const Umacron: Self = Self { inner: 0x03de };

    /// U+0101 LATIN SMALL LETTER A WITH MACRON
    pub const amacron: Self = Self { inner: 0x03e0 };

    /// U+012F LATIN SMALL LETTER I WITH OGONEK
    pub const iogonek: Self = Self { inner: 0x03e7 };

    /// U+0117 LATIN SMALL LETTER E WITH DOT ABOVE
    pub const eabovedot: Self = Self { inner: 0x03ec };

    /// U+012B LATIN SMALL LETTER I WITH MACRON
    pub const imacron: Self = Self { inner: 0x03ef };

    /// U+0146 LATIN SMALL LETTER N WITH CEDILLA
    pub const ncedilla: Self = Self { inner: 0x03f1 };

    /// U+014D LATIN SMALL LETTER O WITH MACRON
    pub const omacron: Self = Self { inner: 0x03f2 };

    /// U+0137 LATIN SMALL LETTER K WITH CEDILLA
    pub const kcedilla: Self = Self { inner: 0x03f3 };

    /// U+0173 LATIN SMALL LETTER U WITH OGONEK
    pub const uogonek: Self = Self { inner: 0x03f9 };

    /// U+0169 LATIN SMALL LETTER U WITH TILDE
    pub const utilde: Self = Self { inner: 0x03fd };

    /// U+016B LATIN SMALL LETTER U WITH MACRON
    pub const umacron: Self = Self { inner: 0x03fe };

    /// U+1E02 LATIN CAPITAL LETTER B WITH DOT ABOVE
    pub const Babovedot: Self = Self { inner: 0x1001e02 };

    /// U+1E03 LATIN SMALL LETTER B WITH DOT ABOVE
    pub const babovedot: Self = Self { inner: 0x1001e03 };

    /// U+1E0A LATIN CAPITAL LETTER D WITH DOT ABOVE
    pub const Dabovedot: Self = Self { inner: 0x1001e0a };

    /// U+1E80 LATIN CAPITAL LETTER W WITH GRAVE
    pub const Wgrave: Self = Self { inner: 0x1001e80 };

    /// U+1E82 LATIN CAPITAL LETTER W WITH ACUTE
    pub const Wacute: Self = Self { inner: 0x1001e82 };

    /// U+1E0B LATIN SMALL LETTER D WITH DOT ABOVE
    pub const dabovedot: Self = Self { inner: 0x1001e0b };

    /// U+1EF2 LATIN CAPITAL LETTER Y WITH GRAVE
    pub const Ygrave: Self = Self { inner: 0x1001ef2 };

    /// U+1E1E LATIN CAPITAL LETTER F WITH DOT ABOVE
    pub const Fabovedot: Self = Self { inner: 0x1001e1e };

    /// U+1E1F LATIN SMALL LETTER F WITH DOT ABOVE
    pub const fabovedot: Self = Self { inner: 0x1001e1f };

    /// U+1E40 LATIN CAPITAL LETTER M WITH DOT ABOVE
    pub const Mabovedot: Self = Self { inner: 0x1001e40 };

    /// U+1E41 LATIN SMALL LETTER M WITH DOT ABOVE
    pub const mabovedot: Self = Self { inner: 0x1001e41 };

    /// U+1E56 LATIN CAPITAL LETTER P WITH DOT ABOVE
    pub const Pabovedot: Self = Self { inner: 0x1001e56 };

    /// U+1E81 LATIN SMALL LETTER W WITH GRAVE
    pub const wgrave: Self = Self { inner: 0x1001e81 };

    /// U+1E57 LATIN SMALL LETTER P WITH DOT ABOVE
    pub const pabovedot: Self = Self { inner: 0x1001e57 };

    /// U+1E83 LATIN SMALL LETTER W WITH ACUTE
    pub const wacute: Self = Self { inner: 0x1001e83 };

    /// U+1E60 LATIN CAPITAL LETTER S WITH DOT ABOVE
    pub const Sabovedot: Self = Self { inner: 0x1001e60 };

    /// U+1EF3 LATIN SMALL LETTER Y WITH GRAVE
    pub const ygrave: Self = Self { inner: 0x1001ef3 };

    /// U+1E84 LATIN CAPITAL LETTER W WITH DIAERESIS
    pub const Wdiaeresis: Self = Self { inner: 0x1001e84 };

    /// U+1E85 LATIN SMALL LETTER W WITH DIAERESIS
    pub const wdiaeresis: Self = Self { inner: 0x1001e85 };

    /// U+1E61 LATIN SMALL LETTER S WITH DOT ABOVE
    pub const sabovedot: Self = Self { inner: 0x1001e61 };

    /// U+0174 LATIN CAPITAL LETTER W WITH CIRCUMFLEX
    pub const Wcircumflex: Self = Self { inner: 0x1000174 };

    /// U+1E6A LATIN CAPITAL LETTER T WITH DOT ABOVE
    pub const Tabovedot: Self = Self { inner: 0x1001e6a };

    /// U+0176 LATIN CAPITAL LETTER Y WITH CIRCUMFLEX
    pub const Ycircumflex: Self = Self { inner: 0x1000176 };

    /// U+0175 LATIN SMALL LETTER W WITH CIRCUMFLEX
    pub const wcircumflex: Self = Self { inner: 0x1000175 };

    /// U+1E6B LATIN SMALL LETTER T WITH DOT ABOVE
    pub const tabovedot: Self = Self { inner: 0x1001e6b };

    /// U+0177 LATIN SMALL LETTER Y WITH CIRCUMFLEX
    pub const ycircumflex: Self = Self { inner: 0x1000177 };

    /// U+0152 LATIN CAPITAL LIGATURE OE
    pub const OE: Self = Self { inner: 0x13bc };

    /// U+0153 LATIN SMALL LIGATURE OE
    pub const oe: Self = Self { inner: 0x13bd };

    /// U+0178 LATIN CAPITAL LETTER Y WITH DIAERESIS
    pub const Ydiaeresis: Self = Self { inner: 0x13be };

    /// U+203E OVERLINE
    pub const overline: Self = Self { inner: 0x047e };

    /// U+3002 IDEOGRAPHIC FULL STOP
    pub const kana_fullstop: Self = Self { inner: 0x04a1 };

    /// U+300C LEFT CORNER BRACKET
    pub const kana_openingbracket: Self = Self { inner: 0x04a2 };

    /// U+300D RIGHT CORNER BRACKET
    pub const kana_closingbracket: Self = Self { inner: 0x04a3 };

    /// U+3001 IDEOGRAPHIC COMMA
    pub const kana_comma: Self = Self { inner: 0x04a4 };

    /// U+30FB KATAKANA MIDDLE DOT
    pub const kana_conjunctive: Self = Self { inner: 0x04a5 };

    /// deprecated
    pub const kana_middledot: Self = Self { inner: 0x04a5 };

    /// U+30F2 KATAKANA LETTER WO
    pub const kana_WO: Self = Self { inner: 0x04a6 };

    /// U+30A1 KATAKANA LETTER SMALL A
    pub const kana_a: Self = Self { inner: 0x04a7 };

    /// U+30A3 KATAKANA LETTER SMALL I
    pub const kana_i: Self = Self { inner: 0x04a8 };

    /// U+30A5 KATAKANA LETTER SMALL U
    pub const kana_u: Self = Self { inner: 0x04a9 };

    /// U+30A7 KATAKANA LETTER SMALL E
    pub const kana_e: Self = Self { inner: 0x04aa };

    /// U+30A9 KATAKANA LETTER SMALL O
    pub const kana_o: Self = Self { inner: 0x04ab };

    /// U+30E3 KATAKANA LETTER SMALL YA
    pub const kana_ya: Self = Self { inner: 0x04ac };

    /// U+30E5 KATAKANA LETTER SMALL YU
    pub const kana_yu: Self = Self { inner: 0x04ad };

    /// U+30E7 KATAKANA LETTER SMALL YO
    pub const kana_yo: Self = Self { inner: 0x04ae };

    /// U+30C3 KATAKANA LETTER SMALL TU
    pub const kana_tsu: Self = Self { inner: 0x04af };

    /// deprecated
    pub const kana_tu: Self = Self { inner: 0x04af };

    /// U+30FC KATAKANA-HIRAGANA PROLONGED SOUND MARK
    pub const prolongedsound: Self = Self { inner: 0x04b0 };

    /// U+30A2 KATAKANA LETTER A
    pub const kana_A: Self = Self { inner: 0x04b1 };

    /// U+30A4 KATAKANA LETTER I
    pub const kana_I: Self = Self { inner: 0x04b2 };

    /// U+30A6 KATAKANA LETTER U
    pub const kana_U: Self = Self { inner: 0x04b3 };

    /// U+30A8 KATAKANA LETTER E
    pub const kana_E: Self = Self { inner: 0x04b4 };

    /// U+30AA KATAKANA LETTER O
    pub const kana_O: Self = Self { inner: 0x04b5 };

    /// U+30AB KATAKANA LETTER KA
    pub const kana_KA: Self = Self { inner: 0x04b6 };

    /// U+30AD KATAKANA LETTER KI
    pub const kana_KI: Self = Self { inner: 0x04b7 };

    /// U+30AF KATAKANA LETTER KU
    pub const kana_KU: Self = Self { inner: 0x04b8 };

    /// U+30B1 KATAKANA LETTER KE
    pub const kana_KE: Self = Self { inner: 0x04b9 };

    /// U+30B3 KATAKANA LETTER KO
    pub const kana_KO: Self = Self { inner: 0x04ba };

    /// U+30B5 KATAKANA LETTER SA
    pub const kana_SA: Self = Self { inner: 0x04bb };

    /// U+30B7 KATAKANA LETTER SI
    pub const kana_SHI: Self = Self { inner: 0x04bc };

    /// U+30B9 KATAKANA LETTER SU
    pub const kana_SU: Self = Self { inner: 0x04bd };

    /// U+30BB KATAKANA LETTER SE
    pub const kana_SE: Self = Self { inner: 0x04be };

    /// U+30BD KATAKANA LETTER SO
    pub const kana_SO: Self = Self { inner: 0x04bf };

    /// U+30BF KATAKANA LETTER TA
    pub const kana_TA: Self = Self { inner: 0x04c0 };

    /// U+30C1 KATAKANA LETTER TI
    pub const kana_CHI: Self = Self { inner: 0x04c1 };

    /// deprecated
    pub const kana_TI: Self = Self { inner: 0x04c1 };

    /// U+30C4 KATAKANA LETTER TU
    pub const kana_TSU: Self = Self { inner: 0x04c2 };

    /// deprecated
    pub const kana_TU: Self = Self { inner: 0x04c2 };

    /// U+30C6 KATAKANA LETTER TE
    pub const kana_TE: Self = Self { inner: 0x04c3 };

    /// U+30C8 KATAKANA LETTER TO
    pub const kana_TO: Self = Self { inner: 0x04c4 };

    /// U+30CA KATAKANA LETTER NA
    pub const kana_NA: Self = Self { inner: 0x04c5 };

    /// U+30CB KATAKANA LETTER NI
    pub const kana_NI: Self = Self { inner: 0x04c6 };

    /// U+30CC KATAKANA LETTER NU
    pub const kana_NU: Self = Self { inner: 0x04c7 };

    /// U+30CD KATAKANA LETTER NE
    pub const kana_NE: Self = Self { inner: 0x04c8 };

    /// U+30CE KATAKANA LETTER NO
    pub const kana_NO: Self = Self { inner: 0x04c9 };

    /// U+30CF KATAKANA LETTER HA
    pub const kana_HA: Self = Self { inner: 0x04ca };

    /// U+30D2 KATAKANA LETTER HI
    pub const kana_HI: Self = Self { inner: 0x04cb };

    /// U+30D5 KATAKANA LETTER HU
    pub const kana_FU: Self = Self { inner: 0x04cc };

    /// deprecated
    pub const kana_HU: Self = Self { inner: 0x04cc };

    /// U+30D8 KATAKANA LETTER HE
    pub const kana_HE: Self = Self { inner: 0x04cd };

    /// U+30DB KATAKANA LETTER HO
    pub const kana_HO: Self = Self { inner: 0x04ce };

    /// U+30DE KATAKANA LETTER MA
    pub const kana_MA: Self = Self { inner: 0x04cf };

    /// U+30DF KATAKANA LETTER MI
    pub const kana_MI: Self = Self { inner: 0x04d0 };

    /// U+30E0 KATAKANA LETTER MU
    pub const kana_MU: Self = Self { inner: 0x04d1 };

    /// U+30E1 KATAKANA LETTER ME
    pub const kana_ME: Self = Self { inner: 0x04d2 };

    /// U+30E2 KATAKANA LETTER MO
    pub const kana_MO: Self = Self { inner: 0x04d3 };

    /// U+30E4 KATAKANA LETTER YA
    pub const kana_YA: Self = Self { inner: 0x04d4 };

    /// U+30E6 KATAKANA LETTER YU
    pub const kana_YU: Self = Self { inner: 0x04d5 };

    /// U+30E8 KATAKANA LETTER YO
    pub const kana_YO: Self = Self { inner: 0x04d6 };

    /// U+30E9 KATAKANA LETTER RA
    pub const kana_RA: Self = Self { inner: 0x04d7 };

    /// U+30EA KATAKANA LETTER RI
    pub const kana_RI: Self = Self { inner: 0x04d8 };

    /// U+30EB KATAKANA LETTER RU
    pub const kana_RU: Self = Self { inner: 0x04d9 };

    /// U+30EC KATAKANA LETTER RE
    pub const kana_RE: Self = Self { inner: 0x04da };

    /// U+30ED KATAKANA LETTER RO
    pub const kana_RO: Self = Self { inner: 0x04db };

    /// U+30EF KATAKANA LETTER WA
    pub const kana_WA: Self = Self { inner: 0x04dc };

    /// U+30F3 KATAKANA LETTER N
    pub const kana_N: Self = Self { inner: 0x04dd };

    /// U+309B KATAKANA-HIRAGANA VOICED SOUND MARK
    pub const voicedsound: Self = Self { inner: 0x04de };

    /// U+309C KATAKANA-HIRAGANA SEMI-VOICED SOUND MARK
    pub const semivoicedsound: Self = Self { inner: 0x04df };

    /// Alias for mode_switch
    pub const kana_switch: Self = Self { inner: 0xff7e };

    /// U+06F0 EXTENDED ARABIC-INDIC DIGIT ZERO
    pub const Farsi_0: Self = Self { inner: 0x10006f0 };

    /// U+06F1 EXTENDED ARABIC-INDIC DIGIT ONE
    pub const Farsi_1: Self = Self { inner: 0x10006f1 };

    /// U+06F2 EXTENDED ARABIC-INDIC DIGIT TWO
    pub const Farsi_2: Self = Self { inner: 0x10006f2 };

    /// U+06F3 EXTENDED ARABIC-INDIC DIGIT THREE
    pub const Farsi_3: Self = Self { inner: 0x10006f3 };

    /// U+06F4 EXTENDED ARABIC-INDIC DIGIT FOUR
    pub const Farsi_4: Self = Self { inner: 0x10006f4 };

    /// U+06F5 EXTENDED ARABIC-INDIC DIGIT FIVE
    pub const Farsi_5: Self = Self { inner: 0x10006f5 };

    /// U+06F6 EXTENDED ARABIC-INDIC DIGIT SIX
    pub const Farsi_6: Self = Self { inner: 0x10006f6 };

    /// U+06F7 EXTENDED ARABIC-INDIC DIGIT SEVEN
    pub const Farsi_7: Self = Self { inner: 0x10006f7 };

    /// U+06F8 EXTENDED ARABIC-INDIC DIGIT EIGHT
    pub const Farsi_8: Self = Self { inner: 0x10006f8 };

    /// U+06F9 EXTENDED ARABIC-INDIC DIGIT NINE
    pub const Farsi_9: Self = Self { inner: 0x10006f9 };

    /// U+066A ARABIC PERCENT SIGN
    pub const Arabic_percent: Self = Self { inner: 0x100066a };

    /// U+0670 ARABIC LETTER SUPERSCRIPT ALEF
    pub const Arabic_superscript_alef: Self = Self { inner: 0x1000670 };

    /// U+0679 ARABIC LETTER TTEH
    pub const Arabic_tteh: Self = Self { inner: 0x1000679 };

    /// U+067E ARABIC LETTER PEH
    pub const Arabic_peh: Self = Self { inner: 0x100067e };

    /// U+0686 ARABIC LETTER TCHEH
    pub const Arabic_tcheh: Self = Self { inner: 0x1000686 };

    /// U+0688 ARABIC LETTER DDAL
    pub const Arabic_ddal: Self = Self { inner: 0x1000688 };

    /// U+0691 ARABIC LETTER RREH
    pub const Arabic_rreh: Self = Self { inner: 0x1000691 };

    /// U+060C ARABIC COMMA
    pub const Arabic_comma: Self = Self { inner: 0x05ac };

    /// U+06D4 ARABIC FULL STOP
    pub const Arabic_fullstop: Self = Self { inner: 0x10006d4 };

    /// U+0660 ARABIC-INDIC DIGIT ZERO
    pub const Arabic_0: Self = Self { inner: 0x1000660 };

    /// U+0661 ARABIC-INDIC DIGIT ONE
    pub const Arabic_1: Self = Self { inner: 0x1000661 };

    /// U+0662 ARABIC-INDIC DIGIT TWO
    pub const Arabic_2: Self = Self { inner: 0x1000662 };

    /// U+0663 ARABIC-INDIC DIGIT THREE
    pub const Arabic_3: Self = Self { inner: 0x1000663 };

    /// U+0664 ARABIC-INDIC DIGIT FOUR
    pub const Arabic_4: Self = Self { inner: 0x1000664 };

    /// U+0665 ARABIC-INDIC DIGIT FIVE
    pub const Arabic_5: Self = Self { inner: 0x1000665 };

    /// U+0666 ARABIC-INDIC DIGIT SIX
    pub const Arabic_6: Self = Self { inner: 0x1000666 };

    /// U+0667 ARABIC-INDIC DIGIT SEVEN
    pub const Arabic_7: Self = Self { inner: 0x1000667 };

    /// U+0668 ARABIC-INDIC DIGIT EIGHT
    pub const Arabic_8: Self = Self { inner: 0x1000668 };

    /// U+0669 ARABIC-INDIC DIGIT NINE
    pub const Arabic_9: Self = Self { inner: 0x1000669 };

    /// U+061B ARABIC SEMICOLON
    pub const Arabic_semicolon: Self = Self { inner: 0x05bb };

    /// U+061F ARABIC QUESTION MARK
    pub const Arabic_question_mark: Self = Self { inner: 0x05bf };

    /// U+0621 ARABIC LETTER HAMZA
    pub const Arabic_hamza: Self = Self { inner: 0x05c1 };

    /// U+0622 ARABIC LETTER ALEF WITH MADDA ABOVE
    pub const Arabic_maddaonalef: Self = Self { inner: 0x05c2 };

    /// U+0623 ARABIC LETTER ALEF WITH HAMZA ABOVE
    pub const Arabic_hamzaonalef: Self = Self { inner: 0x05c3 };

    /// U+0624 ARABIC LETTER WAW WITH HAMZA ABOVE
    pub const Arabic_hamzaonwaw: Self = Self { inner: 0x05c4 };

    /// U+0625 ARABIC LETTER ALEF WITH HAMZA BELOW
    pub const Arabic_hamzaunderalef: Self = Self { inner: 0x05c5 };

    /// U+0626 ARABIC LETTER YEH WITH HAMZA ABOVE
    pub const Arabic_hamzaonyeh: Self = Self { inner: 0x05c6 };

    /// U+0627 ARABIC LETTER ALEF
    pub const Arabic_alef: Self = Self { inner: 0x05c7 };

    /// U+0628 ARABIC LETTER BEH
    pub const Arabic_beh: Self = Self { inner: 0x05c8 };

    /// U+0629 ARABIC LETTER TEH MARBUTA
    pub const Arabic_tehmarbuta: Self = Self { inner: 0x05c9 };

    /// U+062A ARABIC LETTER TEH
    pub const Arabic_teh: Self = Self { inner: 0x05ca };

    /// U+062B ARABIC LETTER THEH
    pub const Arabic_theh: Self = Self { inner: 0x05cb };

    /// U+062C ARABIC LETTER JEEM
    pub const Arabic_jeem: Self = Self { inner: 0x05cc };

    /// U+062D ARABIC LETTER HAH
    pub const Arabic_hah: Self = Self { inner: 0x05cd };

    /// U+062E ARABIC LETTER KHAH
    pub const Arabic_khah: Self = Self { inner: 0x05ce };

    /// U+062F ARABIC LETTER DAL
    pub const Arabic_dal: Self = Self { inner: 0x05cf };

    /// U+0630 ARABIC LETTER THAL
    pub const Arabic_thal: Self = Self { inner: 0x05d0 };

    /// U+0631 ARABIC LETTER REH
    pub const Arabic_ra: Self = Self { inner: 0x05d1 };

    /// U+0632 ARABIC LETTER ZAIN
    pub const Arabic_zain: Self = Self { inner: 0x05d2 };

    /// U+0633 ARABIC LETTER SEEN
    pub const Arabic_seen: Self = Self { inner: 0x05d3 };

    /// U+0634 ARABIC LETTER SHEEN
    pub const Arabic_sheen: Self = Self { inner: 0x05d4 };

    /// U+0635 ARABIC LETTER SAD
    pub const Arabic_sad: Self = Self { inner: 0x05d5 };

    /// U+0636 ARABIC LETTER DAD
    pub const Arabic_dad: Self = Self { inner: 0x05d6 };

    /// U+0637 ARABIC LETTER TAH
    pub const Arabic_tah: Self = Self { inner: 0x05d7 };

    /// U+0638 ARABIC LETTER ZAH
    pub const Arabic_zah: Self = Self { inner: 0x05d8 };

    /// U+0639 ARABIC LETTER AIN
    pub const Arabic_ain: Self = Self { inner: 0x05d9 };

    /// U+063A ARABIC LETTER GHAIN
    pub const Arabic_ghain: Self = Self { inner: 0x05da };

    /// U+0640 ARABIC TATWEEL
    pub const Arabic_tatweel: Self = Self { inner: 0x05e0 };

    /// U+0641 ARABIC LETTER FEH
    pub const Arabic_feh: Self = Self { inner: 0x05e1 };

    /// U+0642 ARABIC LETTER QAF
    pub const Arabic_qaf: Self = Self { inner: 0x05e2 };

    /// U+0643 ARABIC LETTER KAF
    pub const Arabic_kaf: Self = Self { inner: 0x05e3 };

    /// U+0644 ARABIC LETTER LAM
    pub const Arabic_lam: Self = Self { inner: 0x05e4 };

    /// U+0645 ARABIC LETTER MEEM
    pub const Arabic_meem: Self = Self { inner: 0x05e5 };

    /// U+0646 ARABIC LETTER NOON
    pub const Arabic_noon: Self = Self { inner: 0x05e6 };

    /// U+0647 ARABIC LETTER HEH
    pub const Arabic_ha: Self = Self { inner: 0x05e7 };

    /// deprecated
    pub const Arabic_heh: Self = Self { inner: 0x05e7 };

    /// U+0648 ARABIC LETTER WAW
    pub const Arabic_waw: Self = Self { inner: 0x05e8 };

    /// U+0649 ARABIC LETTER ALEF MAKSURA
    pub const Arabic_alefmaksura: Self = Self { inner: 0x05e9 };

    /// U+064A ARABIC LETTER YEH
    pub const Arabic_yeh: Self = Self { inner: 0x05ea };

    /// U+064B ARABIC FATHATAN
    pub const Arabic_fathatan: Self = Self { inner: 0x05eb };

    /// U+064C ARABIC DAMMATAN
    pub const Arabic_dammatan: Self = Self { inner: 0x05ec };

    /// U+064D ARABIC KASRATAN
    pub const Arabic_kasratan: Self = Self { inner: 0x05ed };

    /// U+064E ARABIC FATHA
    pub const Arabic_fatha: Self = Self { inner: 0x05ee };

    /// U+064F ARABIC DAMMA
    pub const Arabic_damma: Self = Self { inner: 0x05ef };

    /// U+0650 ARABIC KASRA
    pub const Arabic_kasra: Self = Self { inner: 0x05f0 };

    /// U+0651 ARABIC SHADDA
    pub const Arabic_shadda: Self = Self { inner: 0x05f1 };

    /// U+0652 ARABIC SUKUN
    pub const Arabic_sukun: Self = Self { inner: 0x05f2 };

    /// U+0653 ARABIC MADDAH ABOVE
    pub const Arabic_madda_above: Self = Self { inner: 0x1000653 };

    /// U+0654 ARABIC HAMZA ABOVE
    pub const Arabic_hamza_above: Self = Self { inner: 0x1000654 };

    /// U+0655 ARABIC HAMZA BELOW
    pub const Arabic_hamza_below: Self = Self { inner: 0x1000655 };

    /// U+0698 ARABIC LETTER JEH
    pub const Arabic_jeh: Self = Self { inner: 0x1000698 };

    /// U+06A4 ARABIC LETTER VEH
    pub const Arabic_veh: Self = Self { inner: 0x10006a4 };

    /// U+06A9 ARABIC LETTER KEHEH
    pub const Arabic_keheh: Self = Self { inner: 0x10006a9 };

    /// U+06AF ARABIC LETTER GAF
    pub const Arabic_gaf: Self = Self { inner: 0x10006af };

    /// U+06BA ARABIC LETTER NOON GHUNNA
    pub const Arabic_noon_ghunna: Self = Self { inner: 0x10006ba };

    /// U+06BE ARABIC LETTER HEH DOACHASHMEE
    pub const Arabic_heh_doachashmee: Self = Self { inner: 0x10006be };

    /// U+06CC ARABIC LETTER FARSI YEH
    pub const Farsi_yeh: Self = Self { inner: 0x10006cc };

    /// U+06CC ARABIC LETTER FARSI YEH
    pub const Arabic_farsi_yeh: Self = Self { inner: 0x10006cc };

    /// U+06D2 ARABIC LETTER YEH BARREE
    pub const Arabic_yeh_baree: Self = Self { inner: 0x10006d2 };

    /// U+06C1 ARABIC LETTER HEH GOAL
    pub const Arabic_heh_goal: Self = Self { inner: 0x10006c1 };

    /// Alias for mode_switch
    pub const Arabic_switch: Self = Self { inner: 0xff7e };

    /// U+0492 CYRILLIC CAPITAL LETTER GHE WITH STROKE
    pub const Cyrillic_GHE_bar: Self = Self { inner: 0x1000492 };

    /// U+0493 CYRILLIC SMALL LETTER GHE WITH STROKE
    pub const Cyrillic_ghe_bar: Self = Self { inner: 0x1000493 };

    /// U+0496 CYRILLIC CAPITAL LETTER ZHE WITH DESCENDER
    pub const Cyrillic_ZHE_descender: Self = Self { inner: 0x1000496 };

    /// U+0497 CYRILLIC SMALL LETTER ZHE WITH DESCENDER
    pub const Cyrillic_zhe_descender: Self = Self { inner: 0x1000497 };

    /// U+049A CYRILLIC CAPITAL LETTER KA WITH DESCENDER
    pub const Cyrillic_KA_descender: Self = Self { inner: 0x100049a };

    /// U+049B CYRILLIC SMALL LETTER KA WITH DESCENDER
    pub const Cyrillic_ka_descender: Self = Self { inner: 0x100049b };

    /// U+049C CYRILLIC CAPITAL LETTER KA WITH VERTICAL STROKE
    pub const Cyrillic_KA_vertstroke: Self = Self { inner: 0x100049c };

    /// U+049D CYRILLIC SMALL LETTER KA WITH VERTICAL STROKE
    pub const Cyrillic_ka_vertstroke: Self = Self { inner: 0x100049d };

    /// U+04A2 CYRILLIC CAPITAL LETTER EN WITH DESCENDER
    pub const Cyrillic_EN_descender: Self = Self { inner: 0x10004a2 };

    /// U+04A3 CYRILLIC SMALL LETTER EN WITH DESCENDER
    pub const Cyrillic_en_descender: Self = Self { inner: 0x10004a3 };

    /// U+04AE CYRILLIC CAPITAL LETTER STRAIGHT U
    pub const Cyrillic_U_straight: Self = Self { inner: 0x10004ae };

    /// U+04AF CYRILLIC SMALL LETTER STRAIGHT U
    pub const Cyrillic_u_straight: Self = Self { inner: 0x10004af };

    /// U+04B0 CYRILLIC CAPITAL LETTER STRAIGHT U WITH STROKE
    pub const Cyrillic_U_straight_bar: Self = Self { inner: 0x10004b0 };

    /// U+04B1 CYRILLIC SMALL LETTER STRAIGHT U WITH STROKE
    pub const Cyrillic_u_straight_bar: Self = Self { inner: 0x10004b1 };

    /// U+04B2 CYRILLIC CAPITAL LETTER HA WITH DESCENDER
    pub const Cyrillic_HA_descender: Self = Self { inner: 0x10004b2 };

    /// U+04B3 CYRILLIC SMALL LETTER HA WITH DESCENDER
    pub const Cyrillic_ha_descender: Self = Self { inner: 0x10004b3 };

    /// U+04B6 CYRILLIC CAPITAL LETTER CHE WITH DESCENDER
    pub const Cyrillic_CHE_descender: Self = Self { inner: 0x10004b6 };

    /// U+04B7 CYRILLIC SMALL LETTER CHE WITH DESCENDER
    pub const Cyrillic_che_descender: Self = Self { inner: 0x10004b7 };

    /// U+04B8 CYRILLIC CAPITAL LETTER CHE WITH VERTICAL STROKE
    pub const Cyrillic_CHE_vertstroke: Self = Self { inner: 0x10004b8 };

    /// U+04B9 CYRILLIC SMALL LETTER CHE WITH VERTICAL STROKE
    pub const Cyrillic_che_vertstroke: Self = Self { inner: 0x10004b9 };

    /// U+04BA CYRILLIC CAPITAL LETTER SHHA
    pub const Cyrillic_SHHA: Self = Self { inner: 0x10004ba };

    /// U+04BB CYRILLIC SMALL LETTER SHHA
    pub const Cyrillic_shha: Self = Self { inner: 0x10004bb };

    /// U+04D8 CYRILLIC CAPITAL LETTER SCHWA
    pub const Cyrillic_SCHWA: Self = Self { inner: 0x10004d8 };

    /// U+04D9 CYRILLIC SMALL LETTER SCHWA
    pub const Cyrillic_schwa: Self = Self { inner: 0x10004d9 };

    /// U+04E2 CYRILLIC CAPITAL LETTER I WITH MACRON
    pub const Cyrillic_I_macron: Self = Self { inner: 0x10004e2 };

    /// U+04E3 CYRILLIC SMALL LETTER I WITH MACRON
    pub const Cyrillic_i_macron: Self = Self { inner: 0x10004e3 };

    /// U+04E8 CYRILLIC CAPITAL LETTER BARRED O
    pub const Cyrillic_O_bar: Self = Self { inner: 0x10004e8 };

    /// U+04E9 CYRILLIC SMALL LETTER BARRED O
    pub const Cyrillic_o_bar: Self = Self { inner: 0x10004e9 };

    /// U+04EE CYRILLIC CAPITAL LETTER U WITH MACRON
    pub const Cyrillic_U_macron: Self = Self { inner: 0x10004ee };

    /// U+04EF CYRILLIC SMALL LETTER U WITH MACRON
    pub const Cyrillic_u_macron: Self = Self { inner: 0x10004ef };

    /// U+0452 CYRILLIC SMALL LETTER DJE
    pub const Serbian_dje: Self = Self { inner: 0x06a1 };

    /// U+0453 CYRILLIC SMALL LETTER GJE
    pub const Macedonia_gje: Self = Self { inner: 0x06a2 };

    /// U+0451 CYRILLIC SMALL LETTER IO
    pub const Cyrillic_io: Self = Self { inner: 0x06a3 };

    /// U+0454 CYRILLIC SMALL LETTER UKRAINIAN IE
    pub const Ukrainian_ie: Self = Self { inner: 0x06a4 };

    /// deprecated
    pub const Ukranian_je: Self = Self { inner: 0x06a4 };

    /// U+0455 CYRILLIC SMALL LETTER DZE
    pub const Macedonia_dse: Self = Self { inner: 0x06a5 };

    /// U+0456 CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
    pub const Ukrainian_i: Self = Self { inner: 0x06a6 };

    /// deprecated
    pub const Ukranian_i: Self = Self { inner: 0x06a6 };

    /// U+0457 CYRILLIC SMALL LETTER YI
    pub const Ukrainian_yi: Self = Self { inner: 0x06a7 };

    /// deprecated
    pub const Ukranian_yi: Self = Self { inner: 0x06a7 };

    /// U+0458 CYRILLIC SMALL LETTER JE
    pub const Cyrillic_je: Self = Self { inner: 0x06a8 };

    /// deprecated
    pub const Serbian_je: Self = Self { inner: 0x06a8 };

    /// U+0459 CYRILLIC SMALL LETTER LJE
    pub const Cyrillic_lje: Self = Self { inner: 0x06a9 };

    /// deprecated
    pub const Serbian_lje: Self = Self { inner: 0x06a9 };

    /// U+045A CYRILLIC SMALL LETTER NJE
    pub const Cyrillic_nje: Self = Self { inner: 0x06aa };

    /// deprecated
    pub const Serbian_nje: Self = Self { inner: 0x06aa };

    /// U+045B CYRILLIC SMALL LETTER TSHE
    pub const Serbian_tshe: Self = Self { inner: 0x06ab };

    /// U+045C CYRILLIC SMALL LETTER KJE
    pub const Macedonia_kje: Self = Self { inner: 0x06ac };

    /// U+0491 CYRILLIC SMALL LETTER GHE WITH UPTURN
    pub const Ukrainian_ghe_with_upturn: Self = Self { inner: 0x06ad };

    /// U+045E CYRILLIC SMALL LETTER SHORT U
    pub const Byelorussian_shortu: Self = Self { inner: 0x06ae };

    /// U+045F CYRILLIC SMALL LETTER DZHE
    pub const Cyrillic_dzhe: Self = Self { inner: 0x06af };

    /// deprecated
    pub const Serbian_dze: Self = Self { inner: 0x06af };

    /// U+2116 NUMERO SIGN
    pub const numerosign: Self = Self { inner: 0x06b0 };

    /// U+0402 CYRILLIC CAPITAL LETTER DJE
    pub const Serbian_DJE: Self = Self { inner: 0x06b1 };

    /// U+0403 CYRILLIC CAPITAL LETTER GJE
    pub const Macedonia_GJE: Self = Self { inner: 0x06b2 };

    /// U+0401 CYRILLIC CAPITAL LETTER IO
    pub const Cyrillic_IO: Self = Self { inner: 0x06b3 };

    /// U+0404 CYRILLIC CAPITAL LETTER UKRAINIAN IE
    pub const Ukrainian_IE: Self = Self { inner: 0x06b4 };

    /// deprecated
    pub const Ukranian_JE: Self = Self { inner: 0x06b4 };

    /// U+0405 CYRILLIC CAPITAL LETTER DZE
    pub const Macedonia_DSE: Self = Self { inner: 0x06b5 };

    /// U+0406 CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I
    pub const Ukrainian_I: Self = Self { inner: 0x06b6 };

    /// deprecated
    pub const Ukranian_I: Self = Self { inner: 0x06b6 };

    /// U+0407 CYRILLIC CAPITAL LETTER YI
    pub const Ukrainian_YI: Self = Self { inner: 0x06b7 };

    /// deprecated
    pub const Ukranian_YI: Self = Self { inner: 0x06b7 };

    /// U+0408 CYRILLIC CAPITAL LETTER JE
    pub const Cyrillic_JE: Self = Self { inner: 0x06b8 };

    /// deprecated
    pub const Serbian_JE: Self = Self { inner: 0x06b8 };

    /// U+0409 CYRILLIC CAPITAL LETTER LJE
    pub const Cyrillic_LJE: Self = Self { inner: 0x06b9 };

    /// deprecated
    pub const Serbian_LJE: Self = Self { inner: 0x06b9 };

    /// U+040A CYRILLIC CAPITAL LETTER NJE
    pub const Cyrillic_NJE: Self = Self { inner: 0x06ba };

    /// deprecated
    pub const Serbian_NJE: Self = Self { inner: 0x06ba };

    /// U+040B CYRILLIC CAPITAL LETTER TSHE
    pub const Serbian_TSHE: Self = Self { inner: 0x06bb };

    /// U+040C CYRILLIC CAPITAL LETTER KJE
    pub const Macedonia_KJE: Self = Self { inner: 0x06bc };

    /// U+0490 CYRILLIC CAPITAL LETTER GHE WITH UPTURN
    pub const Ukrainian_GHE_WITH_UPTURN: Self = Self { inner: 0x06bd };

    /// U+040E CYRILLIC CAPITAL LETTER SHORT U
    pub const Byelorussian_SHORTU: Self = Self { inner: 0x06be };

    /// U+040F CYRILLIC CAPITAL LETTER DZHE
    pub const Cyrillic_DZHE: Self = Self { inner: 0x06bf };

    /// deprecated
    pub const Serbian_DZE: Self = Self { inner: 0x06bf };

    /// U+044E CYRILLIC SMALL LETTER YU
    pub const Cyrillic_yu: Self = Self { inner: 0x06c0 };

    /// U+0430 CYRILLIC SMALL LETTER A
    pub const Cyrillic_a: Self = Self { inner: 0x06c1 };

    /// U+0431 CYRILLIC SMALL LETTER BE
    pub const Cyrillic_be: Self = Self { inner: 0x06c2 };

    /// U+0446 CYRILLIC SMALL LETTER TSE
    pub const Cyrillic_tse: Self = Self { inner: 0x06c3 };

    /// U+0434 CYRILLIC SMALL LETTER DE
    pub const Cyrillic_de: Self = Self { inner: 0x06c4 };

    /// U+0435 CYRILLIC SMALL LETTER IE
    pub const Cyrillic_ie: Self = Self { inner: 0x06c5 };

    /// U+0444 CYRILLIC SMALL LETTER EF
    pub const Cyrillic_ef: Self = Self { inner: 0x06c6 };

    /// U+0433 CYRILLIC SMALL LETTER GHE
    pub const Cyrillic_ghe: Self = Self { inner: 0x06c7 };

    /// U+0445 CYRILLIC SMALL LETTER HA
    pub const Cyrillic_ha: Self = Self { inner: 0x06c8 };

    /// U+0438 CYRILLIC SMALL LETTER I
    pub const Cyrillic_i: Self = Self { inner: 0x06c9 };

    /// U+0439 CYRILLIC SMALL LETTER SHORT I
    pub const Cyrillic_shorti: Self = Self { inner: 0x06ca };

    /// U+043A CYRILLIC SMALL LETTER KA
    pub const Cyrillic_ka: Self = Self { inner: 0x06cb };

    /// U+043B CYRILLIC SMALL LETTER EL
    pub const Cyrillic_el: Self = Self { inner: 0x06cc };

    /// U+043C CYRILLIC SMALL LETTER EM
    pub const Cyrillic_em: Self = Self { inner: 0x06cd };

    /// U+043D CYRILLIC SMALL LETTER EN
    pub const Cyrillic_en: Self = Self { inner: 0x06ce };

    /// U+043E CYRILLIC SMALL LETTER O
    pub const Cyrillic_o: Self = Self { inner: 0x06cf };

    /// U+043F CYRILLIC SMALL LETTER PE
    pub const Cyrillic_pe: Self = Self { inner: 0x06d0 };

    /// U+044F CYRILLIC SMALL LETTER YA
    pub const Cyrillic_ya: Self = Self { inner: 0x06d1 };

    /// U+0440 CYRILLIC SMALL LETTER ER
    pub const Cyrillic_er: Self = Self { inner: 0x06d2 };

    /// U+0441 CYRILLIC SMALL LETTER ES
    pub const Cyrillic_es: Self = Self { inner: 0x06d3 };

    /// U+0442 CYRILLIC SMALL LETTER TE
    pub const Cyrillic_te: Self = Self { inner: 0x06d4 };

    /// U+0443 CYRILLIC SMALL LETTER U
    pub const Cyrillic_u: Self = Self { inner: 0x06d5 };

    /// U+0436 CYRILLIC SMALL LETTER ZHE
    pub const Cyrillic_zhe: Self = Self { inner: 0x06d6 };

    /// U+0432 CYRILLIC SMALL LETTER VE
    pub const Cyrillic_ve: Self = Self { inner: 0x06d7 };

    /// U+044C CYRILLIC SMALL LETTER SOFT SIGN
    pub const Cyrillic_softsign: Self = Self { inner: 0x06d8 };

    /// U+044B CYRILLIC SMALL LETTER YERU
    pub const Cyrillic_yeru: Self = Self { inner: 0x06d9 };

    /// U+0437 CYRILLIC SMALL LETTER ZE
    pub const Cyrillic_ze: Self = Self { inner: 0x06da };

    /// U+0448 CYRILLIC SMALL LETTER SHA
    pub const Cyrillic_sha: Self = Self { inner: 0x06db };

    /// U+044D CYRILLIC SMALL LETTER E
    pub const Cyrillic_e: Self = Self { inner: 0x06dc };

    /// U+0449 CYRILLIC SMALL LETTER SHCHA
    pub const Cyrillic_shcha: Self = Self { inner: 0x06dd };

    /// U+0447 CYRILLIC SMALL LETTER CHE
    pub const Cyrillic_che: Self = Self { inner: 0x06de };

    /// U+044A CYRILLIC SMALL LETTER HARD SIGN
    pub const Cyrillic_hardsign: Self = Self { inner: 0x06df };

    /// U+042E CYRILLIC CAPITAL LETTER YU
    pub const Cyrillic_YU: Self = Self { inner: 0x06e0 };

    /// U+0410 CYRILLIC CAPITAL LETTER A
    pub const Cyrillic_A: Self = Self { inner: 0x06e1 };

    /// U+0411 CYRILLIC CAPITAL LETTER BE
    pub const Cyrillic_BE: Self = Self { inner: 0x06e2 };

    /// U+0426 CYRILLIC CAPITAL LETTER TSE
    pub const Cyrillic_TSE: Self = Self { inner: 0x06e3 };

    /// U+0414 CYRILLIC CAPITAL LETTER DE
    pub const Cyrillic_DE: Self = Self { inner: 0x06e4 };

    /// U+0415 CYRILLIC CAPITAL LETTER IE
    pub const Cyrillic_IE: Self = Self { inner: 0x06e5 };

    /// U+0424 CYRILLIC CAPITAL LETTER EF
    pub const Cyrillic_EF: Self = Self { inner: 0x06e6 };

    /// U+0413 CYRILLIC CAPITAL LETTER GHE
    pub const Cyrillic_GHE: Self = Self { inner: 0x06e7 };

    /// U+0425 CYRILLIC CAPITAL LETTER HA
    pub const Cyrillic_HA: Self = Self { inner: 0x06e8 };

    /// U+0418 CYRILLIC CAPITAL LETTER I
    pub const Cyrillic_I: Self = Self { inner: 0x06e9 };

    /// U+0419 CYRILLIC CAPITAL LETTER SHORT I
    pub const Cyrillic_SHORTI: Self = Self { inner: 0x06ea };

    /// U+041A CYRILLIC CAPITAL LETTER KA
    pub const Cyrillic_KA: Self = Self { inner: 0x06eb };

    /// U+041B CYRILLIC CAPITAL LETTER EL
    pub const Cyrillic_EL: Self = Self { inner: 0x06ec };

    /// U+041C CYRILLIC CAPITAL LETTER EM
    pub const Cyrillic_EM: Self = Self { inner: 0x06ed };

    /// U+041D CYRILLIC CAPITAL LETTER EN
    pub const Cyrillic_EN: Self = Self { inner: 0x06ee };

    /// U+041E CYRILLIC CAPITAL LETTER O
    pub const Cyrillic_O: Self = Self { inner: 0x06ef };

    /// U+041F CYRILLIC CAPITAL LETTER PE
    pub const Cyrillic_PE: Self = Self { inner: 0x06f0 };

    /// U+042F CYRILLIC CAPITAL LETTER YA
    pub const Cyrillic_YA: Self = Self { inner: 0x06f1 };

    /// U+0420 CYRILLIC CAPITAL LETTER ER
    pub const Cyrillic_ER: Self = Self { inner: 0x06f2 };

    /// U+0421 CYRILLIC CAPITAL LETTER ES
    pub const Cyrillic_ES: Self = Self { inner: 0x06f3 };

    /// U+0422 CYRILLIC CAPITAL LETTER TE
    pub const Cyrillic_TE: Self = Self { inner: 0x06f4 };

    /// U+0423 CYRILLIC CAPITAL LETTER U
    pub const Cyrillic_U: Self = Self { inner: 0x06f5 };

    /// U+0416 CYRILLIC CAPITAL LETTER ZHE
    pub const Cyrillic_ZHE: Self = Self { inner: 0x06f6 };

    /// U+0412 CYRILLIC CAPITAL LETTER VE
    pub const Cyrillic_VE: Self = Self { inner: 0x06f7 };

    /// U+042C CYRILLIC CAPITAL LETTER SOFT SIGN
    pub const Cyrillic_SOFTSIGN: Self = Self { inner: 0x06f8 };

    /// U+042B CYRILLIC CAPITAL LETTER YERU
    pub const Cyrillic_YERU: Self = Self { inner: 0x06f9 };

    /// U+0417 CYRILLIC CAPITAL LETTER ZE
    pub const Cyrillic_ZE: Self = Self { inner: 0x06fa };

    /// U+0428 CYRILLIC CAPITAL LETTER SHA
    pub const Cyrillic_SHA: Self = Self { inner: 0x06fb };

    /// U+042D CYRILLIC CAPITAL LETTER E
    pub const Cyrillic_E: Self = Self { inner: 0x06fc };

    /// U+0429 CYRILLIC CAPITAL LETTER SHCHA
    pub const Cyrillic_SHCHA: Self = Self { inner: 0x06fd };

    /// U+0427 CYRILLIC CAPITAL LETTER CHE
    pub const Cyrillic_CHE: Self = Self { inner: 0x06fe };

    /// U+042A CYRILLIC CAPITAL LETTER HARD SIGN
    pub const Cyrillic_HARDSIGN: Self = Self { inner: 0x06ff };

    /// U+0386 GREEK CAPITAL LETTER ALPHA WITH TONOS
    pub const Greek_ALPHAaccent: Self = Self { inner: 0x07a1 };

    /// U+0388 GREEK CAPITAL LETTER EPSILON WITH TONOS
    pub const Greek_EPSILONaccent: Self = Self { inner: 0x07a2 };

    /// U+0389 GREEK CAPITAL LETTER ETA WITH TONOS
    pub const Greek_ETAaccent: Self = Self { inner: 0x07a3 };

    /// U+038A GREEK CAPITAL LETTER IOTA WITH TONOS
    pub const Greek_IOTAaccent: Self = Self { inner: 0x07a4 };

    /// U+03AA GREEK CAPITAL LETTER IOTA WITH DIALYTIKA
    pub const Greek_IOTAdieresis: Self = Self { inner: 0x07a5 };

    /// old typo
    pub const Greek_IOTAdiaeresis: Self = Self { inner: 0x07a5 };

    /// U+038C GREEK CAPITAL LETTER OMICRON WITH TONOS
    pub const Greek_OMICRONaccent: Self = Self { inner: 0x07a7 };

    /// U+038E GREEK CAPITAL LETTER UPSILON WITH TONOS
    pub const Greek_UPSILONaccent: Self = Self { inner: 0x07a8 };

    /// U+03AB GREEK CAPITAL LETTER UPSILON WITH DIALYTIKA
    pub const Greek_UPSILONdieresis: Self = Self { inner: 0x07a9 };

    /// U+038F GREEK CAPITAL LETTER OMEGA WITH TONOS
    pub const Greek_OMEGAaccent: Self = Self { inner: 0x07ab };

    /// U+0385 GREEK DIALYTIKA TONOS
    pub const Greek_accentdieresis: Self = Self { inner: 0x07ae };

    /// U+2015 HORIZONTAL BAR
    pub const Greek_horizbar: Self = Self { inner: 0x07af };

    /// U+03AC GREEK SMALL LETTER ALPHA WITH TONOS
    pub const Greek_alphaaccent: Self = Self { inner: 0x07b1 };

    /// U+03AD GREEK SMALL LETTER EPSILON WITH TONOS
    pub const Greek_epsilonaccent: Self = Self { inner: 0x07b2 };

    /// U+03AE GREEK SMALL LETTER ETA WITH TONOS
    pub const Greek_etaaccent: Self = Self { inner: 0x07b3 };

    /// U+03AF GREEK SMALL LETTER IOTA WITH TONOS
    pub const Greek_iotaaccent: Self = Self { inner: 0x07b4 };

    /// U+03CA GREEK SMALL LETTER IOTA WITH DIALYTIKA
    pub const Greek_iotadieresis: Self = Self { inner: 0x07b5 };

    /// U+0390 GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS
    pub const Greek_iotaaccentdieresis: Self = Self { inner: 0x07b6 };

    /// U+03CC GREEK SMALL LETTER OMICRON WITH TONOS
    pub const Greek_omicronaccent: Self = Self { inner: 0x07b7 };

    /// U+03CD GREEK SMALL LETTER UPSILON WITH TONOS
    pub const Greek_upsilonaccent: Self = Self { inner: 0x07b8 };

    /// U+03CB GREEK SMALL LETTER UPSILON WITH DIALYTIKA
    pub const Greek_upsilondieresis: Self = Self { inner: 0x07b9 };

    /// U+03B0 GREEK SMALL LETTER UPSILON WITH DIALYTIKA AND TONOS
    pub const Greek_upsilonaccentdieresis: Self = Self { inner: 0x07ba };

    /// U+03CE GREEK SMALL LETTER OMEGA WITH TONOS
    pub const Greek_omegaaccent: Self = Self { inner: 0x07bb };

    /// U+0391 GREEK CAPITAL LETTER ALPHA
    pub const Greek_ALPHA: Self = Self { inner: 0x07c1 };

    /// U+0392 GREEK CAPITAL LETTER BETA
    pub const Greek_BETA: Self = Self { inner: 0x07c2 };

    /// U+0393 GREEK CAPITAL LETTER GAMMA
    pub const Greek_GAMMA: Self = Self { inner: 0x07c3 };

    /// U+0394 GREEK CAPITAL LETTER DELTA
    pub const Greek_DELTA: Self = Self { inner: 0x07c4 };

    /// U+0395 GREEK CAPITAL LETTER EPSILON
    pub const Greek_EPSILON: Self = Self { inner: 0x07c5 };

    /// U+0396 GREEK CAPITAL LETTER ZETA
    pub const Greek_ZETA: Self = Self { inner: 0x07c6 };

    /// U+0397 GREEK CAPITAL LETTER ETA
    pub const Greek_ETA: Self = Self { inner: 0x07c7 };

    /// U+0398 GREEK CAPITAL LETTER THETA
    pub const Greek_THETA: Self = Self { inner: 0x07c8 };

    /// U+0399 GREEK CAPITAL LETTER IOTA
    pub const Greek_IOTA: Self = Self { inner: 0x07c9 };

    /// U+039A GREEK CAPITAL LETTER KAPPA
    pub const Greek_KAPPA: Self = Self { inner: 0x07ca };

    /// U+039B GREEK CAPITAL LETTER LAMDA
    pub const Greek_LAMDA: Self = Self { inner: 0x07cb };

    /// U+039B GREEK CAPITAL LETTER LAMDA
    pub const Greek_LAMBDA: Self = Self { inner: 0x07cb };

    /// U+039C GREEK CAPITAL LETTER MU
    pub const Greek_MU: Self = Self { inner: 0x07cc };

    /// U+039D GREEK CAPITAL LETTER NU
    pub const Greek_NU: Self = Self { inner: 0x07cd };

    /// U+039E GREEK CAPITAL LETTER XI
    pub const Greek_XI: Self = Self { inner: 0x07ce };

    /// U+039F GREEK CAPITAL LETTER OMICRON
    pub const Greek_OMICRON: Self = Self { inner: 0x07cf };

    /// U+03A0 GREEK CAPITAL LETTER PI
    pub const Greek_PI: Self = Self { inner: 0x07d0 };

    /// U+03A1 GREEK CAPITAL LETTER RHO
    pub const Greek_RHO: Self = Self { inner: 0x07d1 };

    /// U+03A3 GREEK CAPITAL LETTER SIGMA
    pub const Greek_SIGMA: Self = Self { inner: 0x07d2 };

    /// U+03A4 GREEK CAPITAL LETTER TAU
    pub const Greek_TAU: Self = Self { inner: 0x07d4 };

    /// U+03A5 GREEK CAPITAL LETTER UPSILON
    pub const Greek_UPSILON: Self = Self { inner: 0x07d5 };

    /// U+03A6 GREEK CAPITAL LETTER PHI
    pub const Greek_PHI: Self = Self { inner: 0x07d6 };

    /// U+03A7 GREEK CAPITAL LETTER CHI
    pub const Greek_CHI: Self = Self { inner: 0x07d7 };

    /// U+03A8 GREEK CAPITAL LETTER PSI
    pub const Greek_PSI: Self = Self { inner: 0x07d8 };

    /// U+03A9 GREEK CAPITAL LETTER OMEGA
    pub const Greek_OMEGA: Self = Self { inner: 0x07d9 };

    /// U+03B1 GREEK SMALL LETTER ALPHA
    pub const Greek_alpha: Self = Self { inner: 0x07e1 };

    /// U+03B2 GREEK SMALL LETTER BETA
    pub const Greek_beta: Self = Self { inner: 0x07e2 };

    /// U+03B3 GREEK SMALL LETTER GAMMA
    pub const Greek_gamma: Self = Self { inner: 0x07e3 };

    /// U+03B4 GREEK SMALL LETTER DELTA
    pub const Greek_delta: Self = Self { inner: 0x07e4 };

    /// U+03B5 GREEK SMALL LETTER EPSILON
    pub const Greek_epsilon: Self = Self { inner: 0x07e5 };

    /// U+03B6 GREEK SMALL LETTER ZETA
    pub const Greek_zeta: Self = Self { inner: 0x07e6 };

    /// U+03B7 GREEK SMALL LETTER ETA
    pub const Greek_eta: Self = Self { inner: 0x07e7 };

    /// U+03B8 GREEK SMALL LETTER THETA
    pub const Greek_theta: Self = Self { inner: 0x07e8 };

    /// U+03B9 GREEK SMALL LETTER IOTA
    pub const Greek_iota: Self = Self { inner: 0x07e9 };

    /// U+03BA GREEK SMALL LETTER KAPPA
    pub const Greek_kappa: Self = Self { inner: 0x07ea };

    /// U+03BB GREEK SMALL LETTER LAMDA
    pub const Greek_lamda: Self = Self { inner: 0x07eb };

    /// U+03BB GREEK SMALL LETTER LAMDA
    pub const Greek_lambda: Self = Self { inner: 0x07eb };

    /// U+03BC GREEK SMALL LETTER MU
    pub const Greek_mu: Self = Self { inner: 0x07ec };

    /// U+03BD GREEK SMALL LETTER NU
    pub const Greek_nu: Self = Self { inner: 0x07ed };

    /// U+03BE GREEK SMALL LETTER XI
    pub const Greek_xi: Self = Self { inner: 0x07ee };

    /// U+03BF GREEK SMALL LETTER OMICRON
    pub const Greek_omicron: Self = Self { inner: 0x07ef };

    /// U+03C0 GREEK SMALL LETTER PI
    pub const Greek_pi: Self = Self { inner: 0x07f0 };

    /// U+03C1 GREEK SMALL LETTER RHO
    pub const Greek_rho: Self = Self { inner: 0x07f1 };

    /// U+03C3 GREEK SMALL LETTER SIGMA
    pub const Greek_sigma: Self = Self { inner: 0x07f2 };

    /// U+03C2 GREEK SMALL LETTER FINAL SIGMA
    pub const Greek_finalsmallsigma: Self = Self { inner: 0x07f3 };

    /// U+03C4 GREEK SMALL LETTER TAU
    pub const Greek_tau: Self = Self { inner: 0x07f4 };

    /// U+03C5 GREEK SMALL LETTER UPSILON
    pub const Greek_upsilon: Self = Self { inner: 0x07f5 };

    /// U+03C6 GREEK SMALL LETTER PHI
    pub const Greek_phi: Self = Self { inner: 0x07f6 };

    /// U+03C7 GREEK SMALL LETTER CHI
    pub const Greek_chi: Self = Self { inner: 0x07f7 };

    /// U+03C8 GREEK SMALL LETTER PSI
    pub const Greek_psi: Self = Self { inner: 0x07f8 };

    /// U+03C9 GREEK SMALL LETTER OMEGA
    pub const Greek_omega: Self = Self { inner: 0x07f9 };

    /// Alias for mode_switch
    pub const Greek_switch: Self = Self { inner: 0xff7e };

    /// U+23B7 RADICAL SYMBOL BOTTOM
    pub const leftradical: Self = Self { inner: 0x08a1 };

    /// (U+250C BOX DRAWINGS LIGHT DOWN AND RIGHT)
    pub const topleftradical: Self = Self { inner: 0x08a2 };

    /// (U+2500 BOX DRAWINGS LIGHT HORIZONTAL)
    pub const horizconnector: Self = Self { inner: 0x08a3 };

    /// U+2320 TOP HALF INTEGRAL
    pub const topintegral: Self = Self { inner: 0x08a4 };

    /// U+2321 BOTTOM HALF INTEGRAL
    pub const botintegral: Self = Self { inner: 0x08a5 };

    /// (U+2502 BOX DRAWINGS LIGHT VERTICAL)
    pub const vertconnector: Self = Self { inner: 0x08a6 };

    /// U+23A1 LEFT SQUARE BRACKET UPPER CORNER
    pub const topleftsqbracket: Self = Self { inner: 0x08a7 };

    /// U+23A3 LEFT SQUARE BRACKET LOWER CORNER
    pub const botleftsqbracket: Self = Self { inner: 0x08a8 };

    /// U+23A4 RIGHT SQUARE BRACKET UPPER CORNER
    pub const toprightsqbracket: Self = Self { inner: 0x08a9 };

    /// U+23A6 RIGHT SQUARE BRACKET LOWER CORNER
    pub const botrightsqbracket: Self = Self { inner: 0x08aa };

    /// U+239B LEFT PARENTHESIS UPPER HOOK
    pub const topleftparens: Self = Self { inner: 0x08ab };

    /// U+239D LEFT PARENTHESIS LOWER HOOK
    pub const botleftparens: Self = Self { inner: 0x08ac };

    /// U+239E RIGHT PARENTHESIS UPPER HOOK
    pub const toprightparens: Self = Self { inner: 0x08ad };

    /// U+23A0 RIGHT PARENTHESIS LOWER HOOK
    pub const botrightparens: Self = Self { inner: 0x08ae };

    /// U+23A8 LEFT CURLY BRACKET MIDDLE PIECE
    pub const leftmiddlecurlybrace: Self = Self { inner: 0x08af };

    /// U+23AC RIGHT CURLY BRACKET MIDDLE PIECE
    pub const rightmiddlecurlybrace: Self = Self { inner: 0x08b0 };

    pub const topleftsummation: Self = Self { inner: 0x08b1 };

    pub const botleftsummation: Self = Self { inner: 0x08b2 };

    pub const topvertsummationconnector: Self = Self { inner: 0x08b3 };

    pub const botvertsummationconnector: Self = Self { inner: 0x08b4 };

    pub const toprightsummation: Self = Self { inner: 0x08b5 };

    pub const botrightsummation: Self = Self { inner: 0x08b6 };

    pub const rightmiddlesummation: Self = Self { inner: 0x08b7 };

    /// U+2264 LESS-THAN OR EQUAL TO
    pub const lessthanequal: Self = Self { inner: 0x08bc };

    /// U+2260 NOT EQUAL TO
    pub const notequal: Self = Self { inner: 0x08bd };

    /// U+2265 GREATER-THAN OR EQUAL TO
    pub const greaterthanequal: Self = Self { inner: 0x08be };

    /// U+222B INTEGRAL
    pub const integral: Self = Self { inner: 0x08bf };

    /// U+2234 THEREFORE
    pub const therefore: Self = Self { inner: 0x08c0 };

    /// U+221D PROPORTIONAL TO
    pub const variation: Self = Self { inner: 0x08c1 };

    /// U+221E INFINITY
    pub const infinity: Self = Self { inner: 0x08c2 };

    /// U+2207 NABLA
    pub const nabla: Self = Self { inner: 0x08c5 };

    /// U+223C TILDE OPERATOR
    pub const approximate: Self = Self { inner: 0x08c8 };

    /// U+2243 ASYMPTOTICALLY EQUAL TO
    pub const similarequal: Self = Self { inner: 0x08c9 };

    /// U+21D4 LEFT RIGHT DOUBLE ARROW
    pub const ifonlyif: Self = Self { inner: 0x08cd };

    /// U+21D2 RIGHTWARDS DOUBLE ARROW
    pub const implies: Self = Self { inner: 0x08ce };

    /// U+2261 IDENTICAL TO
    pub const identical: Self = Self { inner: 0x08cf };

    /// U+221A SQUARE ROOT
    pub const radical: Self = Self { inner: 0x08d6 };

    /// U+2282 SUBSET OF
    pub const includedin: Self = Self { inner: 0x08da };

    /// U+2283 SUPERSET OF
    pub const includes: Self = Self { inner: 0x08db };

    /// U+2229 INTERSECTION
    pub const intersection: Self = Self { inner: 0x08dc };

    /// U+222A UNION
    pub const union: Self = Self { inner: 0x08dd };

    /// U+2227 LOGICAL AND
    pub const logicaland: Self = Self { inner: 0x08de };

    /// U+2228 LOGICAL OR
    pub const logicalor: Self = Self { inner: 0x08df };

    /// U+2202 PARTIAL DIFFERENTIAL
    pub const partialderivative: Self = Self { inner: 0x08ef };

    /// U+0192 LATIN SMALL LETTER F WITH HOOK
    pub const function: Self = Self { inner: 0x08f6 };

    /// U+2190 LEFTWARDS ARROW
    pub const leftarrow: Self = Self { inner: 0x08fb };

    /// U+2191 UPWARDS ARROW
    pub const uparrow: Self = Self { inner: 0x08fc };

    /// U+2192 RIGHTWARDS ARROW
    pub const rightarrow: Self = Self { inner: 0x08fd };

    /// U+2193 DOWNWARDS ARROW
    pub const downarrow: Self = Self { inner: 0x08fe };

    pub const blank: Self = Self { inner: 0x09df };

    /// U+25C6 BLACK DIAMOND
    pub const soliddiamond: Self = Self { inner: 0x09e0 };

    /// U+2592 MEDIUM SHADE
    pub const checkerboard: Self = Self { inner: 0x09e1 };

    /// U+2409 SYMBOL FOR HORIZONTAL TABULATION
    pub const ht: Self = Self { inner: 0x09e2 };

    /// U+240C SYMBOL FOR FORM FEED
    pub const ff: Self = Self { inner: 0x09e3 };

    /// U+240D SYMBOL FOR CARRIAGE RETURN
    pub const cr: Self = Self { inner: 0x09e4 };

    /// U+240A SYMBOL FOR LINE FEED
    pub const lf: Self = Self { inner: 0x09e5 };

    /// U+2424 SYMBOL FOR NEWLINE
    pub const nl: Self = Self { inner: 0x09e8 };

    /// U+240B SYMBOL FOR VERTICAL TABULATION
    pub const vt: Self = Self { inner: 0x09e9 };

    /// U+2518 BOX DRAWINGS LIGHT UP AND LEFT
    pub const lowrightcorner: Self = Self { inner: 0x09ea };

    /// U+2510 BOX DRAWINGS LIGHT DOWN AND LEFT
    pub const uprightcorner: Self = Self { inner: 0x09eb };

    /// U+250C BOX DRAWINGS LIGHT DOWN AND RIGHT
    pub const upleftcorner: Self = Self { inner: 0x09ec };

    /// U+2514 BOX DRAWINGS LIGHT UP AND RIGHT
    pub const lowleftcorner: Self = Self { inner: 0x09ed };

    /// U+253C BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL
    pub const crossinglines: Self = Self { inner: 0x09ee };

    /// U+23BA HORIZONTAL SCAN LINE-1
    pub const horizlinescan1: Self = Self { inner: 0x09ef };

    /// U+23BB HORIZONTAL SCAN LINE-3
    pub const horizlinescan3: Self = Self { inner: 0x09f0 };

    /// U+2500 BOX DRAWINGS LIGHT HORIZONTAL
    pub const horizlinescan5: Self = Self { inner: 0x09f1 };

    /// U+23BC HORIZONTAL SCAN LINE-7
    pub const horizlinescan7: Self = Self { inner: 0x09f2 };

    /// U+23BD HORIZONTAL SCAN LINE-9
    pub const horizlinescan9: Self = Self { inner: 0x09f3 };

    /// U+251C BOX DRAWINGS LIGHT VERTICAL AND RIGHT
    pub const leftt: Self = Self { inner: 0x09f4 };

    /// U+2524 BOX DRAWINGS LIGHT VERTICAL AND LEFT
    pub const rightt: Self = Self { inner: 0x09f5 };

    /// U+2534 BOX DRAWINGS LIGHT UP AND HORIZONTAL
    pub const bott: Self = Self { inner: 0x09f6 };

    /// U+252C BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
    pub const topt: Self = Self { inner: 0x09f7 };

    /// U+2502 BOX DRAWINGS LIGHT VERTICAL
    pub const vertbar: Self = Self { inner: 0x09f8 };

    /// U+2003 EM SPACE
    pub const emspace: Self = Self { inner: 0x0aa1 };

    /// U+2002 EN SPACE
    pub const enspace: Self = Self { inner: 0x0aa2 };

    /// U+2004 THREE-PER-EM SPACE
    pub const em3space: Self = Self { inner: 0x0aa3 };

    /// U+2005 FOUR-PER-EM SPACE
    pub const em4space: Self = Self { inner: 0x0aa4 };

    /// U+2007 FIGURE SPACE
    pub const digitspace: Self = Self { inner: 0x0aa5 };

    /// U+2008 PUNCTUATION SPACE
    pub const punctspace: Self = Self { inner: 0x0aa6 };

    /// U+2009 THIN SPACE
    pub const thinspace: Self = Self { inner: 0x0aa7 };

    /// U+200A HAIR SPACE
    pub const hairspace: Self = Self { inner: 0x0aa8 };

    /// U+2014 EM DASH
    pub const emdash: Self = Self { inner: 0x0aa9 };

    /// U+2013 EN DASH
    pub const endash: Self = Self { inner: 0x0aaa };

    /// (U+2423 OPEN BOX)
    pub const signifblank: Self = Self { inner: 0x0aac };

    /// U+2026 HORIZONTAL ELLIPSIS
    pub const ellipsis: Self = Self { inner: 0x0aae };

    /// U+2025 TWO DOT LEADER
    pub const doubbaselinedot: Self = Self { inner: 0x0aaf };

    /// U+2153 VULGAR FRACTION ONE THIRD
    pub const onethird: Self = Self { inner: 0x0ab0 };

    /// U+2154 VULGAR FRACTION TWO THIRDS
    pub const twothirds: Self = Self { inner: 0x0ab1 };

    /// U+2155 VULGAR FRACTION ONE FIFTH
    pub const onefifth: Self = Self { inner: 0x0ab2 };

    /// U+2156 VULGAR FRACTION TWO FIFTHS
    pub const twofifths: Self = Self { inner: 0x0ab3 };

    /// U+2157 VULGAR FRACTION THREE FIFTHS
    pub const threefifths: Self = Self { inner: 0x0ab4 };

    /// U+2158 VULGAR FRACTION FOUR FIFTHS
    pub const fourfifths: Self = Self { inner: 0x0ab5 };

    /// U+2159 VULGAR FRACTION ONE SIXTH
    pub const onesixth: Self = Self { inner: 0x0ab6 };

    /// U+215A VULGAR FRACTION FIVE SIXTHS
    pub const fivesixths: Self = Self { inner: 0x0ab7 };

    /// U+2105 CARE OF
    pub const careof: Self = Self { inner: 0x0ab8 };

    /// U+2012 FIGURE DASH
    pub const figdash: Self = Self { inner: 0x0abb };

    /// (U+27E8 MATHEMATICAL LEFT ANGLE BRACKET)
    pub const leftanglebracket: Self = Self { inner: 0x0abc };

    /// (U+002E FULL STOP)
    pub const decimalpoint: Self = Self { inner: 0x0abd };

    /// (U+27E9 MATHEMATICAL RIGHT ANGLE BRACKET)
    pub const rightanglebracket: Self = Self { inner: 0x0abe };

    pub const marker: Self = Self { inner: 0x0abf };

    /// U+215B VULGAR FRACTION ONE EIGHTH
    pub const oneeighth: Self = Self { inner: 0x0ac3 };

    /// U+215C VULGAR FRACTION THREE EIGHTHS
    pub const threeeighths: Self = Self { inner: 0x0ac4 };

    /// U+215D VULGAR FRACTION FIVE EIGHTHS
    pub const fiveeighths: Self = Self { inner: 0x0ac5 };

    /// U+215E VULGAR FRACTION SEVEN EIGHTHS
    pub const seveneighths: Self = Self { inner: 0x0ac6 };

    /// U+2122 TRADE MARK SIGN
    pub const trademark: Self = Self { inner: 0x0ac9 };

    /// (U+2613 SALTIRE)
    pub const signaturemark: Self = Self { inner: 0x0aca };

    pub const trademarkincircle: Self = Self { inner: 0x0acb };

    /// (U+25C1 WHITE LEFT-POINTING TRIANGLE)
    pub const leftopentriangle: Self = Self { inner: 0x0acc };

    /// (U+25B7 WHITE RIGHT-POINTING TRIANGLE)
    pub const rightopentriangle: Self = Self { inner: 0x0acd };

    /// (U+25CB WHITE CIRCLE)
    pub const emopencircle: Self = Self { inner: 0x0ace };

    /// (U+25AF WHITE VERTICAL RECTANGLE)
    pub const emopenrectangle: Self = Self { inner: 0x0acf };

    /// U+2018 LEFT SINGLE QUOTATION MARK
    pub const leftsinglequotemark: Self = Self { inner: 0x0ad0 };

    /// U+2019 RIGHT SINGLE QUOTATION MARK
    pub const rightsinglequotemark: Self = Self { inner: 0x0ad1 };

    /// U+201C LEFT DOUBLE QUOTATION MARK
    pub const leftdoublequotemark: Self = Self { inner: 0x0ad2 };

    /// U+201D RIGHT DOUBLE QUOTATION MARK
    pub const rightdoublequotemark: Self = Self { inner: 0x0ad3 };

    /// U+211E PRESCRIPTION TAKE
    pub const prescription: Self = Self { inner: 0x0ad4 };

    /// U+2032 PRIME
    pub const minutes: Self = Self { inner: 0x0ad6 };

    /// U+2033 DOUBLE PRIME
    pub const seconds: Self = Self { inner: 0x0ad7 };

    /// U+271D LATIN CROSS
    pub const latincross: Self = Self { inner: 0x0ad9 };

    pub const hexagram: Self = Self { inner: 0x0ada };

    /// (U+25AC BLACK RECTANGLE)
    pub const filledrectbullet: Self = Self { inner: 0x0adb };

    /// (U+25C0 BLACK LEFT-POINTING TRIANGLE)
    pub const filledlefttribullet: Self = Self { inner: 0x0adc };

    /// (U+25B6 BLACK RIGHT-POINTING TRIANGLE)
    pub const filledrighttribullet: Self = Self { inner: 0x0add };

    /// (U+25CF BLACK CIRCLE)
    pub const emfilledcircle: Self = Self { inner: 0x0ade };

    /// (U+25AE BLACK VERTICAL RECTANGLE)
    pub const emfilledrect: Self = Self { inner: 0x0adf };

    /// (U+25E6 WHITE BULLET)
    pub const enopencircbullet: Self = Self { inner: 0x0ae0 };

    /// (U+25AB WHITE SMALL SQUARE)
    pub const enopensquarebullet: Self = Self { inner: 0x0ae1 };

    /// (U+25AD WHITE RECTANGLE)
    pub const openrectbullet: Self = Self { inner: 0x0ae2 };

    /// (U+25B3 WHITE UP-POINTING TRIANGLE)
    pub const opentribulletup: Self = Self { inner: 0x0ae3 };

    /// (U+25BD WHITE DOWN-POINTING TRIANGLE)
    pub const opentribulletdown: Self = Self { inner: 0x0ae4 };

    /// (U+2606 WHITE STAR)
    pub const openstar: Self = Self { inner: 0x0ae5 };

    /// (U+2022 BULLET)
    pub const enfilledcircbullet: Self = Self { inner: 0x0ae6 };

    /// (U+25AA BLACK SMALL SQUARE)
    pub const enfilledsqbullet: Self = Self { inner: 0x0ae7 };

    /// (U+25B2 BLACK UP-POINTING TRIANGLE)
    pub const filledtribulletup: Self = Self { inner: 0x0ae8 };

    /// (U+25BC BLACK DOWN-POINTING TRIANGLE)
    pub const filledtribulletdown: Self = Self { inner: 0x0ae9 };

    /// (U+261C WHITE LEFT POINTING INDEX)
    pub const leftpointer: Self = Self { inner: 0x0aea };

    /// (U+261E WHITE RIGHT POINTING INDEX)
    pub const rightpointer: Self = Self { inner: 0x0aeb };

    /// U+2663 BLACK CLUB SUIT
    pub const club: Self = Self { inner: 0x0aec };

    /// U+2666 BLACK DIAMOND SUIT
    pub const diamond: Self = Self { inner: 0x0aed };

    /// U+2665 BLACK HEART SUIT
    pub const heart: Self = Self { inner: 0x0aee };

    /// U+2720 MALTESE CROSS
    pub const maltesecross: Self = Self { inner: 0x0af0 };

    /// U+2020 DAGGER
    pub const dagger: Self = Self { inner: 0x0af1 };

    /// U+2021 DOUBLE DAGGER
    pub const doubledagger: Self = Self { inner: 0x0af2 };

    /// U+2713 CHECK MARK
    pub const checkmark: Self = Self { inner: 0x0af3 };

    /// U+2717 BALLOT X
    pub const ballotcross: Self = Self { inner: 0x0af4 };

    /// U+266F MUSIC SHARP SIGN
    pub const musicalsharp: Self = Self { inner: 0x0af5 };

    /// U+266D MUSIC FLAT SIGN
    pub const musicalflat: Self = Self { inner: 0x0af6 };

    /// U+2642 MALE SIGN
    pub const malesymbol: Self = Self { inner: 0x0af7 };

    /// U+2640 FEMALE SIGN
    pub const femalesymbol: Self = Self { inner: 0x0af8 };

    /// U+260E BLACK TELEPHONE
    pub const telephone: Self = Self { inner: 0x0af9 };

    /// U+2315 TELEPHONE RECORDER
    pub const telephonerecorder: Self = Self { inner: 0x0afa };

    /// U+2117 SOUND RECORDING COPYRIGHT
    pub const phonographcopyright: Self = Self { inner: 0x0afb };

    /// U+2038 CARET
    pub const caret: Self = Self { inner: 0x0afc };

    /// U+201A SINGLE LOW-9 QUOTATION MARK
    pub const singlelowquotemark: Self = Self { inner: 0x0afd };

    /// U+201E DOUBLE LOW-9 QUOTATION MARK
    pub const doublelowquotemark: Self = Self { inner: 0x0afe };

    pub const cursor: Self = Self { inner: 0x0aff };

    /// (U+003C LESS-THAN SIGN)
    pub const leftcaret: Self = Self { inner: 0x0ba3 };

    /// (U+003E GREATER-THAN SIGN)
    pub const rightcaret: Self = Self { inner: 0x0ba6 };

    /// (U+2228 LOGICAL OR)
    pub const downcaret: Self = Self { inner: 0x0ba8 };

    /// (U+2227 LOGICAL AND)
    pub const upcaret: Self = Self { inner: 0x0ba9 };

    /// (U+00AF MACRON)
    pub const overbar: Self = Self { inner: 0x0bc0 };

    /// U+22A5 UP TACK
    pub const downtack: Self = Self { inner: 0x0bc2 };

    /// (U+2229 INTERSECTION)
    pub const upshoe: Self = Self { inner: 0x0bc3 };

    /// U+230A LEFT FLOOR
    pub const downstile: Self = Self { inner: 0x0bc4 };

    /// (U+005F LOW LINE)
    pub const underbar: Self = Self { inner: 0x0bc6 };

    /// U+2218 RING OPERATOR
    pub const jot: Self = Self { inner: 0x0bca };

    /// U+2395 APL FUNCTIONAL SYMBOL QUAD
    pub const quad: Self = Self { inner: 0x0bcc };

    /// U+22A4 DOWN TACK
    pub const uptack: Self = Self { inner: 0x0bce };

    /// U+25CB WHITE CIRCLE
    pub const circle: Self = Self { inner: 0x0bcf };

    /// U+2308 LEFT CEILING
    pub const upstile: Self = Self { inner: 0x0bd3 };

    /// (U+222A UNION)
    pub const downshoe: Self = Self { inner: 0x0bd6 };

    /// (U+2283 SUPERSET OF)
    pub const rightshoe: Self = Self { inner: 0x0bd8 };

    /// (U+2282 SUBSET OF)
    pub const leftshoe: Self = Self { inner: 0x0bda };

    /// U+22A2 RIGHT TACK
    pub const lefttack: Self = Self { inner: 0x0bdc };

    /// U+22A3 LEFT TACK
    pub const righttack: Self = Self { inner: 0x0bfc };

    /// U+2017 DOUBLE LOW LINE
    pub const hebrew_doublelowline: Self = Self { inner: 0x0cdf };

    /// U+05D0 HEBREW LETTER ALEF
    pub const hebrew_aleph: Self = Self { inner: 0x0ce0 };

    /// U+05D1 HEBREW LETTER BET
    pub const hebrew_bet: Self = Self { inner: 0x0ce1 };

    /// deprecated
    pub const hebrew_beth: Self = Self { inner: 0x0ce1 };

    /// U+05D2 HEBREW LETTER GIMEL
    pub const hebrew_gimel: Self = Self { inner: 0x0ce2 };

    /// deprecated
    pub const hebrew_gimmel: Self = Self { inner: 0x0ce2 };

    /// U+05D3 HEBREW LETTER DALET
    pub const hebrew_dalet: Self = Self { inner: 0x0ce3 };

    /// deprecated
    pub const hebrew_daleth: Self = Self { inner: 0x0ce3 };

    /// U+05D4 HEBREW LETTER HE
    pub const hebrew_he: Self = Self { inner: 0x0ce4 };

    /// U+05D5 HEBREW LETTER VAV
    pub const hebrew_waw: Self = Self { inner: 0x0ce5 };

    /// U+05D6 HEBREW LETTER ZAYIN
    pub const hebrew_zain: Self = Self { inner: 0x0ce6 };

    /// deprecated
    pub const hebrew_zayin: Self = Self { inner: 0x0ce6 };

    /// U+05D7 HEBREW LETTER HET
    pub const hebrew_chet: Self = Self { inner: 0x0ce7 };

    /// deprecated
    pub const hebrew_het: Self = Self { inner: 0x0ce7 };

    /// U+05D8 HEBREW LETTER TET
    pub const hebrew_tet: Self = Self { inner: 0x0ce8 };

    /// deprecated
    pub const hebrew_teth: Self = Self { inner: 0x0ce8 };

    /// U+05D9 HEBREW LETTER YOD
    pub const hebrew_yod: Self = Self { inner: 0x0ce9 };

    /// U+05DA HEBREW LETTER FINAL KAF
    pub const hebrew_finalkaph: Self = Self { inner: 0x0cea };

    /// U+05DB HEBREW LETTER KAF
    pub const hebrew_kaph: Self = Self { inner: 0x0ceb };

    /// U+05DC HEBREW LETTER LAMED
    pub const hebrew_lamed: Self = Self { inner: 0x0cec };

    /// U+05DD HEBREW LETTER FINAL MEM
    pub const hebrew_finalmem: Self = Self { inner: 0x0ced };

    /// U+05DE HEBREW LETTER MEM
    pub const hebrew_mem: Self = Self { inner: 0x0cee };

    /// U+05DF HEBREW LETTER FINAL NUN
    pub const hebrew_finalnun: Self = Self { inner: 0x0cef };

    /// U+05E0 HEBREW LETTER NUN
    pub const hebrew_nun: Self = Self { inner: 0x0cf0 };

    /// U+05E1 HEBREW LETTER SAMEKH
    pub const hebrew_samech: Self = Self { inner: 0x0cf1 };

    /// deprecated
    pub const hebrew_samekh: Self = Self { inner: 0x0cf1 };

    /// U+05E2 HEBREW LETTER AYIN
    pub const hebrew_ayin: Self = Self { inner: 0x0cf2 };

    /// U+05E3 HEBREW LETTER FINAL PE
    pub const hebrew_finalpe: Self = Self { inner: 0x0cf3 };

    /// U+05E4 HEBREW LETTER PE
    pub const hebrew_pe: Self = Self { inner: 0x0cf4 };

    /// U+05E5 HEBREW LETTER FINAL TSADI
    pub const hebrew_finalzade: Self = Self { inner: 0x0cf5 };

    /// deprecated
    pub const hebrew_finalzadi: Self = Self { inner: 0x0cf5 };

    /// U+05E6 HEBREW LETTER TSADI
    pub const hebrew_zade: Self = Self { inner: 0x0cf6 };

    /// deprecated
    pub const hebrew_zadi: Self = Self { inner: 0x0cf6 };

    /// U+05E7 HEBREW LETTER QOF
    pub const hebrew_qoph: Self = Self { inner: 0x0cf7 };

    /// deprecated
    pub const hebrew_kuf: Self = Self { inner: 0x0cf7 };

    /// U+05E8 HEBREW LETTER RESH
    pub const hebrew_resh: Self = Self { inner: 0x0cf8 };

    /// U+05E9 HEBREW LETTER SHIN
    pub const hebrew_shin: Self = Self { inner: 0x0cf9 };

    /// U+05EA HEBREW LETTER TAV
    pub const hebrew_taw: Self = Self { inner: 0x0cfa };

    /// deprecated
    pub const hebrew_taf: Self = Self { inner: 0x0cfa };

    /// Alias for mode_switch
    pub const Hebrew_switch: Self = Self { inner: 0xff7e };

    /// U+0E01 THAI CHARACTER KO KAI
    pub const Thai_kokai: Self = Self { inner: 0x0da1 };

    /// U+0E02 THAI CHARACTER KHO KHAI
    pub const Thai_khokhai: Self = Self { inner: 0x0da2 };

    /// U+0E03 THAI CHARACTER KHO KHUAT
    pub const Thai_khokhuat: Self = Self { inner: 0x0da3 };

    /// U+0E04 THAI CHARACTER KHO KHWAI
    pub const Thai_khokhwai: Self = Self { inner: 0x0da4 };

    /// U+0E05 THAI CHARACTER KHO KHON
    pub const Thai_khokhon: Self = Self { inner: 0x0da5 };

    /// U+0E06 THAI CHARACTER KHO RAKHANG
    pub const Thai_khorakhang: Self = Self { inner: 0x0da6 };

    /// U+0E07 THAI CHARACTER NGO NGU
    pub const Thai_ngongu: Self = Self { inner: 0x0da7 };

    /// U+0E08 THAI CHARACTER CHO CHAN
    pub const Thai_chochan: Self = Self { inner: 0x0da8 };

    /// U+0E09 THAI CHARACTER CHO CHING
    pub const Thai_choching: Self = Self { inner: 0x0da9 };

    /// U+0E0A THAI CHARACTER CHO CHANG
    pub const Thai_chochang: Self = Self { inner: 0x0daa };

    /// U+0E0B THAI CHARACTER SO SO
    pub const Thai_soso: Self = Self { inner: 0x0dab };

    /// U+0E0C THAI CHARACTER CHO CHOE
    pub const Thai_chochoe: Self = Self { inner: 0x0dac };

    /// U+0E0D THAI CHARACTER YO YING
    pub const Thai_yoying: Self = Self { inner: 0x0dad };

    /// U+0E0E THAI CHARACTER DO CHADA
    pub const Thai_dochada: Self = Self { inner: 0x0dae };

    /// U+0E0F THAI CHARACTER TO PATAK
    pub const Thai_topatak: Self = Self { inner: 0x0daf };

    /// U+0E10 THAI CHARACTER THO THAN
    pub const Thai_thothan: Self = Self { inner: 0x0db0 };

    /// U+0E11 THAI CHARACTER THO NANGMONTHO
    pub const Thai_thonangmontho: Self = Self { inner: 0x0db1 };

    /// U+0E12 THAI CHARACTER THO PHUTHAO
    pub const Thai_thophuthao: Self = Self { inner: 0x0db2 };

    /// U+0E13 THAI CHARACTER NO NEN
    pub const Thai_nonen: Self = Self { inner: 0x0db3 };

    /// U+0E14 THAI CHARACTER DO DEK
    pub const Thai_dodek: Self = Self { inner: 0x0db4 };

    /// U+0E15 THAI CHARACTER TO TAO
    pub const Thai_totao: Self = Self { inner: 0x0db5 };

    /// U+0E16 THAI CHARACTER THO THUNG
    pub const Thai_thothung: Self = Self { inner: 0x0db6 };

    /// U+0E17 THAI CHARACTER THO THAHAN
    pub const Thai_thothahan: Self = Self { inner: 0x0db7 };

    /// U+0E18 THAI CHARACTER THO THONG
    pub const Thai_thothong: Self = Self { inner: 0x0db8 };

    /// U+0E19 THAI CHARACTER NO NU
    pub const Thai_nonu: Self = Self { inner: 0x0db9 };

    /// U+0E1A THAI CHARACTER BO BAIMAI
    pub const Thai_bobaimai: Self = Self { inner: 0x0dba };

    /// U+0E1B THAI CHARACTER PO PLA
    pub const Thai_popla: Self = Self { inner: 0x0dbb };

    /// U+0E1C THAI CHARACTER PHO PHUNG
    pub const Thai_phophung: Self = Self { inner: 0x0dbc };

    /// U+0E1D THAI CHARACTER FO FA
    pub const Thai_fofa: Self = Self { inner: 0x0dbd };

    /// U+0E1E THAI CHARACTER PHO PHAN
    pub const Thai_phophan: Self = Self { inner: 0x0dbe };

    /// U+0E1F THAI CHARACTER FO FAN
    pub const Thai_fofan: Self = Self { inner: 0x0dbf };

    /// U+0E20 THAI CHARACTER PHO SAMPHAO
    pub const Thai_phosamphao: Self = Self { inner: 0x0dc0 };

    /// U+0E21 THAI CHARACTER MO MA
    pub const Thai_moma: Self = Self { inner: 0x0dc1 };

    /// U+0E22 THAI CHARACTER YO YAK
    pub const Thai_yoyak: Self = Self { inner: 0x0dc2 };

    /// U+0E23 THAI CHARACTER RO RUA
    pub const Thai_rorua: Self = Self { inner: 0x0dc3 };

    /// U+0E24 THAI CHARACTER RU
    pub const Thai_ru: Self = Self { inner: 0x0dc4 };

    /// U+0E25 THAI CHARACTER LO LING
    pub const Thai_loling: Self = Self { inner: 0x0dc5 };

    /// U+0E26 THAI CHARACTER LU
    pub const Thai_lu: Self = Self { inner: 0x0dc6 };

    /// U+0E27 THAI CHARACTER WO WAEN
    pub const Thai_wowaen: Self = Self { inner: 0x0dc7 };

    /// U+0E28 THAI CHARACTER SO SALA
    pub const Thai_sosala: Self = Self { inner: 0x0dc8 };

    /// U+0E29 THAI CHARACTER SO RUSI
    pub const Thai_sorusi: Self = Self { inner: 0x0dc9 };

    /// U+0E2A THAI CHARACTER SO SUA
    pub const Thai_sosua: Self = Self { inner: 0x0dca };

    /// U+0E2B THAI CHARACTER HO HIP
    pub const Thai_hohip: Self = Self { inner: 0x0dcb };

    /// U+0E2C THAI CHARACTER LO CHULA
    pub const Thai_lochula: Self = Self { inner: 0x0dcc };

    /// U+0E2D THAI CHARACTER O ANG
    pub const Thai_oang: Self = Self { inner: 0x0dcd };

    /// U+0E2E THAI CHARACTER HO NOKHUK
    pub const Thai_honokhuk: Self = Self { inner: 0x0dce };

    /// U+0E2F THAI CHARACTER PAIYANNOI
    pub const Thai_paiyannoi: Self = Self { inner: 0x0dcf };

    /// U+0E30 THAI CHARACTER SARA A
    pub const Thai_saraa: Self = Self { inner: 0x0dd0 };

    /// U+0E31 THAI CHARACTER MAI HAN-AKAT
    pub const Thai_maihanakat: Self = Self { inner: 0x0dd1 };

    /// U+0E32 THAI CHARACTER SARA AA
    pub const Thai_saraaa: Self = Self { inner: 0x0dd2 };

    /// U+0E33 THAI CHARACTER SARA AM
    pub const Thai_saraam: Self = Self { inner: 0x0dd3 };

    /// U+0E34 THAI CHARACTER SARA I
    pub const Thai_sarai: Self = Self { inner: 0x0dd4 };

    /// U+0E35 THAI CHARACTER SARA II
    pub const Thai_saraii: Self = Self { inner: 0x0dd5 };

    /// U+0E36 THAI CHARACTER SARA UE
    pub const Thai_saraue: Self = Self { inner: 0x0dd6 };

    /// U+0E37 THAI CHARACTER SARA UEE
    pub const Thai_sarauee: Self = Self { inner: 0x0dd7 };

    /// U+0E38 THAI CHARACTER SARA U
    pub const Thai_sarau: Self = Self { inner: 0x0dd8 };

    /// U+0E39 THAI CHARACTER SARA UU
    pub const Thai_sarauu: Self = Self { inner: 0x0dd9 };

    /// U+0E3A THAI CHARACTER PHINTHU
    pub const Thai_phinthu: Self = Self { inner: 0x0dda };

    pub const Thai_maihanakat_maitho: Self = Self { inner: 0x0dde };

    /// U+0E3F THAI CURRENCY SYMBOL BAHT
    pub const Thai_baht: Self = Self { inner: 0x0ddf };

    /// U+0E40 THAI CHARACTER SARA E
    pub const Thai_sarae: Self = Self { inner: 0x0de0 };

    /// U+0E41 THAI CHARACTER SARA AE
    pub const Thai_saraae: Self = Self { inner: 0x0de1 };

    /// U+0E42 THAI CHARACTER SARA O
    pub const Thai_sarao: Self = Self { inner: 0x0de2 };

    /// U+0E43 THAI CHARACTER SARA AI MAIMUAN
    pub const Thai_saraaimaimuan: Self = Self { inner: 0x0de3 };

    /// U+0E44 THAI CHARACTER SARA AI MAIMALAI
    pub const Thai_saraaimaimalai: Self = Self { inner: 0x0de4 };

    /// U+0E45 THAI CHARACTER LAKKHANGYAO
    pub const Thai_lakkhangyao: Self = Self { inner: 0x0de5 };

    /// U+0E46 THAI CHARACTER MAIYAMOK
    pub const Thai_maiyamok: Self = Self { inner: 0x0de6 };

    /// U+0E47 THAI CHARACTER MAITAIKHU
    pub const Thai_maitaikhu: Self = Self { inner: 0x0de7 };

    /// U+0E48 THAI CHARACTER MAI EK
    pub const Thai_maiek: Self = Self { inner: 0x0de8 };

    /// U+0E49 THAI CHARACTER MAI THO
    pub const Thai_maitho: Self = Self { inner: 0x0de9 };

    /// U+0E4A THAI CHARACTER MAI TRI
    pub const Thai_maitri: Self = Self { inner: 0x0dea };

    /// U+0E4B THAI CHARACTER MAI CHATTAWA
    pub const Thai_maichattawa: Self = Self { inner: 0x0deb };

    /// U+0E4C THAI CHARACTER THANTHAKHAT
    pub const Thai_thanthakhat: Self = Self { inner: 0x0dec };

    /// U+0E4D THAI CHARACTER NIKHAHIT
    pub const Thai_nikhahit: Self = Self { inner: 0x0ded };

    /// U+0E50 THAI DIGIT ZERO
    pub const Thai_leksun: Self = Self { inner: 0x0df0 };

    /// U+0E51 THAI DIGIT ONE
    pub const Thai_leknung: Self = Self { inner: 0x0df1 };

    /// U+0E52 THAI DIGIT TWO
    pub const Thai_leksong: Self = Self { inner: 0x0df2 };

    /// U+0E53 THAI DIGIT THREE
    pub const Thai_leksam: Self = Self { inner: 0x0df3 };

    /// U+0E54 THAI DIGIT FOUR
    pub const Thai_leksi: Self = Self { inner: 0x0df4 };

    /// U+0E55 THAI DIGIT FIVE
    pub const Thai_lekha: Self = Self { inner: 0x0df5 };

    /// U+0E56 THAI DIGIT SIX
    pub const Thai_lekhok: Self = Self { inner: 0x0df6 };

    /// U+0E57 THAI DIGIT SEVEN
    pub const Thai_lekchet: Self = Self { inner: 0x0df7 };

    /// U+0E58 THAI DIGIT EIGHT
    pub const Thai_lekpaet: Self = Self { inner: 0x0df8 };

    /// U+0E59 THAI DIGIT NINE
    pub const Thai_lekkao: Self = Self { inner: 0x0df9 };

    /// Hangul start/stop(toggle)
    pub const Hangul: Self = Self { inner: 0xff31 };

    /// Hangul start
    pub const Hangul_Start: Self = Self { inner: 0xff32 };

    /// Hangul end, English start
    pub const Hangul_End: Self = Self { inner: 0xff33 };

    /// Start Hangul->Hanja Conversion
    pub const Hangul_Hanja: Self = Self { inner: 0xff34 };

    /// Hangul Jamo mode
    pub const Hangul_Jamo: Self = Self { inner: 0xff35 };

    /// Hangul Romaja mode
    pub const Hangul_Romaja: Self = Self { inner: 0xff36 };

    /// Hangul code input mode
    pub const Hangul_Codeinput: Self = Self { inner: 0xff37 };

    /// Jeonja mode
    pub const Hangul_Jeonja: Self = Self { inner: 0xff38 };

    /// Banja mode
    pub const Hangul_Banja: Self = Self { inner: 0xff39 };

    /// Pre Hanja conversion
    pub const Hangul_PreHanja: Self = Self { inner: 0xff3a };

    /// Post Hanja conversion
    pub const Hangul_PostHanja: Self = Self { inner: 0xff3b };

    /// Single candidate
    pub const Hangul_SingleCandidate: Self = Self { inner: 0xff3c };

    /// Multiple candidate
    pub const Hangul_MultipleCandidate: Self = Self { inner: 0xff3d };

    /// Previous candidate
    pub const Hangul_PreviousCandidate: Self = Self { inner: 0xff3e };

    /// Special symbols
    pub const Hangul_Special: Self = Self { inner: 0xff3f };

    /// Alias for mode_switch
    pub const Hangul_switch: Self = Self { inner: 0xff7e };

    pub const Hangul_Kiyeog: Self = Self { inner: 0x0ea1 };

    pub const Hangul_SsangKiyeog: Self = Self { inner: 0x0ea2 };

    pub const Hangul_KiyeogSios: Self = Self { inner: 0x0ea3 };

    pub const Hangul_Nieun: Self = Self { inner: 0x0ea4 };

    pub const Hangul_NieunJieuj: Self = Self { inner: 0x0ea5 };

    pub const Hangul_NieunHieuh: Self = Self { inner: 0x0ea6 };

    pub const Hangul_Dikeud: Self = Self { inner: 0x0ea7 };

    pub const Hangul_SsangDikeud: Self = Self { inner: 0x0ea8 };

    pub const Hangul_Rieul: Self = Self { inner: 0x0ea9 };

    pub const Hangul_RieulKiyeog: Self = Self { inner: 0x0eaa };

    pub const Hangul_RieulMieum: Self = Self { inner: 0x0eab };

    pub const Hangul_RieulPieub: Self = Self { inner: 0x0eac };

    pub const Hangul_RieulSios: Self = Self { inner: 0x0ead };

    pub const Hangul_RieulTieut: Self = Self { inner: 0x0eae };

    pub const Hangul_RieulPhieuf: Self = Self { inner: 0x0eaf };

    pub const Hangul_RieulHieuh: Self = Self { inner: 0x0eb0 };

    pub const Hangul_Mieum: Self = Self { inner: 0x0eb1 };

    pub const Hangul_Pieub: Self = Self { inner: 0x0eb2 };

    pub const Hangul_SsangPieub: Self = Self { inner: 0x0eb3 };

    pub const Hangul_PieubSios: Self = Self { inner: 0x0eb4 };

    pub const Hangul_Sios: Self = Self { inner: 0x0eb5 };

    pub const Hangul_SsangSios: Self = Self { inner: 0x0eb6 };

    pub const Hangul_Ieung: Self = Self { inner: 0x0eb7 };

    pub const Hangul_Jieuj: Self = Self { inner: 0x0eb8 };

    pub const Hangul_SsangJieuj: Self = Self { inner: 0x0eb9 };

    pub const Hangul_Cieuc: Self = Self { inner: 0x0eba };

    pub const Hangul_Khieuq: Self = Self { inner: 0x0ebb };

    pub const Hangul_Tieut: Self = Self { inner: 0x0ebc };

    pub const Hangul_Phieuf: Self = Self { inner: 0x0ebd };

    pub const Hangul_Hieuh: Self = Self { inner: 0x0ebe };

    pub const Hangul_A: Self = Self { inner: 0x0ebf };

    pub const Hangul_AE: Self = Self { inner: 0x0ec0 };

    pub const Hangul_YA: Self = Self { inner: 0x0ec1 };

    pub const Hangul_YAE: Self = Self { inner: 0x0ec2 };

    pub const Hangul_EO: Self = Self { inner: 0x0ec3 };

    pub const Hangul_E: Self = Self { inner: 0x0ec4 };

    pub const Hangul_YEO: Self = Self { inner: 0x0ec5 };

    pub const Hangul_YE: Self = Self { inner: 0x0ec6 };

    pub const Hangul_O: Self = Self { inner: 0x0ec7 };

    pub const Hangul_WA: Self = Self { inner: 0x0ec8 };

    pub const Hangul_WAE: Self = Self { inner: 0x0ec9 };

    pub const Hangul_OE: Self = Self { inner: 0x0eca };

    pub const Hangul_YO: Self = Self { inner: 0x0ecb };

    pub const Hangul_U: Self = Self { inner: 0x0ecc };

    pub const Hangul_WEO: Self = Self { inner: 0x0ecd };

    pub const Hangul_WE: Self = Self { inner: 0x0ece };

    pub const Hangul_WI: Self = Self { inner: 0x0ecf };

    pub const Hangul_YU: Self = Self { inner: 0x0ed0 };

    pub const Hangul_EU: Self = Self { inner: 0x0ed1 };

    pub const Hangul_YI: Self = Self { inner: 0x0ed2 };

    pub const Hangul_I: Self = Self { inner: 0x0ed3 };

    pub const Hangul_J_Kiyeog: Self = Self { inner: 0x0ed4 };

    pub const Hangul_J_SsangKiyeog: Self = Self { inner: 0x0ed5 };

    pub const Hangul_J_KiyeogSios: Self = Self { inner: 0x0ed6 };

    pub const Hangul_J_Nieun: Self = Self { inner: 0x0ed7 };

    pub const Hangul_J_NieunJieuj: Self = Self { inner: 0x0ed8 };

    pub const Hangul_J_NieunHieuh: Self = Self { inner: 0x0ed9 };

    pub const Hangul_J_Dikeud: Self = Self { inner: 0x0eda };

    pub const Hangul_J_Rieul: Self = Self { inner: 0x0edb };

    pub const Hangul_J_RieulKiyeog: Self = Self { inner: 0x0edc };

    pub const Hangul_J_RieulMieum: Self = Self { inner: 0x0edd };

    pub const Hangul_J_RieulPieub: Self = Self { inner: 0x0ede };

    pub const Hangul_J_RieulSios: Self = Self { inner: 0x0edf };

    pub const Hangul_J_RieulTieut: Self = Self { inner: 0x0ee0 };

    pub const Hangul_J_RieulPhieuf: Self = Self { inner: 0x0ee1 };

    pub const Hangul_J_RieulHieuh: Self = Self { inner: 0x0ee2 };

    pub const Hangul_J_Mieum: Self = Self { inner: 0x0ee3 };

    pub const Hangul_J_Pieub: Self = Self { inner: 0x0ee4 };

    pub const Hangul_J_PieubSios: Self = Self { inner: 0x0ee5 };

    pub const Hangul_J_Sios: Self = Self { inner: 0x0ee6 };

    pub const Hangul_J_SsangSios: Self = Self { inner: 0x0ee7 };

    pub const Hangul_J_Ieung: Self = Self { inner: 0x0ee8 };

    pub const Hangul_J_Jieuj: Self = Self { inner: 0x0ee9 };

    pub const Hangul_J_Cieuc: Self = Self { inner: 0x0eea };

    pub const Hangul_J_Khieuq: Self = Self { inner: 0x0eeb };

    pub const Hangul_J_Tieut: Self = Self { inner: 0x0eec };

    pub const Hangul_J_Phieuf: Self = Self { inner: 0x0eed };

    pub const Hangul_J_Hieuh: Self = Self { inner: 0x0eee };

    pub const Hangul_RieulYeorinHieuh: Self = Self { inner: 0x0eef };

    pub const Hangul_SunkyeongeumMieum: Self = Self { inner: 0x0ef0 };

    pub const Hangul_SunkyeongeumPieub: Self = Self { inner: 0x0ef1 };

    pub const Hangul_PanSios: Self = Self { inner: 0x0ef2 };

    pub const Hangul_KkogjiDalrinIeung: Self = Self { inner: 0x0ef3 };

    pub const Hangul_SunkyeongeumPhieuf: Self = Self { inner: 0x0ef4 };

    pub const Hangul_YeorinHieuh: Self = Self { inner: 0x0ef5 };

    pub const Hangul_AraeA: Self = Self { inner: 0x0ef6 };

    pub const Hangul_AraeAE: Self = Self { inner: 0x0ef7 };

    pub const Hangul_J_PanSios: Self = Self { inner: 0x0ef8 };

    pub const Hangul_J_KkogjiDalrinIeung: Self = Self { inner: 0x0ef9 };

    pub const Hangul_J_YeorinHieuh: Self = Self { inner: 0x0efa };

    /// (U+20A9 WON SIGN)
    pub const Korean_Won: Self = Self { inner: 0x0eff };

    /// U+0587 ARMENIAN SMALL LIGATURE ECH YIWN
    pub const Armenian_ligature_ew: Self = Self { inner: 0x1000587 };

    /// U+0589 ARMENIAN FULL STOP
    pub const Armenian_full_stop: Self = Self { inner: 0x1000589 };

    /// U+0589 ARMENIAN FULL STOP
    pub const Armenian_verjaket: Self = Self { inner: 0x1000589 };

    /// U+055D ARMENIAN COMMA
    pub const Armenian_separation_mark: Self = Self { inner: 0x100055d };

    /// U+055D ARMENIAN COMMA
    pub const Armenian_but: Self = Self { inner: 0x100055d };

    /// U+058A ARMENIAN HYPHEN
    pub const Armenian_hyphen: Self = Self { inner: 0x100058a };

    /// U+058A ARMENIAN HYPHEN
    pub const Armenian_yentamna: Self = Self { inner: 0x100058a };

    /// U+055C ARMENIAN EXCLAMATION MARK
    pub const Armenian_exclam: Self = Self { inner: 0x100055c };

    /// U+055C ARMENIAN EXCLAMATION MARK
    pub const Armenian_amanak: Self = Self { inner: 0x100055c };

    /// U+055B ARMENIAN EMPHASIS MARK
    pub const Armenian_accent: Self = Self { inner: 0x100055b };

    /// U+055B ARMENIAN EMPHASIS MARK
    pub const Armenian_shesht: Self = Self { inner: 0x100055b };

    /// U+055E ARMENIAN QUESTION MARK
    pub const Armenian_question: Self = Self { inner: 0x100055e };

    /// U+055E ARMENIAN QUESTION MARK
    pub const Armenian_paruyk: Self = Self { inner: 0x100055e };

    /// U+0531 ARMENIAN CAPITAL LETTER AYB
    pub const Armenian_AYB: Self = Self { inner: 0x1000531 };

    /// U+0561 ARMENIAN SMALL LETTER AYB
    pub const Armenian_ayb: Self = Self { inner: 0x1000561 };

    /// U+0532 ARMENIAN CAPITAL LETTER BEN
    pub const Armenian_BEN: Self = Self { inner: 0x1000532 };

    /// U+0562 ARMENIAN SMALL LETTER BEN
    pub const Armenian_ben: Self = Self { inner: 0x1000562 };

    /// U+0533 ARMENIAN CAPITAL LETTER GIM
    pub const Armenian_GIM: Self = Self { inner: 0x1000533 };

    /// U+0563 ARMENIAN SMALL LETTER GIM
    pub const Armenian_gim: Self = Self { inner: 0x1000563 };

    /// U+0534 ARMENIAN CAPITAL LETTER DA
    pub const Armenian_DA: Self = Self { inner: 0x1000534 };

    /// U+0564 ARMENIAN SMALL LETTER DA
    pub const Armenian_da: Self = Self { inner: 0x1000564 };

    /// U+0535 ARMENIAN CAPITAL LETTER ECH
    pub const Armenian_YECH: Self = Self { inner: 0x1000535 };

    /// U+0565 ARMENIAN SMALL LETTER ECH
    pub const Armenian_yech: Self = Self { inner: 0x1000565 };

    /// U+0536 ARMENIAN CAPITAL LETTER ZA
    pub const Armenian_ZA: Self = Self { inner: 0x1000536 };

    /// U+0566 ARMENIAN SMALL LETTER ZA
    pub const Armenian_za: Self = Self { inner: 0x1000566 };

    /// U+0537 ARMENIAN CAPITAL LETTER EH
    pub const Armenian_E: Self = Self { inner: 0x1000537 };

    /// U+0567 ARMENIAN SMALL LETTER EH
    pub const Armenian_e: Self = Self { inner: 0x1000567 };

    /// U+0538 ARMENIAN CAPITAL LETTER ET
    pub const Armenian_AT: Self = Self { inner: 0x1000538 };

    /// U+0568 ARMENIAN SMALL LETTER ET
    pub const Armenian_at: Self = Self { inner: 0x1000568 };

    /// U+0539 ARMENIAN CAPITAL LETTER TO
    pub const Armenian_TO: Self = Self { inner: 0x1000539 };

    /// U+0569 ARMENIAN SMALL LETTER TO
    pub const Armenian_to: Self = Self { inner: 0x1000569 };

    /// U+053A ARMENIAN CAPITAL LETTER ZHE
    pub const Armenian_ZHE: Self = Self { inner: 0x100053a };

    /// U+056A ARMENIAN SMALL LETTER ZHE
    pub const Armenian_zhe: Self = Self { inner: 0x100056a };

    /// U+053B ARMENIAN CAPITAL LETTER INI
    pub const Armenian_INI: Self = Self { inner: 0x100053b };

    /// U+056B ARMENIAN SMALL LETTER INI
    pub const Armenian_ini: Self = Self { inner: 0x100056b };

    /// U+053C ARMENIAN CAPITAL LETTER LIWN
    pub const Armenian_LYUN: Self = Self { inner: 0x100053c };

    /// U+056C ARMENIAN SMALL LETTER LIWN
    pub const Armenian_lyun: Self = Self { inner: 0x100056c };

    /// U+053D ARMENIAN CAPITAL LETTER XEH
    pub const Armenian_KHE: Self = Self { inner: 0x100053d };

    /// U+056D ARMENIAN SMALL LETTER XEH
    pub const Armenian_khe: Self = Self { inner: 0x100056d };

    /// U+053E ARMENIAN CAPITAL LETTER CA
    pub const Armenian_TSA: Self = Self { inner: 0x100053e };

    /// U+056E ARMENIAN SMALL LETTER CA
    pub const Armenian_tsa: Self = Self { inner: 0x100056e };

    /// U+053F ARMENIAN CAPITAL LETTER KEN
    pub const Armenian_KEN: Self = Self { inner: 0x100053f };

    /// U+056F ARMENIAN SMALL LETTER KEN
    pub const Armenian_ken: Self = Self { inner: 0x100056f };

    /// U+0540 ARMENIAN CAPITAL LETTER HO
    pub const Armenian_HO: Self = Self { inner: 0x1000540 };

    /// U+0570 ARMENIAN SMALL LETTER HO
    pub const Armenian_ho: Self = Self { inner: 0x1000570 };

    /// U+0541 ARMENIAN CAPITAL LETTER JA
    pub const Armenian_DZA: Self = Self { inner: 0x1000541 };

    /// U+0571 ARMENIAN SMALL LETTER JA
    pub const Armenian_dza: Self = Self { inner: 0x1000571 };

    /// U+0542 ARMENIAN CAPITAL LETTER GHAD
    pub const Armenian_GHAT: Self = Self { inner: 0x1000542 };

    /// U+0572 ARMENIAN SMALL LETTER GHAD
    pub const Armenian_ghat: Self = Self { inner: 0x1000572 };

    /// U+0543 ARMENIAN CAPITAL LETTER CHEH
    pub const Armenian_TCHE: Self = Self { inner: 0x1000543 };

    /// U+0573 ARMENIAN SMALL LETTER CHEH
    pub const Armenian_tche: Self = Self { inner: 0x1000573 };

    /// U+0544 ARMENIAN CAPITAL LETTER MEN
    pub const Armenian_MEN: Self = Self { inner: 0x1000544 };

    /// U+0574 ARMENIAN SMALL LETTER MEN
    pub const Armenian_men: Self = Self { inner: 0x1000574 };

    /// U+0545 ARMENIAN CAPITAL LETTER YI
    pub const Armenian_HI: Self = Self { inner: 0x1000545 };

    /// U+0575 ARMENIAN SMALL LETTER YI
    pub const Armenian_hi: Self = Self { inner: 0x1000575 };

    /// U+0546 ARMENIAN CAPITAL LETTER NOW
    pub const Armenian_NU: Self = Self { inner: 0x1000546 };

    /// U+0576 ARMENIAN SMALL LETTER NOW
    pub const Armenian_nu: Self = Self { inner: 0x1000576 };

    /// U+0547 ARMENIAN CAPITAL LETTER SHA
    pub const Armenian_SHA: Self = Self { inner: 0x1000547 };

    /// U+0577 ARMENIAN SMALL LETTER SHA
    pub const Armenian_sha: Self = Self { inner: 0x1000577 };

    /// U+0548 ARMENIAN CAPITAL LETTER VO
    pub const Armenian_VO: Self = Self { inner: 0x1000548 };

    /// U+0578 ARMENIAN SMALL LETTER VO
    pub const Armenian_vo: Self = Self { inner: 0x1000578 };

    /// U+0549 ARMENIAN CAPITAL LETTER CHA
    pub const Armenian_CHA: Self = Self { inner: 0x1000549 };

    /// U+0579 ARMENIAN SMALL LETTER CHA
    pub const Armenian_cha: Self = Self { inner: 0x1000579 };

    /// U+054A ARMENIAN CAPITAL LETTER PEH
    pub const Armenian_PE: Self = Self { inner: 0x100054a };

    /// U+057A ARMENIAN SMALL LETTER PEH
    pub const Armenian_pe: Self = Self { inner: 0x100057a };

    /// U+054B ARMENIAN CAPITAL LETTER JHEH
    pub const Armenian_JE: Self = Self { inner: 0x100054b };

    /// U+057B ARMENIAN SMALL LETTER JHEH
    pub const Armenian_je: Self = Self { inner: 0x100057b };

    /// U+054C ARMENIAN CAPITAL LETTER RA
    pub const Armenian_RA: Self = Self { inner: 0x100054c };

    /// U+057C ARMENIAN SMALL LETTER RA
    pub const Armenian_ra: Self = Self { inner: 0x100057c };

    /// U+054D ARMENIAN CAPITAL LETTER SEH
    pub const Armenian_SE: Self = Self { inner: 0x100054d };

    /// U+057D ARMENIAN SMALL LETTER SEH
    pub const Armenian_se: Self = Self { inner: 0x100057d };

    /// U+054E ARMENIAN CAPITAL LETTER VEW
    pub const Armenian_VEV: Self = Self { inner: 0x100054e };

    /// U+057E ARMENIAN SMALL LETTER VEW
    pub const Armenian_vev: Self = Self { inner: 0x100057e };

    /// U+054F ARMENIAN CAPITAL LETTER TIWN
    pub const Armenian_TYUN: Self = Self { inner: 0x100054f };

    /// U+057F ARMENIAN SMALL LETTER TIWN
    pub const Armenian_tyun: Self = Self { inner: 0x100057f };

    /// U+0550 ARMENIAN CAPITAL LETTER REH
    pub const Armenian_RE: Self = Self { inner: 0x1000550 };

    /// U+0580 ARMENIAN SMALL LETTER REH
    pub const Armenian_re: Self = Self { inner: 0x1000580 };

    /// U+0551 ARMENIAN CAPITAL LETTER CO
    pub const Armenian_TSO: Self = Self { inner: 0x1000551 };

    /// U+0581 ARMENIAN SMALL LETTER CO
    pub const Armenian_tso: Self = Self { inner: 0x1000581 };

    /// U+0552 ARMENIAN CAPITAL LETTER YIWN
    pub const Armenian_VYUN: Self = Self { inner: 0x1000552 };

    /// U+0582 ARMENIAN SMALL LETTER YIWN
    pub const Armenian_vyun: Self = Self { inner: 0x1000582 };

    /// U+0553 ARMENIAN CAPITAL LETTER PIWR
    pub const Armenian_PYUR: Self = Self { inner: 0x1000553 };

    /// U+0583 ARMENIAN SMALL LETTER PIWR
    pub const Armenian_pyur: Self = Self { inner: 0x1000583 };

    /// U+0554 ARMENIAN CAPITAL LETTER KEH
    pub const Armenian_KE: Self = Self { inner: 0x1000554 };

    /// U+0584 ARMENIAN SMALL LETTER KEH
    pub const Armenian_ke: Self = Self { inner: 0x1000584 };

    /// U+0555 ARMENIAN CAPITAL LETTER OH
    pub const Armenian_O: Self = Self { inner: 0x1000555 };

    /// U+0585 ARMENIAN SMALL LETTER OH
    pub const Armenian_o: Self = Self { inner: 0x1000585 };

    /// U+0556 ARMENIAN CAPITAL LETTER FEH
    pub const Armenian_FE: Self = Self { inner: 0x1000556 };

    /// U+0586 ARMENIAN SMALL LETTER FEH
    pub const Armenian_fe: Self = Self { inner: 0x1000586 };

    /// U+055A ARMENIAN APOSTROPHE
    pub const Armenian_apostrophe: Self = Self { inner: 0x100055a };

    /// U+10D0 GEORGIAN LETTER AN
    pub const Georgian_an: Self = Self { inner: 0x10010d0 };

    /// U+10D1 GEORGIAN LETTER BAN
    pub const Georgian_ban: Self = Self { inner: 0x10010d1 };

    /// U+10D2 GEORGIAN LETTER GAN
    pub const Georgian_gan: Self = Self { inner: 0x10010d2 };

    /// U+10D3 GEORGIAN LETTER DON
    pub const Georgian_don: Self = Self { inner: 0x10010d3 };

    /// U+10D4 GEORGIAN LETTER EN
    pub const Georgian_en: Self = Self { inner: 0x10010d4 };

    /// U+10D5 GEORGIAN LETTER VIN
    pub const Georgian_vin: Self = Self { inner: 0x10010d5 };

    /// U+10D6 GEORGIAN LETTER ZEN
    pub const Georgian_zen: Self = Self { inner: 0x10010d6 };

    /// U+10D7 GEORGIAN LETTER TAN
    pub const Georgian_tan: Self = Self { inner: 0x10010d7 };

    /// U+10D8 GEORGIAN LETTER IN
    pub const Georgian_in: Self = Self { inner: 0x10010d8 };

    /// U+10D9 GEORGIAN LETTER KAN
    pub const Georgian_kan: Self = Self { inner: 0x10010d9 };

    /// U+10DA GEORGIAN LETTER LAS
    pub const Georgian_las: Self = Self { inner: 0x10010da };

    /// U+10DB GEORGIAN LETTER MAN
    pub const Georgian_man: Self = Self { inner: 0x10010db };

    /// U+10DC GEORGIAN LETTER NAR
    pub const Georgian_nar: Self = Self { inner: 0x10010dc };

    /// U+10DD GEORGIAN LETTER ON
    pub const Georgian_on: Self = Self { inner: 0x10010dd };

    /// U+10DE GEORGIAN LETTER PAR
    pub const Georgian_par: Self = Self { inner: 0x10010de };

    /// U+10DF GEORGIAN LETTER ZHAR
    pub const Georgian_zhar: Self = Self { inner: 0x10010df };

    /// U+10E0 GEORGIAN LETTER RAE
    pub const Georgian_rae: Self = Self { inner: 0x10010e0 };

    /// U+10E1 GEORGIAN LETTER SAN
    pub const Georgian_san: Self = Self { inner: 0x10010e1 };

    /// U+10E2 GEORGIAN LETTER TAR
    pub const Georgian_tar: Self = Self { inner: 0x10010e2 };

    /// U+10E3 GEORGIAN LETTER UN
    pub const Georgian_un: Self = Self { inner: 0x10010e3 };

    /// U+10E4 GEORGIAN LETTER PHAR
    pub const Georgian_phar: Self = Self { inner: 0x10010e4 };

    /// U+10E5 GEORGIAN LETTER KHAR
    pub const Georgian_khar: Self = Self { inner: 0x10010e5 };

    /// U+10E6 GEORGIAN LETTER GHAN
    pub const Georgian_ghan: Self = Self { inner: 0x10010e6 };

    /// U+10E7 GEORGIAN LETTER QAR
    pub const Georgian_qar: Self = Self { inner: 0x10010e7 };

    /// U+10E8 GEORGIAN LETTER SHIN
    pub const Georgian_shin: Self = Self { inner: 0x10010e8 };

    /// U+10E9 GEORGIAN LETTER CHIN
    pub const Georgian_chin: Self = Self { inner: 0x10010e9 };

    /// U+10EA GEORGIAN LETTER CAN
    pub const Georgian_can: Self = Self { inner: 0x10010ea };

    /// U+10EB GEORGIAN LETTER JIL
    pub const Georgian_jil: Self = Self { inner: 0x10010eb };

    /// U+10EC GEORGIAN LETTER CIL
    pub const Georgian_cil: Self = Self { inner: 0x10010ec };

    /// U+10ED GEORGIAN LETTER CHAR
    pub const Georgian_char: Self = Self { inner: 0x10010ed };

    /// U+10EE GEORGIAN LETTER XAN
    pub const Georgian_xan: Self = Self { inner: 0x10010ee };

    /// U+10EF GEORGIAN LETTER JHAN
    pub const Georgian_jhan: Self = Self { inner: 0x10010ef };

    /// U+10F0 GEORGIAN LETTER HAE
    pub const Georgian_hae: Self = Self { inner: 0x10010f0 };

    /// U+10F1 GEORGIAN LETTER HE
    pub const Georgian_he: Self = Self { inner: 0x10010f1 };

    /// U+10F2 GEORGIAN LETTER HIE
    pub const Georgian_hie: Self = Self { inner: 0x10010f2 };

    /// U+10F3 GEORGIAN LETTER WE
    pub const Georgian_we: Self = Self { inner: 0x10010f3 };

    /// U+10F4 GEORGIAN LETTER HAR
    pub const Georgian_har: Self = Self { inner: 0x10010f4 };

    /// U+10F5 GEORGIAN LETTER HOE
    pub const Georgian_hoe: Self = Self { inner: 0x10010f5 };

    /// U+10F6 GEORGIAN LETTER FI
    pub const Georgian_fi: Self = Self { inner: 0x10010f6 };

    /// U+1E8A LATIN CAPITAL LETTER X WITH DOT ABOVE
    pub const Xabovedot: Self = Self { inner: 0x1001e8a };

    /// U+012C LATIN CAPITAL LETTER I WITH BREVE
    pub const Ibreve: Self = Self { inner: 0x100012c };

    /// U+01B5 LATIN CAPITAL LETTER Z WITH STROKE
    pub const Zstroke: Self = Self { inner: 0x10001b5 };

    /// U+01E6 LATIN CAPITAL LETTER G WITH CARON
    pub const Gcaron: Self = Self { inner: 0x10001e6 };

    /// U+01D2 LATIN CAPITAL LETTER O WITH CARON
    pub const Ocaron: Self = Self { inner: 0x10001d1 };

    /// U+019F LATIN CAPITAL LETTER O WITH MIDDLE TILDE
    pub const Obarred: Self = Self { inner: 0x100019f };

    /// U+1E8B LATIN SMALL LETTER X WITH DOT ABOVE
    pub const xabovedot: Self = Self { inner: 0x1001e8b };

    /// U+012D LATIN SMALL LETTER I WITH BREVE
    pub const ibreve: Self = Self { inner: 0x100012d };

    /// U+01B6 LATIN SMALL LETTER Z WITH STROKE
    pub const zstroke: Self = Self { inner: 0x10001b6 };

    /// U+01E7 LATIN SMALL LETTER G WITH CARON
    pub const gcaron: Self = Self { inner: 0x10001e7 };

    /// U+01D2 LATIN SMALL LETTER O WITH CARON
    pub const ocaron: Self = Self { inner: 0x10001d2 };

    /// U+0275 LATIN SMALL LETTER BARRED O
    pub const obarred: Self = Self { inner: 0x1000275 };

    /// U+018F LATIN CAPITAL LETTER SCHWA
    pub const SCHWA: Self = Self { inner: 0x100018f };

    /// U+0259 LATIN SMALL LETTER SCHWA
    pub const schwa: Self = Self { inner: 0x1000259 };

    /// U+1E36 LATIN CAPITAL LETTER L WITH DOT BELOW
    pub const Lbelowdot: Self = Self { inner: 0x1001e36 };

    /// U+1E37 LATIN SMALL LETTER L WITH DOT BELOW
    pub const lbelowdot: Self = Self { inner: 0x1001e37 };

    /// U+1EA0 LATIN CAPITAL LETTER A WITH DOT BELOW
    pub const Abelowdot: Self = Self { inner: 0x1001ea0 };

    /// U+1EA1 LATIN SMALL LETTER A WITH DOT BELOW
    pub const abelowdot: Self = Self { inner: 0x1001ea1 };

    /// U+1EA2 LATIN CAPITAL LETTER A WITH HOOK ABOVE
    pub const Ahook: Self = Self { inner: 0x1001ea2 };

    /// U+1EA3 LATIN SMALL LETTER A WITH HOOK ABOVE
    pub const ahook: Self = Self { inner: 0x1001ea3 };

    /// U+1EA4 LATIN CAPITAL LETTER A WITH CIRCUMFLEX AND ACUTE
    pub const Acircumflexacute: Self = Self { inner: 0x1001ea4 };

    /// U+1EA5 LATIN SMALL LETTER A WITH CIRCUMFLEX AND ACUTE
    pub const acircumflexacute: Self = Self { inner: 0x1001ea5 };

    /// U+1EA6 LATIN CAPITAL LETTER A WITH CIRCUMFLEX AND GRAVE
    pub const Acircumflexgrave: Self = Self { inner: 0x1001ea6 };

    /// U+1EA7 LATIN SMALL LETTER A WITH CIRCUMFLEX AND GRAVE
    pub const acircumflexgrave: Self = Self { inner: 0x1001ea7 };

    /// U+1EA8 LATIN CAPITAL LETTER A WITH CIRCUMFLEX AND HOOK ABOVE
    pub const Acircumflexhook: Self = Self { inner: 0x1001ea8 };

    /// U+1EA9 LATIN SMALL LETTER A WITH CIRCUMFLEX AND HOOK ABOVE
    pub const acircumflexhook: Self = Self { inner: 0x1001ea9 };

    /// U+1EAA LATIN CAPITAL LETTER A WITH CIRCUMFLEX AND TILDE
    pub const Acircumflextilde: Self = Self { inner: 0x1001eaa };

    /// U+1EAB LATIN SMALL LETTER A WITH CIRCUMFLEX AND TILDE
    pub const acircumflextilde: Self = Self { inner: 0x1001eab };

    /// U+1EAC LATIN CAPITAL LETTER A WITH CIRCUMFLEX AND DOT BELOW
    pub const Acircumflexbelowdot: Self = Self { inner: 0x1001eac };

    /// U+1EAD LATIN SMALL LETTER A WITH CIRCUMFLEX AND DOT BELOW
    pub const acircumflexbelowdot: Self = Self { inner: 0x1001ead };

    /// U+1EAE LATIN CAPITAL LETTER A WITH BREVE AND ACUTE
    pub const Abreveacute: Self = Self { inner: 0x1001eae };

    /// U+1EAF LATIN SMALL LETTER A WITH BREVE AND ACUTE
    pub const abreveacute: Self = Self { inner: 0x1001eaf };

    /// U+1EB0 LATIN CAPITAL LETTER A WITH BREVE AND GRAVE
    pub const Abrevegrave: Self = Self { inner: 0x1001eb0 };

    /// U+1EB1 LATIN SMALL LETTER A WITH BREVE AND GRAVE
    pub const abrevegrave: Self = Self { inner: 0x1001eb1 };

    /// U+1EB2 LATIN CAPITAL LETTER A WITH BREVE AND HOOK ABOVE
    pub const Abrevehook: Self = Self { inner: 0x1001eb2 };

    /// U+1EB3 LATIN SMALL LETTER A WITH BREVE AND HOOK ABOVE
    pub const abrevehook: Self = Self { inner: 0x1001eb3 };

    /// U+1EB4 LATIN CAPITAL LETTER A WITH BREVE AND TILDE
    pub const Abrevetilde: Self = Self { inner: 0x1001eb4 };

    /// U+1EB5 LATIN SMALL LETTER A WITH BREVE AND TILDE
    pub const abrevetilde: Self = Self { inner: 0x1001eb5 };

    /// U+1EB6 LATIN CAPITAL LETTER A WITH BREVE AND DOT BELOW
    pub const Abrevebelowdot: Self = Self { inner: 0x1001eb6 };

    /// U+1EB7 LATIN SMALL LETTER A WITH BREVE AND DOT BELOW
    pub const abrevebelowdot: Self = Self { inner: 0x1001eb7 };

    /// U+1EB8 LATIN CAPITAL LETTER E WITH DOT BELOW
    pub const Ebelowdot: Self = Self { inner: 0x1001eb8 };

    /// U+1EB9 LATIN SMALL LETTER E WITH DOT BELOW
    pub const ebelowdot: Self = Self { inner: 0x1001eb9 };

    /// U+1EBA LATIN CAPITAL LETTER E WITH HOOK ABOVE
    pub const Ehook: Self = Self { inner: 0x1001eba };

    /// U+1EBB LATIN SMALL LETTER E WITH HOOK ABOVE
    pub const ehook: Self = Self { inner: 0x1001ebb };

    /// U+1EBC LATIN CAPITAL LETTER E WITH TILDE
    pub const Etilde: Self = Self { inner: 0x1001ebc };

    /// U+1EBD LATIN SMALL LETTER E WITH TILDE
    pub const etilde: Self = Self { inner: 0x1001ebd };

    /// U+1EBE LATIN CAPITAL LETTER E WITH CIRCUMFLEX AND ACUTE
    pub const Ecircumflexacute: Self = Self { inner: 0x1001ebe };

    /// U+1EBF LATIN SMALL LETTER E WITH CIRCUMFLEX AND ACUTE
    pub const ecircumflexacute: Self = Self { inner: 0x1001ebf };

    /// U+1EC0 LATIN CAPITAL LETTER E WITH CIRCUMFLEX AND GRAVE
    pub const Ecircumflexgrave: Self = Self { inner: 0x1001ec0 };

    /// U+1EC1 LATIN SMALL LETTER E WITH CIRCUMFLEX AND GRAVE
    pub const ecircumflexgrave: Self = Self { inner: 0x1001ec1 };

    /// U+1EC2 LATIN CAPITAL LETTER E WITH CIRCUMFLEX AND HOOK ABOVE
    pub const Ecircumflexhook: Self = Self { inner: 0x1001ec2 };

    /// U+1EC3 LATIN SMALL LETTER E WITH CIRCUMFLEX AND HOOK ABOVE
    pub const ecircumflexhook: Self = Self { inner: 0x1001ec3 };

    /// U+1EC4 LATIN CAPITAL LETTER E WITH CIRCUMFLEX AND TILDE
    pub const Ecircumflextilde: Self = Self { inner: 0x1001ec4 };

    /// U+1EC5 LATIN SMALL LETTER E WITH CIRCUMFLEX AND TILDE
    pub const ecircumflextilde: Self = Self { inner: 0x1001ec5 };

    /// U+1EC6 LATIN CAPITAL LETTER E WITH CIRCUMFLEX AND DOT BELOW
    pub const Ecircumflexbelowdot: Self = Self { inner: 0x1001ec6 };

    /// U+1EC7 LATIN SMALL LETTER E WITH CIRCUMFLEX AND DOT BELOW
    pub const ecircumflexbelowdot: Self = Self { inner: 0x1001ec7 };

    /// U+1EC8 LATIN CAPITAL LETTER I WITH HOOK ABOVE
    pub const Ihook: Self = Self { inner: 0x1001ec8 };

    /// U+1EC9 LATIN SMALL LETTER I WITH HOOK ABOVE
    pub const ihook: Self = Self { inner: 0x1001ec9 };

    /// U+1ECA LATIN CAPITAL LETTER I WITH DOT BELOW
    pub const Ibelowdot: Self = Self { inner: 0x1001eca };

    /// U+1ECB LATIN SMALL LETTER I WITH DOT BELOW
    pub const ibelowdot: Self = Self { inner: 0x1001ecb };

    /// U+1ECC LATIN CAPITAL LETTER O WITH DOT BELOW
    pub const Obelowdot: Self = Self { inner: 0x1001ecc };

    /// U+1ECD LATIN SMALL LETTER O WITH DOT BELOW
    pub const obelowdot: Self = Self { inner: 0x1001ecd };

    /// U+1ECE LATIN CAPITAL LETTER O WITH HOOK ABOVE
    pub const Ohook: Self = Self { inner: 0x1001ece };

    /// U+1ECF LATIN SMALL LETTER O WITH HOOK ABOVE
    pub const ohook: Self = Self { inner: 0x1001ecf };

    /// U+1ED0 LATIN CAPITAL LETTER O WITH CIRCUMFLEX AND ACUTE
    pub const Ocircumflexacute: Self = Self { inner: 0x1001ed0 };

    /// U+1ED1 LATIN SMALL LETTER O WITH CIRCUMFLEX AND ACUTE
    pub const ocircumflexacute: Self = Self { inner: 0x1001ed1 };

    /// U+1ED2 LATIN CAPITAL LETTER O WITH CIRCUMFLEX AND GRAVE
    pub const Ocircumflexgrave: Self = Self { inner: 0x1001ed2 };

    /// U+1ED3 LATIN SMALL LETTER O WITH CIRCUMFLEX AND GRAVE
    pub const ocircumflexgrave: Self = Self { inner: 0x1001ed3 };

    /// U+1ED4 LATIN CAPITAL LETTER O WITH CIRCUMFLEX AND HOOK ABOVE
    pub const Ocircumflexhook: Self = Self { inner: 0x1001ed4 };

    /// U+1ED5 LATIN SMALL LETTER O WITH CIRCUMFLEX AND HOOK ABOVE
    pub const ocircumflexhook: Self = Self { inner: 0x1001ed5 };

    /// U+1ED6 LATIN CAPITAL LETTER O WITH CIRCUMFLEX AND TILDE
    pub const Ocircumflextilde: Self = Self { inner: 0x1001ed6 };

    /// U+1ED7 LATIN SMALL LETTER O WITH CIRCUMFLEX AND TILDE
    pub const ocircumflextilde: Self = Self { inner: 0x1001ed7 };

    /// U+1ED8 LATIN CAPITAL LETTER O WITH CIRCUMFLEX AND DOT BELOW
    pub const Ocircumflexbelowdot: Self = Self { inner: 0x1001ed8 };

    /// U+1ED9 LATIN SMALL LETTER O WITH CIRCUMFLEX AND DOT BELOW
    pub const ocircumflexbelowdot: Self = Self { inner: 0x1001ed9 };

    /// U+1EDA LATIN CAPITAL LETTER O WITH HORN AND ACUTE
    pub const Ohornacute: Self = Self { inner: 0x1001eda };

    /// U+1EDB LATIN SMALL LETTER O WITH HORN AND ACUTE
    pub const ohornacute: Self = Self { inner: 0x1001edb };

    /// U+1EDC LATIN CAPITAL LETTER O WITH HORN AND GRAVE
    pub const Ohorngrave: Self = Self { inner: 0x1001edc };

    /// U+1EDD LATIN SMALL LETTER O WITH HORN AND GRAVE
    pub const ohorngrave: Self = Self { inner: 0x1001edd };

    /// U+1EDE LATIN CAPITAL LETTER O WITH HORN AND HOOK ABOVE
    pub const Ohornhook: Self = Self { inner: 0x1001ede };

    /// U+1EDF LATIN SMALL LETTER O WITH HORN AND HOOK ABOVE
    pub const ohornhook: Self = Self { inner: 0x1001edf };

    /// U+1EE0 LATIN CAPITAL LETTER O WITH HORN AND TILDE
    pub const Ohorntilde: Self = Self { inner: 0x1001ee0 };

    /// U+1EE1 LATIN SMALL LETTER O WITH HORN AND TILDE
    pub const ohorntilde: Self = Self { inner: 0x1001ee1 };

    /// U+1EE2 LATIN CAPITAL LETTER O WITH HORN AND DOT BELOW
    pub const Ohornbelowdot: Self = Self { inner: 0x1001ee2 };

    /// U+1EE3 LATIN SMALL LETTER O WITH HORN AND DOT BELOW
    pub const ohornbelowdot: Self = Self { inner: 0x1001ee3 };

    /// U+1EE4 LATIN CAPITAL LETTER U WITH DOT BELOW
    pub const Ubelowdot: Self = Self { inner: 0x1001ee4 };

    /// U+1EE5 LATIN SMALL LETTER U WITH DOT BELOW
    pub const ubelowdot: Self = Self { inner: 0x1001ee5 };

    /// U+1EE6 LATIN CAPITAL LETTER U WITH HOOK ABOVE
    pub const Uhook: Self = Self { inner: 0x1001ee6 };

    /// U+1EE7 LATIN SMALL LETTER U WITH HOOK ABOVE
    pub const uhook: Self = Self { inner: 0x1001ee7 };

    /// U+1EE8 LATIN CAPITAL LETTER U WITH HORN AND ACUTE
    pub const Uhornacute: Self = Self { inner: 0x1001ee8 };

    /// U+1EE9 LATIN SMALL LETTER U WITH HORN AND ACUTE
    pub const uhornacute: Self = Self { inner: 0x1001ee9 };

    /// U+1EEA LATIN CAPITAL LETTER U WITH HORN AND GRAVE
    pub const Uhorngrave: Self = Self { inner: 0x1001eea };

    /// U+1EEB LATIN SMALL LETTER U WITH HORN AND GRAVE
    pub const uhorngrave: Self = Self { inner: 0x1001eeb };

    /// U+1EEC LATIN CAPITAL LETTER U WITH HORN AND HOOK ABOVE
    pub const Uhornhook: Self = Self { inner: 0x1001eec };

    /// U+1EED LATIN SMALL LETTER U WITH HORN AND HOOK ABOVE
    pub const uhornhook: Self = Self { inner: 0x1001eed };

    /// U+1EEE LATIN CAPITAL LETTER U WITH HORN AND TILDE
    pub const Uhorntilde: Self = Self { inner: 0x1001eee };

    /// U+1EEF LATIN SMALL LETTER U WITH HORN AND TILDE
    pub const uhorntilde: Self = Self { inner: 0x1001eef };

    /// U+1EF0 LATIN CAPITAL LETTER U WITH HORN AND DOT BELOW
    pub const Uhornbelowdot: Self = Self { inner: 0x1001ef0 };

    /// U+1EF1 LATIN SMALL LETTER U WITH HORN AND DOT BELOW
    pub const uhornbelowdot: Self = Self { inner: 0x1001ef1 };

    /// U+1EF4 LATIN CAPITAL LETTER Y WITH DOT BELOW
    pub const Ybelowdot: Self = Self { inner: 0x1001ef4 };

    /// U+1EF5 LATIN SMALL LETTER Y WITH DOT BELOW
    pub const ybelowdot: Self = Self { inner: 0x1001ef5 };

    /// U+1EF6 LATIN CAPITAL LETTER Y WITH HOOK ABOVE
    pub const Yhook: Self = Self { inner: 0x1001ef6 };

    /// U+1EF7 LATIN SMALL LETTER Y WITH HOOK ABOVE
    pub const yhook: Self = Self { inner: 0x1001ef7 };

    /// U+1EF8 LATIN CAPITAL LETTER Y WITH TILDE
    pub const Ytilde: Self = Self { inner: 0x1001ef8 };

    /// U+1EF9 LATIN SMALL LETTER Y WITH TILDE
    pub const ytilde: Self = Self { inner: 0x1001ef9 };

    /// U+01A0 LATIN CAPITAL LETTER O WITH HORN
    pub const Ohorn: Self = Self { inner: 0x10001a0 };

    /// U+01A1 LATIN SMALL LETTER O WITH HORN
    pub const ohorn: Self = Self { inner: 0x10001a1 };

    /// U+01AF LATIN CAPITAL LETTER U WITH HORN
    pub const Uhorn: Self = Self { inner: 0x10001af };

    /// U+01B0 LATIN SMALL LETTER U WITH HORN
    pub const uhorn: Self = Self { inner: 0x10001b0 };

    /// U+20A0 EURO-CURRENCY SIGN
    pub const EcuSign: Self = Self { inner: 0x10020a0 };

    /// U+20A1 COLON SIGN
    pub const ColonSign: Self = Self { inner: 0x10020a1 };

    /// U+20A2 CRUZEIRO SIGN
    pub const CruzeiroSign: Self = Self { inner: 0x10020a2 };

    /// U+20A3 FRENCH FRANC SIGN
    pub const FFrancSign: Self = Self { inner: 0x10020a3 };

    /// U+20A4 LIRA SIGN
    pub const LiraSign: Self = Self { inner: 0x10020a4 };

    /// U+20A5 MILL SIGN
    pub const MillSign: Self = Self { inner: 0x10020a5 };

    /// U+20A6 NAIRA SIGN
    pub const NairaSign: Self = Self { inner: 0x10020a6 };

    /// U+20A7 PESETA SIGN
    pub const PesetaSign: Self = Self { inner: 0x10020a7 };

    /// U+20A8 RUPEE SIGN
    pub const RupeeSign: Self = Self { inner: 0x10020a8 };

    /// U+20A9 WON SIGN
    pub const WonSign: Self = Self { inner: 0x10020a9 };

    /// U+20AA NEW SHEQEL SIGN
    pub const NewSheqelSign: Self = Self { inner: 0x10020aa };

    /// U+20AB DONG SIGN
    pub const DongSign: Self = Self { inner: 0x10020ab };

    /// U+20AC EURO SIGN
    pub const EuroSign: Self = Self { inner: 0x20ac };
}
