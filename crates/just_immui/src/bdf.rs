use std::collections::HashMap;

use just_bdf::{Font, Glyph};

pub(crate) struct BdfCharMap {
    glyphs: Vec<Glyph>,
    ascii: [usize; 128],
    map: HashMap<u32, usize>,
    default: usize,
}

impl BdfCharMap {
    pub fn ib8x8u() -> Self {
        let font = just_bdf::parse(include_str!("ib8x8u.bdf")).unwrap();
        Self::new(font)
    }

    pub fn new(font: Font) -> Self {
        let default = font.glyphs.len() - 1;
        let mut char_map = BdfCharMap {
            glyphs: font.glyphs,
            map: HashMap::new(),
            ascii: [default; 128],
            default,
        };

        for (idx, g) in char_map.glyphs.iter().enumerate() {
            if let just_bdf::Encoding::AdobeStandard(enc) = g.encoding {
                if enc < 128 {
                    char_map.ascii[enc as usize] = idx;
                }
                char_map.map.insert(enc, idx);
            }
        }
        char_map
    }

    pub fn get(&self, c: char) -> &Glyph {
        let k = c as u32;
        if k < 128 {
            &self.glyphs[self.ascii[k as usize]]
        } else {
            let idx = self.map.get(&k).unwrap_or(&self.default);
            &self.glyphs[*idx]
        }
    }
}
