// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

mod lexer;
mod parser;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Location {
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParserError {
    DuplicateGlobalProperty(&'static str),
    InvalidGlobalProperty(String),
    MissingGlobalProperty(&'static str),

    DuplicateGlyphProperty(String, &'static str),
    InvalidGlyphProperty(String, String),
    MissingGlyphProperty(String, &'static str),

    InvalidArgument(Span),
    UnclosedString(Span),
    UnexpectedEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoding {
    AdobeStandard(u32),
    NonStandard(Option<i32>),
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Number {
    Float(f32),
    Integer(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Size {
    pub point_size: i32,
    pub x_res: i32,
    pub y_res: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontBoundingBox {
    pub width: u32,
    pub height: u32,
    pub x_off: i32,
    pub y_off: i32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Glyph {
    pub name: String,
    pub encoding: Encoding,
    pub s_width: Vector2<Number>,
    pub d_width: Vector2<i32>,
    pub s_width1: Vector2<Number>,
    pub d_width1: Vector2<i32>,
    pub v_vector: Option<Vector2<i32>>,
    pub bounding_box: FontBoundingBox,
    pub bitmap: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vector2<T> {
    pub width: T,
    pub height: T,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum PropertyValue {
    String(String),
    Number(Number),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Font {
    pub version: Number,
    pub content_version: Option<i32>,
    pub font: String,
    pub size: Size,
    pub font_bounding_box: FontBoundingBox,
    pub properties: Vec<Property>,
    pub metric_set: i32,
    pub s_width: Option<Vector2<Number>>,
    pub d_width: Option<Vector2<i32>>,
    pub s_width1: Option<Vector2<Number>>,
    pub d_width1: Option<Vector2<i32>>,
    pub v_vector: Option<Vector2<i32>>,
    pub glyphs: Vec<Glyph>,
}

pub fn parse(input: &str) -> Result<Font, ParserError> {
    let lexer = lexer::Lexer::new(input);
    let parser = parser::Parser::new(lexer);
    parser.parse()
}

#[test]
fn wikipedia_example() {
    // From https://en.wikipedia.org/wiki/Glyph_Bitmap_Distribution_Format#Example
    let unparsed_font = r#"
STARTFONT 2.1
FONT -gnu-unifont-medium-r-normal--16-160-75-75-c-80-iso10646-1
SIZE 16 75 75
FONTBOUNDINGBOX 16 16 0 -2
STARTPROPERTIES 2
FONT_ASCENT 14
FONT_DESCENT 2
ENDPROPERTIES
CHARS 1
STARTCHAR U+0041
ENCODING 65
SWIDTH 500 0
DWIDTH 8 0
BBX 8 16 0 -2
BITMAP
00
00
00
00
18
24
24
42
42
7E
42
42
42
42
00
00
ENDCHAR
ENDFONT
"#;

    let font = parse(unparsed_font).expect("Could not parse font file");
    assert_eq!(font.version, Number::Float(2.1));
    assert_eq!(font.glyphs.len(), 1);
}
