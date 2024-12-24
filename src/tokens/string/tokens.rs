//! String and char literal tokens.

use std::{borrow::Cow, str};

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Pretty},
    tokens::Token,
};

// TODO:
// - Create StringSyntax to describe escapes.
// - How to represent char literals with buggy delimiters, like those allowed
//   with littleBugHunter's `'..` pattern? Maybe QuoteStyle::Custom with open
//   and close.
// - Improve Debug for CharData::Byte.

/// A string literal token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct StringToken<'s> {
    /// The literal, escaped text, including quotes.
    #[debug("{:?}", literal.as_bstr())]
    pub literal: Cow<'s, [u8]>,
    /// The unescaped data.
    #[debug("{:?}", unescaped.as_bstr())]
    pub unescaped: Cow<'s, [u8]>,
    /// The encoding of the unescaped data.
    pub encoding: Encoding,
    /// The style of the quotes enclosing this string literal.
    pub quotes: QuoteStyle,
    /// All errors from parsing this string literal. When any errors are
    /// present, the unescaped data is best-effort.
    pub errors: EnumSet<StringError>,
}

/// A character literal token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct CharToken<'s> {
    /// The literal, escaped text, including quotes.
    #[debug("{:?}", literal.as_bstr())]
    pub literal: Cow<'s, [u8]>,
    /// The unescaped data.
    pub unescaped: CharData,
    /// The style of the quotes enclosing this char literal.
    pub quotes: QuoteStyle,
    /// All errors from parsing this char literal. When any errors are present,
    /// the unescaped data is best-effort.
    pub errors: EnumSet<CharError>,
}

/// A token enclosed in non-semantic quotes (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotedToken<'s> {
    /// The effective token.
    pub inner: Box<Token<'s>>,
    /// The style of the quotes enclosing this token.
    pub quotes: QuoteStyle,
    /// All errors from parsing this quoted token.
    pub errors: EnumSet<QuotedError>,
}

/// The encoding of an unescaped string literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// A string encoded as UTF-8 chars.
    Utf8,
    /// A string encoded as raw bytes.
    Bytes,
}

/// The unescaped data of a char literal.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub enum CharData {
    /// A char encoded as a Unicode code point.
    Unicode(char),
    /// A char encoded as a byte.
    Byte(#[debug("{:?}", &[*_0].as_bstr())] u8),
}

/// The quote style of a string or char literal or a quoted word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteStyle {
    /// A string enclosed in `"`-quotes.
    Double,
    /// A string enclosed in `'`-quotes (e.g., Whitelips non-NUL-terminated
    /// strings).
    Single,
    /// A string not enclosed in quotes (Burghard).
    Bare,
}

/// A parse error for a string literal.
#[derive(EnumSetType, Debug)]
pub enum StringError {
    /// Has no closing quote.
    Unterminated,
    /// Invalid escape sequence.
    InvalidEscape,
    /// Contains invalid UTF-8.
    InvalidUtf8,
}

/// A parse error for a char literal.
#[derive(EnumSetType, Debug)]
pub enum CharError {
    /// Has no closing quote.
    Unterminated,
    /// Has no chars.
    Empty,
    /// Has more than one char.
    MultipleChars,
    /// Invalid escape sequence.
    InvalidEscape,
    /// Contains invalid UTF-8.
    InvalidUtf8,
    /// Expected exactly one byte, not a non-ASCII Unicode code point.
    UnexpectedUnicode,
}

/// A parse error for a quoted token.
#[derive(EnumSetType, Debug)]
pub enum QuotedError {
    /// Has no closing quote.
    Unterminated,
}

impl Encoding {
    /// Returns the character to use for representing invalid sequences: either
    /// U+FFFD replacement character for `Utf8` or U+001A substitute (SUB) for
    /// `Bytes`.
    pub const fn replacement(&self) -> char {
        match self {
            Encoding::Utf8 => '\u{fffd}',
            Encoding::Bytes => '\x1a',
        }
    }
}

impl QuoteStyle {
    /// The opening and closing quote.
    pub const fn quote(&self) -> &'static str {
        match self {
            QuoteStyle::Double => "\"",
            QuoteStyle::Single => "'",
            QuoteStyle::Bare => "",
        }
    }
}

impl From<QuotedError> for StringError {
    fn from(err: QuotedError) -> Self {
        match err {
            QuotedError::Unterminated => StringError::Unterminated,
        }
    }
}

impl HasError for StringToken<'_> {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl HasError for CharToken<'_> {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl HasError for QuotedToken<'_> {
    fn has_error(&self) -> bool {
        self.inner.has_error() || !self.errors.is_empty()
    }
}

impl Pretty for StringToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.quotes.quote().pretty(buf);
        self.literal.pretty(buf);
        if !self.errors.contains(StringError::Unterminated) {
            self.quotes.quote().pretty(buf);
        }
    }
}

impl Pretty for CharToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.quotes.quote().pretty(buf);
        self.literal.pretty(buf);
        if !self.errors.contains(CharError::Unterminated) {
            self.quotes.quote().pretty(buf);
        }
    }
}

impl Pretty for QuotedToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.quotes.quote().pretty(buf);
        self.inner.pretty(buf);
        if !self.errors.contains(QuotedError::Unterminated) {
            self.quotes.quote().pretty(buf);
        }
    }
}
