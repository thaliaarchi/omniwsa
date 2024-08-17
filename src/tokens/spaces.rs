//! Space tokens and nodes.

use std::fmt::{self, Debug, Formatter};

use crate::{
    syntax::HasError,
    tokens::{Token, TokenError, TokenKind},
};

// TODO:
// - Possibly switch Space to VecDeque.

/// A sequence of whitespace, separator, and comment tokens.
#[derive(Clone, PartialEq, Eq)]
pub struct Spaces<'s> {
    /// The contained tokens. All are always `Space`, `LineTerm`, `Eof`,
    /// `LineComment`, `BlockComment`, `InstSep`, or `ArgSep`.
    pub tokens: Vec<Token<'s>>,
}

impl<'s> Spaces<'s> {
    /// Constructs a new, empty space sequence.
    pub fn new() -> Self {
        Spaces { tokens: Vec::new() }
    }

    /// Returns a reference to the space tokens in this sequence.
    pub fn tokens(&self) -> &[Token<'s>] {
        &self.tokens
    }

    /// Returns a mutable reference to the space tokens in this sequence.
    pub fn tokens_mut(&mut self) -> &mut [Token<'s>] {
        &mut self.tokens
    }

    /// Pushes a whitespace or block comment token to the sequence.
    pub fn push(&mut self, token: Token<'s>) {
        Self::assert_space(&token);
        self.tokens.push(token);
    }

    /// Pushes a whitespace or block comment token to the front of the sequence.
    pub fn push_front(&mut self, token: Token<'s>) {
        Self::assert_space(&token);
        self.tokens.insert(0, token);
    }

    /// Trims leading spaces.
    pub fn trim_leading(&mut self) {
        let i = self
            .tokens
            .iter()
            .position(|tok| tok.kind != TokenKind::Space)
            .unwrap_or(self.tokens.len());
        self.tokens.drain(..i);
    }

    /// Trims trailing spaces before the end of a line.
    pub fn trim_trailing(&mut self) {
        let mut j = self.tokens.len();
        if j > 0
            && matches!(
                self.tokens[j - 1].kind,
                TokenKind::LineTerm | TokenKind::Eof | TokenKind::Error(TokenError::Utf8 { .. })
            )
        {
            j -= 1;
        }
        if j > 0 && matches!(self.tokens[j - 1].kind, TokenKind::LineComment { .. }) {
            j -= 1;
        }
        let i = self.tokens[..j]
            .iter()
            .rposition(|tok| tok.kind != TokenKind::Space)
            .map(|i| i + 1)
            .unwrap_or(0);
        self.tokens.drain(i..j);
    }

    /// Returns the number of tokens in this sequence.
    #[inline]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns whether this sequence contains no tokens.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn assert_space(token: &Token<'s>) {
        debug_assert!(matches!(
            token.kind,
            TokenKind::Space
                | TokenKind::LineTerm
                | TokenKind::Eof
                | TokenKind::InstSep
                | TokenKind::ArgSep
                | TokenKind::LineComment { .. }
                | TokenKind::BlockComment { .. }
        ));
    }
}

impl HasError for Spaces<'_> {
    fn has_error(&self) -> bool {
        self.tokens.has_error()
    }
}

impl<'s> From<Vec<Token<'s>>> for Spaces<'s> {
    fn from(tokens: Vec<Token<'s>>) -> Self {
        tokens.iter().for_each(Self::assert_space);
        Spaces { tokens }
    }
}

impl<'s> From<Token<'s>> for Spaces<'s> {
    fn from(token: Token<'s>) -> Self {
        Self::assert_space(&token);
        Spaces {
            tokens: vec![token],
        }
    }
}

impl Debug for Spaces<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Spaces ")?;
        f.debug_list().entries(&self.tokens).finish()
    }
}
