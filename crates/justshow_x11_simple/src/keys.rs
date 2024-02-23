use justshow_x11::{keysym::KeySym, replies::GetKeyboardMapping, requests::KeyCode};

#[derive(Debug, Clone)]
pub struct KeySymbols {
    pub(crate) min_keycode: u8,
    pub(crate) max_keycode: u8,
    pub(crate) reply: GetKeyboardMapping,
}

impl KeySymbols {
    fn get_keysym(&self, keycode: u8, mut col: usize) -> KeySym {
        let mut per = self.reply.keysyms_per_keycode;
        if (col >= per as usize && col > 3)
            || keycode < self.min_keycode
            || keycode > self.max_keycode
        {
            return KeySym::NO_SYMBOL;
        }

        let keysyms =
            &self.reply.keysyms[(keycode as usize - self.min_keycode as usize) * per as usize..];
        if col < 4 {
            if col > 1 {
                while per > 2 && keysyms[per as usize - 1] == KeySym::NO_SYMBOL {
                    per -= 1;
                }
                if per < 3 {
                    col -= 2;
                }
            }

            if per as usize <= (col | 1) || keysyms[col | 1] == KeySym::NO_SYMBOL {
                let ConvertedCase { lsym, usym } = convert_case(keysyms[col & !1]);
                if (col & 1) == 0 {
                    return lsym;
                } else if usym == lsym {
                    return KeySym::NO_SYMBOL;
                } else {
                    return usym;
                }
            }
        }

        keysyms[col]
    }

    pub fn get_keycodes(&self, keysym: KeySym) -> Vec<KeyCode> {
        let mut res = Vec::new();

        for i in self.min_keycode..=self.max_keycode {
            for j in 0..self.reply.keysyms_per_keycode {
                let ks = self.get_keysym(i, j as usize);
                if ks == keysym {
                    res.push(KeyCode::from(i));
                    break;
                }
            }
        }

        res
    }
}

struct ConvertedCase {
    lsym: KeySym,
    usym: KeySym,
}

