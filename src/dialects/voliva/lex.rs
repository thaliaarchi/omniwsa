//! Lexer for the voliva Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    dialects::{Voliva, dialect::DialectState},
    lex::{Lex, Scanner},
    tokens::{
        Token, WordError, WordToken,
        comment::{LineCommentError, LineCommentStyle, LineCommentToken},
        integer::{BaseStyle, Sign},
        spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken},
        string::Encoding,
    },
};

// TODO:
// - Create token type for decorator comments.
// - Handle `_`-variables in parser.
// - Replace each byte of invalid UTF-8 sequences with the U+FFFD replacement
//   character. This would require either splitting `Encoding::Utf8` into
//   `Utf8Max` and `Utf8Min` or adding an orthogonal error resolution strategy.

/// A lexer for tokens in the voliva Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s, 'd> {
    dialect: &'d DialectState<Voliva>,
    scan: Scanner<'s>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for voliva-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d DialectState<Voliva>) -> Self {
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
            '"' => scan
                .string_lit_oneline()
                .unescape_simple(unescape(true), Encoding::Utf8)
                .into(),
            '\'' => scan
                .char_lit_oneline()
                .unescape_simple(unescape(false), Encoding::Utf8)
                .into(),
            ';' => {
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
            ch if is_space(ch) => {
                scan.bump_while_char(is_space);
                Token::from(SpaceToken::from(scan.text()))
            }
            _ => {
                scan.bump_until_char(|ch| is_space(ch) || matches!(ch, ';' | '"' | '\'' | '\n'));

                let int = self
                    .dialect
                    .integers()
                    .parse(scan.text().into(), &mut self.digit_buf);
                // Explicit signs are only allowed for decimal.
                if int.errors.is_empty()
                    && (int.sign == Sign::None || int.base_style == BaseStyle::Decimal)
                {
                    return Token::from(int);
                }

                let mut errors = EnumSet::new();
                if scan.has_invalid_utf8() {
                    errors |= WordError::InvalidUtf8;
                };
                Token::from(WordToken {
                    word: scan.text().into(),
                    errors,
                })
            }
        }
    }
}

/// Returns whether a char is a whitespace character according to JavaScript
/// [`RegExp` `\s`](https://tc39.es/ecma262/multipage/text-processing.html#sec-compiletocharset),
/// excluding `\n`.
fn is_space(ch: char) -> bool {
    match ch {
        '\t' | '\u{000b}' | '\u{000c}' | '\r' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'
        | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
        | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200a}' | '\u{2028}' | '\u{2029}'
        | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}' => true,
        _ => false,
    }
}

/// Resolves a backslash-escaped char to its represented value.
#[inline]
fn unescape(double_quote: bool) -> impl Fn(char) -> Option<char> {
    move |b| match b {
        '"' if double_quote => Some('"'),
        '\'' if !double_quote => Some('\''),
        '\\' => Some('\\'),
        'b' => Some('\x08'),
        'f' => Some('\x0c'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\x0b'),
        _ => None,
    }
}
