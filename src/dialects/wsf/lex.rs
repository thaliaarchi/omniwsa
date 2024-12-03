//! Lexer for the wsf Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    dialects::{dialect::DialectState, Wsf},
    lex::{Lex, Scanner},
    tokens::{
        comment::{LineCommentError, LineCommentStyle, LineCommentToken},
        label::LabelColonToken,
        spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken},
        string::Encoding,
        ErrorToken, Token, WordToken,
    },
};

// TODO:
// - Lex words and fused tokens.
// - Handle \x char escapes. Change `unescape_byte` to return
//   `enum Escape { Byte(u8), Hex2, Invalid }`.

/// A lexer for tokens in the wsf Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s, 'd> {
    dialect: &'d DialectState<Wsf>,
    scan: Scanner<'s>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for wsf-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d DialectState<Wsf>) -> Self {
        Lexer {
            dialect,
            scan: Scanner::new(src),
            digit_buf: Vec::new(),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s, '_> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.start_next();

        if scan.eof() {
            return EofToken.into();
        }

        match scan.next_char() {
            'A'..='Z' | 'a'..='z' | '_' | '.' => {
                scan.bump_while_ascii(|ch| {
                    ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' || ch == b'.'
                });
                Token::from(WordToken {
                    word: scan.text().into(),
                    errors: EnumSet::empty(),
                })
            }
            '-' | '0'..='9' => {
                scan.bump_while_ascii(|ch| ch.is_ascii_digit());
                self.dialect
                    .integers()
                    .parse(scan.text().into(), &mut self.digit_buf)
                    .into()
            }
            '"' => scan
                .string_lit_oneline()
                .unescape_simple(unescape_byte(true), Encoding::Bytes)
                .into(),
            '\'' => scan
                .char_lit_oneline()
                .unescape_simple(unescape_byte(false), Encoding::Bytes)
                .into(),
            ':' => LabelColonToken.into(),
            '#' => {
                let text = scan.bump_until_lf();
                let mut errors = EnumSet::new();
                if scan.has_invalid_utf8() {
                    errors |= LineCommentError::InvalidUtf8;
                };
                Token::from(LineCommentToken {
                    text,
                    style: LineCommentStyle::Semi,
                    errors,
                })
            }
            '\n' => Token::from(LineTermToken::from(LineTermStyle::Lf)),
            ' ' | '\t' | '\x0b' | '\x0c' | '\r' => {
                scan.bump_while_ascii(|ch| matches!(ch, b' ' | b'\t' | b'\x0b' | b'\x0c' | b'\r'));
                Token::from(SpaceToken::from(scan.text()))
            }
            _ => {
                scan.bump_until_ascii(|ch| {
                    matches!(ch,
                        b'A'..=b'Z'
                        | b'a'..=b'z'
                        | b'_'
                        | b'-'
                        | b'0'..=b'9'
                        | b':'
                        | b'#'
                        | b'\n'
                        | b' '
                        | b'\t'
                        | b'\x0b'
                        | b'\x0c'
                        | b'\r'
                    )
                });
                Token::from(ErrorToken::from(scan.text()))
            }
        }
    }
}

/// Resolves a backslash-escaped byte to its represented value.
#[inline]
fn unescape_byte(double_quote: bool) -> impl Fn(u8) -> Option<u8> {
    move |b| match b {
        b'"' if double_quote => Some(b'"'),
        b'\'' if !double_quote => Some(b'\''),
        b'\\' => Some(b'\\'),
        b'a' => Some(b'\x07'),
        b'b' => Some(b'\x08'),
        b'e' => Some(b'\x1b'),
        b'f' => Some(b'\x0c'),
        b'n' => Some(b'\n'),
        b'r' => Some(b'\r'),
        b't' => Some(b'\t'),
        b'v' => Some(b'\x0b'),
        _ => None,
    }
}
