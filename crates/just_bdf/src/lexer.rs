use crate::{Location, Number, Span};
use core::{iter::Peekable, str::CharIndices};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum StringValidity {
    Valid,
    Unclosed,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Token<'src> {
    Keyword(&'src str),
    String(&'src str, StringValidity),
    Integer(i32),
    Number(f32),
}

fn is_valid_ident_character(c: char) -> bool {
    !c.is_whitespace() && !['"'].contains(&c)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Located<T> {
    pub location: Location,
    pub value: T,
}

impl<T> From<(usize, T)> for Located<T> {
    fn from(value: (usize, T)) -> Self {
        Located {
            location: Location { offset: value.0 },
            value: value.1,
        }
    }
}

pub struct Lexer<'src, I>
where
    I: Iterator,
{
    iter: Peekable<I>,
    source: &'src str,
    bitmap_mode: bool,
}

impl<'src> Lexer<'src, CharIndices<'src>> {
    #[inline]
    pub fn new(source: &'src str) -> Self {
        Self::with_iterator(source.char_indices().peekable(), source)
    }
}

impl<'src, I> Lexer<'src, I>
where
    I: Iterator<Item = (usize, char)>,
{
    #[inline]
    pub fn with_iterator(iter: Peekable<I>, source: &'src str) -> Self {
        Self {
            iter,
            source,
            bitmap_mode: false,
        }
    }

    #[inline]
    fn peek_char_after_whitespace(&mut self) -> Option<Located<char>> {
        while let Some(c) = self.peek_char() {
            if !c.value.is_whitespace() {
                return Some(c);
            }
            let _ = self.next_char();
        }
        None
    }

    #[inline]
    pub fn content(&self) -> &'src str {
        self.source
    }

    fn peek_char(&mut self) -> Option<Located<char>> {
        self.iter.peek().map(|&c| c.into())
    }

    fn next_char(&mut self) -> Option<Located<char>> {
        self.iter.next().map(|c| c.into())
    }

    fn unsigned_number(&mut self, radix: u32) -> Spanned<Number> {
        let start = self.next_char().unwrap();
        let mut end = start;
        let mut numerator = start.value.to_digit(radix).unwrap() as i32;

        let mut dot = false;

        while let Some(c) = self.peek_char() {
            if let Some(n) = c.value.to_digit(radix) {
                let _ = self.next_char();
                numerator *= radix as i32;
                numerator += n as i32;
                end = c;
            } else if c.value == '.' {
                let _ = self.next_char();
                dot = true;
                break;
            } else {
                break;
            }
        }

        if dot == true {
            while let Some(c) = self.peek_char() {
                if let Some(_) = c.value.to_digit(radix) {
                    let _ = self.next_char();
                    end = c;
                } else {
                    break;
                }
            }

            return Spanned {
                span: Span {
                    start: start.location,
                    end: end.location,
                },
                value: Number::Float(
                    f32::from_str(
                        &self.content()
                            [start.location.offset..(end.location.offset + end.value.len_utf8())],
                    )
                    .unwrap(),
                ),
            };
        }

        Spanned {
            span: Span {
                start: start.location,
                end: end.location,
            },
            value: Number::Integer(numerator),
        }
    }

    fn next_normal(&mut self) -> Option<Spanned<Token<'src>>> {
        let start = self.peek_char_after_whitespace()?;

        match start.value {
            '-' => {
                let _ = self.next_char();

                let next = self.peek_char().map(|t| t.value);
                if next.is_some_and(|c| c.is_digit(10)) {
                    let number = self.unsigned_number(10);
                    let token = match number.value {
                        Number::Float(num) => Token::Number(-num),
                        Number::Integer(int) => Token::Integer(-int),
                    };

                    return Some(Spanned {
                        span: Span {
                            start: start.location,
                            end: number.span.end,
                        },
                        value: token,
                    });
                }

                let mut end = start;

                while let Some(c) = self.peek_char() {
                    if !is_valid_ident_character(c.value) {
                        break;
                    }
                    let _ = self.next_char();
                    end = c;
                }

                let ident = &self.content()
                    [start.location.offset..(end.location.offset + end.value.len_utf8())];

                Some(Spanned {
                    span: Span {
                        start: start.location,
                        end: end.location,
                    },
                    value: Token::Keyword(ident),
                })
            }
            '"' => {
                let _ = self.next_char();
                let mut end = start;
                let mut validity = StringValidity::Unclosed;
                while let Some(c) = self.next_char() {
                    end = c;
                    if c.value == '"' {
                        validity = StringValidity::Valid;
                        break;
                    }
                    if c.value == '\n' {
                        break;
                    }
                }

                let raw_str = match validity {
                    StringValidity::Valid => {
                        &self.content()[start.location.offset + 1..end.location.offset]
                    }
                    StringValidity::Unclosed => &self.content()
                        [start.location.offset + 1..(end.location.offset + end.value.len_utf8())],
                };

                Some(Spanned {
                    span: Span {
                        start: start.location,
                        end: end.location,
                    },
                    value: Token::String(raw_str, validity),
                })
            }
            digit if digit.is_digit(10) => {
                let number = self.unsigned_number(10);

                let token = match number.value {
                    Number::Float(num) => Token::Number(num),
                    Number::Integer(int) => Token::Integer(int),
                };

                Some(Spanned {
                    span: number.span,
                    value: token,
                })
            }
            _ => {
                let mut end = start;

                while let Some(c) = self.peek_char() {
                    if !is_valid_ident_character(c.value) {
                        break;
                    }
                    let _ = self.next_char();
                    end = c;
                }

                let ident = &self.content()
                    [start.location.offset..(end.location.offset + end.value.len_utf8())];

                if ident == "BITMAP" {
                    self.bitmap_mode = true;
                }

                Some(Spanned {
                    span: Span {
                        start: start.location,
                        end: end.location,
                    },
                    value: Token::Keyword(ident),
                })
            }
        }
    }
}

impl<'src, I> Iterator for Lexer<'src, I>
where
    I: Iterator<Item = (usize, char)>,
{
    type Item = Spanned<Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitmap_mode {
            let start = self.peek_char_after_whitespace()?;

            let mut end = start;

            while let Some(c) = self.peek_char() {
                if !is_valid_ident_character(c.value) {
                    break;
                }
                let _ = self.next_char();
                end = c;
            }

            let ident = &self.content()
                [start.location.offset..(end.location.offset + end.value.len_utf8())];

            if ident == "ENDCHAR" {
                self.bitmap_mode = false;
                Some(Spanned {
                    span: Span {
                        start: start.location,
                        end: end.location,
                    },
                    value: Token::Keyword(ident),
                })
            } else {
                if let Ok(hex) = i32::from_str_radix(ident, 16) {
                    Some(Spanned {
                        span: Span {
                            start: start.location,
                            end: end.location,
                        },
                        value: Token::Integer(hex),
                    })
                } else {
                    Some(Spanned {
                        span: Span {
                            start: start.location,
                            end: end.location,
                        },
                        value: Token::Keyword(ident),
                    })
                }
            }
        } else {
            self.next_normal()
        }
    }
}
