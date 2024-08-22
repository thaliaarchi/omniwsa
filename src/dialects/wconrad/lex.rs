//! Lexer for the wconrad Whitespace assembly dialect.

use crate::{
    lex::{ByteScanner, Lex},
    tokens::{spaces::EofToken, Token},
};

/// A lexer for tokens in the wconrad Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: ByteScanner<'s>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for wconrad-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        Lexer {
            scan: ByteScanner::new(src),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.reset();
        if scan.eof() {
            return Token::from(EofToken);
        }
        match scan.next_byte() {
            b'-' | b'+' | b'0'..=b'9' => todo!(),
            b'#' => todo!(),
            b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r' => todo!(),
            _ => todo!(),
        }
    }
}
