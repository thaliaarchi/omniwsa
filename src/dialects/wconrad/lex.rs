//! Lexer for the wconrad Whitespace assembly dialect.

use crate::{
    lex::{Lex, Scanner},
    tokens::{spaces::EofToken, Token},
};

/// A lexer for tokens in the wconrad Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Scanner<'s>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for wconrad-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        Lexer {
            scan: Scanner::new(src),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.start_next();
        if scan.eof() {
            return Token::from(EofToken);
        }
        match scan.next_char_or_replace() {
            '-' | '+' | '0'..='9' => todo!(),
            '#' => todo!(),
            ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r' => todo!(),
            _ => todo!(),
        }
    }
}
