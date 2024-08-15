//! String literal parsing and token.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    str,
    str::Utf8Error,
};

use bstr::ByteSlice;

use crate::{syntax::HasError, tokens::Token};

// TODO:
// - How to represent escapes in strings and chars?
// - How to represent char literals with buggy delimiters, like those allowed
//   with littleBugHunter's `'..` pattern? Maybe QuoteStyle::Custom with open
//   and close.

/// A string literal token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringToken<'s> {
    /// The unescaped data of this string literal.
    pub data: StringData<'s>,
    /// The style of the quotes enclosing this string literal.
    pub quotes: QuoteStyle,
}

/// A character literal token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharToken {
    /// The unescaped data of this char literal.
    pub data: CharData,
    /// The style of the quotes enclosing this char literal.
    pub quotes: QuoteStyle,
}

/// A token enclosed in non-semantic quotes (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuotedToken<'s> {
    /// The effective token.
    pub inner: Box<Token<'s>>,
    /// The style of the quotes enclosing this token.
    pub quotes: QuoteStyle,
}

/// The unescaped data of a string literal.
#[derive(Clone, PartialEq, Eq)]
pub enum StringData<'s> {
    /// A string encoded as UTF-8 chars.
    Utf8(Cow<'s, str>),
    /// A string encoded as raw bytes.
    Bytes(Cow<'s, [u8]>),
}

/// The unescaped data of a char literal.
#[derive(Clone, PartialEq, Eq)]
pub enum CharData {
    /// A char encoded as a Unicode code point.
    Unicode(char),
    /// A char encoded as a byte.
    Byte(u8),
    /// A char literal with no or more than one char.
    Error,
}

/// The quote style of a string or char literal or a quoted word.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuoteStyle {
    /// A string enclosed in `"`-quotes.
    Double,
    /// A string enclosed in `'`-quotes (e.g., Whitelips non-NUL-terminated
    /// strings).
    Single,
    /// A `"`-quoted string missing a closing quote, i.e., an error.
    UnclosedDouble,
    /// A `'`-quoted string missing a closing quote, i.e., an error.
    UnclosedSingle,
    /// A string not enclosed in quotes (Burghard).
    Bare,
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

impl HasError for CharData {
    fn has_error(&self) -> bool {
        match self {
            CharData::Unicode(_) | CharData::Byte(_) => false,
            CharData::Error => true,
        }
    }
}

impl HasError for QuoteStyle {
    fn has_error(&self) -> bool {
        match self {
            QuoteStyle::Double | QuoteStyle::Single | QuoteStyle::Bare => false,
            QuoteStyle::UnclosedDouble | QuoteStyle::UnclosedSingle => true,
        }
    }
}

impl Debug for StringData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StringData::Utf8(s) => f.debug_tuple("Utf8").field(s).finish(),
            StringData::Bytes(b) => f.debug_tuple("Bytes").field(&b.as_bstr()).finish(),
        }
    }
}

impl Debug for CharData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CharData::Unicode(c) => f.debug_tuple("Unicode").field(c).finish(),
            CharData::Byte(b) => f.debug_tuple("Byte").field(&[*b].as_bstr()).finish(),
            CharData::Error => write!(f, "Error"),
        }
    }
}
