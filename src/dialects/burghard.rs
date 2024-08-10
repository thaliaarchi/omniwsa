//! Parsing for the Burghard Whitespace assembly dialect.

use std::str;

use crate::{
    scan::Utf8Scanner,
    token::{Token, TokenError, TokenKind},
};

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Utf8Scanner<'s>,
    /// The remaining text at the first UTF-8 error and the length of the
    /// invalid sequence.
    invalid_utf8: Option<(&'s [u8], usize)>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        let (src, invalid_utf8) = match str::from_utf8(src) {
            Ok(src) => (src, None),
            Err(err) => {
                let (valid, rest) = src.split_at(err.valid_up_to());
                let error_len = err.error_len().unwrap_or(rest.len());
                // SAFETY: This sequence has been validated as UTF-8.
                let valid = unsafe { str::from_utf8_unchecked(valid) };
                (valid, Some((rest, error_len)))
            }
        };
        Lexer {
            scan: Utf8Scanner::new(src),
            invalid_utf8,
        }
    }

    /// Scans the next token from the source.
    pub fn next_token(&mut self) -> Token<'s> {
        self.scan.reset();

        if self.scan.eof() {
            if let Some((rest, error_len)) = self.invalid_utf8.take() {
                return Token::new(
                    rest,
                    TokenKind::Error(TokenError::InvalidUtf8 { error_len }),
                );
            }
            return Token::new(b"", TokenKind::Eof);
        }

        match self.scan.next_char() {
            ';' => self.scan.line_comment(),
            '-' if self.scan.bump_if(|c| c == '-') => self.scan.line_comment(),
            // TODO: Make nested.
            '{' if self.scan.bump_if(|c| c == '-') => self.scan.block_comment(*b"-}"),
            _ => {
                todo!()
            }
        }
    }
}
