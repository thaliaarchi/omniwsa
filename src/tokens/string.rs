//! String literal parsing and token.

use std::{borrow::Cow, str, str::Utf8Error};

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
    pub unescaped: StringData<'s>,
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

/// The unescaped data of a string literal.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub enum StringData<'s> {
    /// A string encoded as UTF-8 chars.
    Utf8(Cow<'s, str>),
    /// A string encoded as raw bytes.
    Bytes(#[debug("{:?}", _0.as_bstr())] Cow<'s, [u8]>),
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
}

/// A parse error for a char literal.
#[derive(EnumSetType, Debug)]
pub enum CharError {
    /// Has no closing quote.
    Unterminated,
    /// Has no chars.
    Empty,
    /// Has more than one char.
    MoreThanOneChar,
}

/// A parse error for a quoted token.
#[derive(EnumSetType, Debug)]
pub enum QuotedError {
    /// Has no closing quote.
    Unterminated,
}

impl<'s> StringData<'s> {
    /// Constructs a `StringData` from bytes, validating that it is UTF-8.
    pub fn from_utf8(b: Cow<'s, [u8]>) -> Result<Self, Utf8Error> {
        let s = match b {
            Cow::Borrowed(b) => Cow::Borrowed(str::from_utf8(b)?),
            Cow::Owned(b) => Cow::Owned(String::from_utf8(b).map_err(|err| err.utf8_error())?),
        };
        Ok(StringData::Utf8(s))
    }

    /// Constructs a `StringData` from bytes, assuming that it is valid UTF-8.
    ///
    /// # Safety
    ///
    /// The bytes must be valid UTF-8.
    pub unsafe fn from_utf8_unchecked(b: Cow<'s, [u8]>) -> Self {
        // SAFETY: Guaranteed by caller.
        let s = unsafe {
            match b {
                Cow::Borrowed(b) => Cow::Borrowed(str::from_utf8_unchecked(b)),
                Cow::Owned(b) => Cow::Owned(String::from_utf8_unchecked(b)),
            }
        };
        StringData::Utf8(s)
    }
}

impl QuoteStyle {
    /// The opening and closing quote.
    pub fn quote(&self) -> &'static str {
        match self {
            QuoteStyle::Double => "\"",
            QuoteStyle::Single => "'",
            QuoteStyle::Bare => "",
        }
    }
}

impl<'s> From<Cow<'s, str>> for StringData<'s> {
    fn from(s: Cow<'s, str>) -> Self {
        StringData::Utf8(s)
    }
}

impl<'s> From<Cow<'s, [u8]>> for StringData<'s> {
    fn from(b: Cow<'s, [u8]>) -> Self {
        StringData::Bytes(b)
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
