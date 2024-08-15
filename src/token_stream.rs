//! A stream of tokens for matching and aggregating tokens.

use std::mem;

use crate::{
    syntax::{InstSep, Space},
    tokens::{Token, TokenError, TokenKind},
};

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
        matches!(self.curr(), TokenKind::Eof)
    }

    /// Consumes space and block comment tokens.
    pub fn space(&mut self) -> Space<'s> {
        let mut space = Space::new();
        while matches!(
            self.curr(),
            TokenKind::Space | TokenKind::BlockComment { .. }
        ) {
            space.push(self.advance());
        }
        space
    }

    /// Consumes a line comment token.
    pub fn line_comment(&mut self) -> Option<Token<'s>> {
        match self.curr() {
            TokenKind::LineComment { .. } => Some(self.advance()),
            _ => None,
        }
    }

    /// Consumes a line terminator, EOF, or invalid UTF-8 error token.
    pub fn line_term(&mut self) -> Option<Token<'s>> {
        match self.curr() {
            TokenKind::LineTerm | TokenKind::Eof | TokenKind::Error(TokenError::Utf8 { .. }) => {
                Some(self.advance())
            }
            _ => None,
        }
    }

    /// Consumes an optional line comment, followed by a line terminator (or EOF
    /// or invalid UTF-8 error). Panics if not at such a token.
    pub fn line_term_sep(&mut self, space_before: Space<'s>) -> InstSep<'s> {
        InstSep::LineTerm {
            space_before,
            line_comment: self.line_comment(),
            line_term: self.line_term().expect("line terminator"),
        }
    }
}