fn convert_case(sym: KeySym) -> ConvertedCase {
    let mut lower = sym;
    let mut upper = sym;

    match sym.inner >> 8 {
        // Latin 1
        0 => {
            if sym >= KeySym::A && sym <= KeySym::Z {
                lower += KeySym::a - KeySym::A;
            } else if sym >= KeySym::a && sym <= KeySym::z {
                upper += KeySym::a - KeySym::A;
            } else if (sym >= KeySym::Agrave) && (sym <= KeySym::Odiaeresis) {
                lower += KeySym::agrave - KeySym::Agrave;
            } else if (sym >= KeySym::agrave) && (sym <= KeySym::odiaeresis) {
                upper -= KeySym::agrave - KeySym::Agrave;
            } else if (sym >= KeySym::Ooblique) && (sym <= KeySym::Thorn) {
                lower += KeySym::oslash - KeySym::Ooblique;
            } else if (sym >= KeySym::oslash) && (sym <= KeySym::thorn) {
                upper -= KeySym::oslash - KeySym::Ooblique;
            }
        }
        // Latin 2
        1 => {
            if sym == KeySym::Aogonek {
                lower = KeySym::aogonek;
            } else if sym >= KeySym::Lstroke && sym <= KeySym::Sacute {
                lower += KeySym::lstroke - KeySym::Lstroke;
            } else if sym >= KeySym::Scaron && sym <= KeySym::Zacute {
                lower += KeySym::scaron - KeySym::Scaron;
            } else if sym >= KeySym::Zcaron && sym <= KeySym::Zabovedot {
                lower += KeySym::zcaron - KeySym::Zcaron;
            } else if sym == KeySym::aogonek {
                upper = KeySym::Aogonek;
            } else if sym >= KeySym::lstroke && sym <= KeySym::sacute {
                upper -= KeySym::lstroke - KeySym::Lstroke;
            } else if sym >= KeySym::scaron && sym <= KeySym::zacute {
                upper -= KeySym::scaron - KeySym::Scaron;
            } else if sym >= KeySym::zcaron && sym <= KeySym::zabovedot {
                upper -= KeySym::zcaron - KeySym::Zcaron;
            } else if sym >= KeySym::Racute && sym <= KeySym::Tcedilla {
                lower += KeySym::racute - KeySym::Racute;
            } else if sym >= KeySym::racute && sym <= KeySym::tcedilla {
                upper -= KeySym::racute - KeySym::Racute;
            }
        }
        // Latin 3
        2 => {
            if sym >= KeySym::Hstroke && sym <= KeySym::Hcircumflex {
                lower += KeySym::hstroke - KeySym::Hstroke;
            } else if sym >= KeySym::Gbreve && sym <= KeySym::Jcircumflex {
                lower += KeySym::gbreve - KeySym::Gbreve;
            } else if sym >= KeySym::hstroke && sym <= KeySym::hcircumflex {
                upper -= KeySym::hstroke - KeySym::Hstroke;
            } else if sym >= KeySym::gbreve && sym <= KeySym::jcircumflex {
                upper -= KeySym::gbreve - KeySym::Gbreve;
            } else if sym >= KeySym::Cabovedot && sym <= KeySym::Scircumflex {
                lower += KeySym::cabovedot - KeySym::Cabovedot;
            } else if sym >= KeySym::cabovedot && sym <= KeySym::scircumflex {
                upper -= KeySym::cabovedot - KeySym::Cabovedot;
            }
        }
        // Latin 4
        3 => {
            if sym >= KeySym::Rcedilla && sym <= KeySym::Tslash {
                lower += KeySym::rcedilla - KeySym::Rcedilla;
            } else if sym >= KeySym::rcedilla && sym <= KeySym::tslash {
                upper -= KeySym::rcedilla - KeySym::Rcedilla;
            } else if sym == KeySym::ENG {
                lower = KeySym::eng;
            } else if sym == KeySym::eng {
                upper = KeySym::ENG;
            } else if sym >= KeySym::Amacron && sym <= KeySym::Umacron {
                lower += KeySym::amacron - KeySym::Amacron;
            } else if sym >= KeySym::amacron && sym <= KeySym::umacron {
                upper -= KeySym::amacron - KeySym::Amacron;
            }
        }
        // Cyrillic
        6 => {
            if sym >= KeySym::Serbian_DJE && sym <= KeySym::Serbian_DZE {
                lower -= KeySym::Serbian_DJE - KeySym::Serbian_dje;
            } else if sym >= KeySym::Serbian_dje && sym <= KeySym::Serbian_dze {
                upper += KeySym::Serbian_DJE - KeySym::Serbian_dje;
            } else if sym >= KeySym::Cyrillic_YU && sym <= KeySym::Cyrillic_HARDSIGN {
                lower -= KeySym::Cyrillic_YU - KeySym::Cyrillic_yu;
            } else if sym >= KeySym::Cyrillic_yu && sym <= KeySym::Cyrillic_hardsign {
                upper += KeySym::Cyrillic_YU - KeySym::Cyrillic_yu;
            }
        }
        // Greek
        7 => {
            if sym >= KeySym::Greek_ALPHAaccent && sym <= KeySym::Greek_OMEGAaccent {
                lower += KeySym::Greek_alphaaccent - KeySym::Greek_ALPHAaccent;
            } else if sym >= KeySym::Greek_alphaaccent
                && sym <= KeySym::Greek_omegaaccent
                && sym != KeySym::Greek_iotaaccentdieresis
                && sym != KeySym::Greek_upsilonaccentdieresis
            {
                upper -= KeySym::Greek_alphaaccent - KeySym::Greek_ALPHAaccent;
            } else if sym >= KeySym::Greek_ALPHA && sym <= KeySym::Greek_OMEGA {
                lower += KeySym::Greek_alpha - KeySym::Greek_ALPHA;
            } else if sym >= KeySym::Greek_alpha
                && sym <= KeySym::Greek_omega
                && sym != KeySym::Greek_finalsmallsigma
            {
                upper -= KeySym::Greek_alpha - KeySym::Greek_ALPHA;
            }
        }
        // Armenian
        20 => {
            if sym >= KeySym::Armenian_AYB && sym <= KeySym::Armenian_fe {
                lower.inner = sym.inner | 1;
                upper.inner = sym.inner & !1;
            }
        }
        _ => {}
    }

    ConvertedCase {
        lsym: lower,
        usym: upper,
    }
}
