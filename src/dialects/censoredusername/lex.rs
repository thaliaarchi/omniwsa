//! Lexer for the CensoredUsername Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    dialects::{CensoredUsername, dialect::DialectState},
    lex::{Lex, Scanner},
    tokens::{
        ErrorToken, Token, WordToken,
        comment::{LineCommentError, LineCommentStyle, LineCommentToken},
        label::LabelColonToken,
        spaces::{ArgSepStyle, ArgSepToken, EofToken, LineTermStyle, LineTermToken, SpaceToken},
    },
};

/// A lexer for tokens in the CensoredUsername Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s, 'd> {
    dialect: &'d DialectState<CensoredUsername>,
    scan: Scanner<'s>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for CensoredUsername-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d DialectState<CensoredUsername>) -> Self {
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
            'A'..='Z' | 'a'..='z' | '_' => {
                scan.bump_while_ascii(|ch| ch.is_ascii_alphanumeric() || ch == b'_');
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
            ':' => LabelColonToken.into(),
            ',' => Token::from(ArgSepToken::from(ArgSepStyle::Comma)),
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
            ' ' | '\t' => {
                scan.bump_while_ascii(|ch| ch == b' ' || ch == b'\t');
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
                        | b','
                        | b';'
                        | b'\n'
                        | b' '
                        | b'\t'
                    )
                });
                Token::from(ErrorToken::from(scan.text()))
            }
        }
    }
}
