use crate::{
    lexer::{Lexer, Spanned, StringValidity, Token},
    Encoding, Font, FontBoundingBox, Glyph, Number, ParserError, Property, PropertyValue, Size,
    Vector2,
};
use std::iter::Peekable;

pub struct Parser<'src, I>
where
    I: Iterator<Item = (usize, char)>,
{
    lexer: Peekable<Lexer<'src, I>>,
    font: Font,
}

impl<'src, I> Parser<'src, I>
where
    I: Iterator<Item = (usize, char)>,
{
    pub fn new(lexer: Lexer<'src, I>) -> Self {
        Self {
            lexer: lexer.peekable(),
            font: Font {
                version: Number::Float(0.0),
                content_version: None,
                font: "".to_string(),
                size: Size {
                    point_size: 0,
                    x_res: 0,
                    y_res: 0,
                },
                font_bounding_box: FontBoundingBox {
                    width: 0,
                    height: 0,
                    x_off: 0,
                    y_off: 0,
                },
                properties: Vec::new(),
                metric_set: 0,
                s_width: None,
                d_width: None,
                s_width1: None,
                d_width1: None,
                v_vector: None,
                glyphs: Vec::new(),
            },
        }
    }

    fn next_token(&mut self) -> Result<Spanned<Token<'src>>, ParserError> {
        self.lexer.next().ok_or(ParserError::UnexpectedEof)
    }

    fn peek_token(&mut self) -> Result<Spanned<Token<'src>>, ParserError> {
        self.lexer.peek().ok_or(ParserError::UnexpectedEof).copied()
    }

    fn keyword(&mut self, expected: &'static str) -> Result<(), ParserError> {
        let t = self.next_token()?;
        match t.value {
            Token::Keyword(got) if got == expected => Ok(()),
            _ => Err(ParserError::InvalidArgument(t.span)),
        }
    }

    fn any_keyword(&mut self) -> Result<&str, ParserError> {
        let t = self.next_token()?;
        match t.value {
            Token::Keyword(got) => Ok(got),
            _ => Err(ParserError::InvalidArgument(t.span)),
        }
    }

    fn integer(&mut self) -> Result<i32, ParserError> {
        let t = self.next_token()?;
        match t.value {
            Token::Integer(integer) => Ok(integer),
            _ => Err(ParserError::InvalidArgument(t.span)),
        }
    }

    fn number(&mut self) -> Result<Number, ParserError> {
        let t = self.next_token()?;
        match t.value {
            Token::Number(number) => Ok(Number::Float(number)),
            Token::Integer(integer) => Ok(Number::Integer(integer)),
            _ => Err(ParserError::InvalidArgument(t.span)),
        }
    }

    fn property_value(&mut self) -> Result<PropertyValue, ParserError> {
        let t = self.next_token()?;
        match t.value {
            Token::Number(number) => Ok(PropertyValue::Number(Number::Float(number))),
            Token::Integer(integer) => Ok(PropertyValue::Number(Number::Integer(integer))),
            Token::String(got, StringValidity::Valid) => Ok(PropertyValue::String(got.to_string())),
            Token::String(_, _) => Err(ParserError::UnclosedString(t.span)),
            _ => Err(ParserError::InvalidArgument(t.span)),
        }
    }

    fn glyph(&mut self) -> Result<Glyph, ParserError> {
        self.keyword("STARTCHAR")?;
        let name = self.any_keyword()?.to_string();

        let mut glyph = Glyph {
            name: name.clone(),
            encoding: Encoding::AdobeStandard(0),
            s_width: Vector2 {
                width: Number::Integer(0),
                height: Number::Integer(0),
            },
            d_width: Vector2 {
                width: 0,
                height: 0,
            },
            s_width1: Vector2 {
                width: Number::Integer(0),
                height: Number::Integer(0),
            },
            d_width1: Vector2 {
                width: 0,
                height: 0,
            },
            v_vector: None,
            bounding_box: FontBoundingBox {
                width: 0,
                height: 0,
                x_off: 0,
                y_off: 0,
            },
            bitmap: Vec::new(),
        };

        macro_rules! check_duplicate {
            ($var:ident, $name:literal) => {
                if $var {
                    return Err(ParserError::DuplicateGlyphProperty(name.clone(), $name));
                }
                $var = true;
            };
        }

        macro_rules! check_missing {
            ($var:ident, $name:literal) => {
                if !$var {
                    Err(ParserError::MissingGlyphProperty(name.clone(), $name))?
                }
            };
        }

        let mut encoding_set = false;
        let mut s_width_set = false;
        let mut d_width_set = false;
        let mut s_width1_set = false;
        let mut d_width1_set = false;
        let mut v_vector_set = false;
        let mut bbx_set = false;

        let metric_set = self.font.metric_set;
        loop {
            let kw = self.any_keyword()?;
            match kw {
                "ENCODING" => {
                    check_duplicate!(encoding_set, "ENCODING");

                    let i = self.integer()?;
                    if i < 0 {
                        let v = match self.peek_token() {
                            Ok(Spanned {
                                span: _,
                                value: Token::Integer(int),
                            }) => Some(int),
                            _ => None,
                        };

                        glyph.encoding = Encoding::NonStandard(v);
                    } else {
                        glyph.encoding = Encoding::AdobeStandard(i as u32);
                    }
                }
                "SWIDTH" => {
                    check_duplicate!(s_width_set, "SWIDTH");

                    glyph.s_width.width = self.number()?;
                    glyph.s_width.height = self.number()?;
                }
                "DWIDTH" => {
                    check_duplicate!(d_width_set, "DWIDTH");

                    glyph.d_width.width = self.integer()?;
                    glyph.d_width.height = self.integer()?;
                }
                "SWIDTH1" if metric_set != 0 => {
                    check_duplicate!(s_width1_set, "SWIDTH1");

                    glyph.s_width1.width = self.number()?;
                    glyph.s_width1.height = self.number()?;
                }
                "DWIDTH1" if metric_set != 0 => {
                    check_duplicate!(d_width1_set, "DWIDTH1");

                    glyph.d_width1.width = self.integer()?;
                    glyph.d_width1.height = self.integer()?;
                }
                "VVECTOR" => {
                    check_duplicate!(v_vector_set, "VVECTOR");

                    let v_vector = Vector2 {
                        width: self.integer()?,
                        height: self.integer()?,
                    };
                    glyph.v_vector = Some(v_vector);
                }
                "BBX" => {
                    check_duplicate!(bbx_set, "BBX");

                    let width = self.integer()? as u32;
                    let height = self.integer()? as u32;
                    let x_off = self.integer()?;
                    let y_off = self.integer()?;
                    glyph.bounding_box = FontBoundingBox {
                        width,
                        height,
                        x_off,
                        y_off,
                    };
                }
                "BITMAP" => {
                    check_missing!(encoding_set, "ENCODING");
                    check_missing!(s_width_set, "SWIDTH");
                    check_missing!(d_width_set, "DWIDTH");
                    if metric_set != 0 {
                        check_missing!(s_width1_set, "SWIDTH1");
                        check_missing!(d_width1_set, "DWIDTH1");
                    }
                    check_missing!(bbx_set, "BBX");

                    let w = (glyph.bounding_box.width + 7) / 8;

                    glyph.bitmap = Vec::with_capacity(w as usize);
                    for _ in 0..glyph.bounding_box.height {
                        let n = self.integer()?;
                        for &b in &n.to_le_bytes()[0..w as usize] {
                            glyph.bitmap.push(b);
                        }
                    }
                    self.keyword("ENDCHAR")?;

                    return Ok(glyph);
                }
                invalid => {
                    return Err(ParserError::InvalidGlyphProperty(
                        name.to_string(),
                        invalid.to_string(),
                    ));
                }
            }
        }
    }

    pub fn parse(mut self) -> Result<Font, ParserError> {
        self.keyword("STARTFONT")?;
        self.font.version = self.number()?;

        macro_rules! check_duplicate {
            ($var:ident, $name:literal) => {
                if $var {
                    return Err(ParserError::DuplicateGlobalProperty($name));
                }
                $var = true;
            };
        }

        macro_rules! check_missing {
            ($var:ident, $name:literal) => {
                if !$var {
                    return Err(ParserError::MissingGlobalProperty($name));
                }
            };
        }

        let mut font_set = false;
        let mut content_version_set = false;
        let mut size_set = false;
        let mut font_bounding_box_set = false;
        let mut metric_set_set = false;
        let mut properties_set = false;

        loop {
            let kw = self.any_keyword()?;
            match kw {
                "FONT" => {
                    check_duplicate!(font_set, "FONT");

                    self.font.font = self.any_keyword()?.to_string();
                }
                "CONTENTVERSION" => {
                    check_duplicate!(content_version_set, "CONTENTVERSION");

                    self.font.content_version = Some(self.integer()?);
                }
                "SIZE" => {
                    check_duplicate!(size_set, "SIZE");

                    self.font.size.point_size = self.integer()?;
                    self.font.size.x_res = self.integer()?;
                    self.font.size.y_res = self.integer()?;
                }
                "FONTBOUNDINGBOX" => {
                    check_duplicate!(font_bounding_box_set, "FONTBOUNDINGBOX");

                    self.font.font_bounding_box.width = self.integer()? as u32;
                    self.font.font_bounding_box.height = self.integer()? as u32;
                    self.font.font_bounding_box.x_off = self.integer()?;
                    self.font.font_bounding_box.y_off = self.integer()?;
                }
                "METRICSSET" => {
                    check_duplicate!(metric_set_set, "METRICSSET");

                    self.font.metric_set = self.integer()?;
                }
                "STARTPROPERTIES" => {
                    check_duplicate!(properties_set, "STARTPROPERTIES");

                    let n = self.integer()?;
                    self.font.properties = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        let name = self.any_keyword()?.to_string();
                        let value = self.property_value()?;
                        let property = Property { name, value };
                        self.font.properties.push(property);
                    }
                    self.keyword("ENDPROPERTIES")?;
                }
                "CHARS" => {
                    check_missing!(properties_set, "STARTPROPERTIES");
                    check_missing!(font_bounding_box_set, "FONTBOUNDINGBOX");
                    check_missing!(size_set, "SIZE");
                    check_missing!(font_set, "FONT");

                    let n = self.integer()?;
                    self.font.glyphs = Vec::with_capacity(n as usize);
                    for _ in 0..n {
                        let glyph = self.glyph()?;
                        self.font.glyphs.push(glyph);
                    }
                    self.keyword("ENDFONT")?;

                    return Ok(self.font);
                }
                invalid => return Err(ParserError::InvalidGlobalProperty(invalid.to_string())),
            }
        }
    }
}
