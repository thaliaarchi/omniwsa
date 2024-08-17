//! A stream of tokens for matching and aggregating tokens.

use std::mem;

use crate::tokens::{Token, TokenKind};

/// A lexical scanner for some Whitespace assembly dialect.
pub trait Lex<'s> {
    /// Scans the next token from the source.
    fn next_token(&mut self) -> Token<'s>;
}

/// A stream of tokens for matching against the current token and aggregating
/// tokens.
#[derive(Clone, Debug)]
pub struct TokenStream<'s, L> {
    lex: L,
    tok: Token<'s>,
}

impl<'s, L: Lex<'s>> TokenStream<'s, L> {
    /// Constructs a new token stream for processing the tokens from a lexer.
    pub fn new(mut lexer: L) -> Self {
        let tok = lexer.next_token();
        TokenStream { lex: lexer, tok }
    }

    /// Returns the kind of the current token.
    pub fn curr(&self) -> &TokenKind<'s> {
        &self.tok.kind
    }

    /// Returns the current token and advances to the next token.
    pub fn advance(&mut self) -> Token<'s> {
        mem::replace(&mut self.tok, self.lex.next_token())
    }

    /// Returns whether the parser is at EOF.
    pub fn eof(&self) -> bool {
        matches!(self.curr(), TokenKind::Eof(_))
    }
}
