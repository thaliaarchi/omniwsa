//! Parsing for the Burghard Whitespace assembly dialect.

use std::str;

use crate::scan::Utf8Scanner;

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Utf8Scanner<'s>,
    /// A sequence of invalid UTF-8.
    invalid: &'s [u8],
    /// Text following invalid UTF-8.
    rest: &'s [u8],
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        let (valid, invalid, rest) = match str::from_utf8(src) {
            Ok(src) => (src, &b""[..], &b""[..]),
            Err(err) => {
                let (valid, rest) = src.split_at(err.valid_up_to());
                let (invalid, rest) = rest.split_at(err.error_len().unwrap_or(rest.len()));
                // SAFETY: This sequence has been validated as UTF-8.
                (unsafe { str::from_utf8_unchecked(valid) }, invalid, rest)
            }
        };
        Lexer {
            scan: Utf8Scanner::new(valid),
            invalid,
            rest,
        }
    }
}
