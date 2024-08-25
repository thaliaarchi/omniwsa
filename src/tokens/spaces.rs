//! Space tokens and nodes.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
};

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Pretty},
    tokens::Token,
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

/// Horizontal whitespace token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
#[debug("SpaceToken({:?})", space.as_bstr())]
pub struct SpaceToken<'s> {
    /// The text of this whitespace.
    pub space: Cow<'s, [u8]>,
}

/// Line terminator token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
#[debug("LineTermToken({:?})", style)]
pub struct LineTermToken {
    /// The style of this line terminator.
    pub style: LineTermStyle,
}

/// The style of a line terminator
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineTermStyle {
    /// Line feed.
    Lf,
    /// Carriage return, line feed.
    Crlf,
    /// Carriage return.
    Cr,
}

/// End of file token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EofToken;

/// Instruction separator token (e.g., Respace `;` or Palaiologos `/`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstSepToken {
    /// The style of this instruction separator.
    pub style: InstSepStyle,
    /// All errors from parsing this instruction separator.
    pub errors: EnumSet<InstSepError>,
}

/// The style of an argument separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstSepStyle {
    /// `;` argument separator (Respace).
    Semi,
    /// `/` argument separator (Palaiologos).
    Slash,
}

/// A parse error for an instruction separator.
#[derive(EnumSetType, Debug)]
pub enum InstSepError {
    /// Multiple adjacent instruction separators.
    Multiple,
    /// It is at the start of the line.
    StartOfLine,
    /// It is at the end of the line.
    EndOfLine,
}

/// Argument separator token (e.g., Palaiologos `,`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArgSepToken {
    /// The style of this argument separator.
    pub style: ArgSepStyle,
    /// All errors from parsing this argument separator.
    pub errors: EnumSet<ArgSepError>,
}

/// The style of an argument separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgSepStyle {
    /// `,` argument separator (Palaiologos).
    Comma,
}

/// A parse error for an argument separator.
#[derive(EnumSetType, Debug)]
pub enum ArgSepError {
    /// This argument separator is not between arguments.
    NotBetweenArguments,
    /// Multiple adjacent argument separators.
    Multiple,
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
            .position(|tok| !matches!(tok, Token::Space(_)))
            .unwrap_or(self.tokens.len());
        self.tokens.drain(..i);
    }

    /// Trims trailing spaces before the end of a line.
    pub fn trim_trailing(&mut self) {
        let mut j = self.tokens.len();
        if j > 0 && matches!(self.tokens[j - 1], Token::LineTerm(_) | Token::Eof(_)) {
            j -= 1;
        }
        if j > 0 && matches!(self.tokens[j - 1], Token::LineComment(_)) {
            j -= 1;
        }
        let i = self.tokens[..j]
            .iter()
            .rposition(|tok| !matches!(tok, Token::Space(_)))
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
            token,
            Token::Space(_)
                | Token::LineTerm(_)
                | Token::Eof(_)
                | Token::InstSep(_)
                | Token::ArgSep(_)
                | Token::LineComment(_)
                | Token::BlockComment(_)
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

impl LineTermStyle {
    /// The line terminator.
    pub fn as_str(&self) -> &'static str {
        match self {
            LineTermStyle::Lf => "\n",
            LineTermStyle::Crlf => "\r\n",
            LineTermStyle::Cr => "\r",
        }
    }
}

impl InstSepStyle {
    /// The instruction separator.
    pub fn as_str(&self) -> &'static str {
        match self {
            InstSepStyle::Semi => ";",
            InstSepStyle::Slash => "/",
        }
    }
}

impl ArgSepStyle {
    /// The argument separator.
    pub fn as_str(&self) -> &'static str {
        match self {
            ArgSepStyle::Comma => ",",
        }
    }
}

impl<'s, T: Into<Cow<'s, [u8]>>> From<T> for SpaceToken<'s> {
    fn from(space: T) -> Self {
        SpaceToken {
            space: space.into(),
        }
    }
}

impl From<LineTermStyle> for LineTermToken {
    fn from(style: LineTermStyle) -> Self {
        LineTermToken { style }
    }
}

impl From<InstSepStyle> for InstSepToken {
    fn from(style: InstSepStyle) -> Self {
        InstSepToken {
            style,
            errors: EnumSet::empty(),
        }
    }
}

impl From<ArgSepStyle> for ArgSepToken {
    fn from(style: ArgSepStyle) -> Self {
        ArgSepToken {
            style,
            errors: EnumSet::empty(),
        }
    }
}

impl HasError for SpaceToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for LineTermToken {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for EofToken {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for InstSepToken {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl HasError for ArgSepToken {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Pretty for SpaceToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.space.pretty(buf);
    }
}

impl Pretty for LineTermToken {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.as_str().pretty(buf);
    }
}

impl Pretty for EofToken {
    fn pretty(&self, _buf: &mut Vec<u8>) {}
}

impl Pretty for InstSepToken {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.as_str().pretty(buf);
    }
}

impl Pretty for ArgSepToken {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.as_str().pretty(buf);
    }
}
