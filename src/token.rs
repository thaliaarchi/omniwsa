//! Lexical tokens for interoperable Whitespace assembly.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
    str::{self, Utf8Error},
};

use bstr::ByteSlice;
use enumset::{EnumSet, EnumSetType};
use rug::Integer;

pub use crate::mnemonics::Opcode;
use crate::syntax::HasError;

// TODO:
// - Whitelips, Lime, and Respace macro definitions.
// - Respace `@define`.
// - How to represent escapes in strings and chars?
// - How to represent equivalent integers?
// - Store byte string uniformly, instead of a mix of &[u8] and Cow.
//   - Create utilities for slicing and manipulating easier than Cow.
// - Move `Token::text` into token variants, so text is not stored redundantly,
//   and rename `TokenKind` -> `Token`. For example, the line comment prefix
//   needs to be manipulated in both `Token::text` and `LineComment::prefix`.
// - Extract each token as a struct to manage manipulation routines.
// - How to represent char literals with buggy delimiters, like those allowed
//   with littleBugHunter's `'..` pattern? Maybe QuoteStyle::Custom with open
//   and close.
// - Make UTF-8 error a first-class token.

/// A lexical token, a unit of scanned text, in interoperable Whitespace
/// assembly.
#[derive(Clone, PartialEq, Eq)]
pub struct Token<'s> {
    pub text: Cow<'s, [u8]>,
    pub kind: TokenKind<'s>,
}

/// A kind of token.
#[derive(Clone, PartialEq, Eq)]
pub enum TokenKind<'s> {
    /// Instruction or predefined macro opcode.
    Opcode(Opcode),
    /// Integer.
    Integer(IntegerToken),
    /// Character.
    Char { data: CharData, quotes: QuoteStyle },
    /// String.
    String {
        data: StringData<'s>,
        quotes: QuoteStyle,
    },
    /// Identifier.
    Ident {
        /// A prefix sigil to mark identifiers (e.g., Burghard `_`).
        sigil: &'s [u8],
        /// The identifier with its sigil removed.
        ident: Cow<'s, [u8]>,
    },
    /// Label.
    Label {
        /// A prefix sigil to mark labels (e.g., Palaiologos `@` and `%`).
        sigil: &'s [u8],
        /// The label with its sigil removed.
        label: Cow<'s, [u8]>,
        /// Errors for this label.
        errors: EnumSet<LabelError>,
    },
    /// Label colon marker (i.e., `:`).
    LabelColon,
    /// Instruction separator (e.g., Respace `;` or Palaiologos `/`).
    InstSep,
    /// Argument separator (e.g., Palaiologos `,`).
    ArgSep,
    /// Horizontal whitespace.
    Space,
    /// Line terminator.
    LineTerm,
    /// End of file.
    Eof,
    /// Line comment (e.g., `//`).
    LineComment {
        prefix: &'s [u8],
        text: &'s [u8],
        /// Errors for this line comment.
        errors: EnumSet<LineCommentError>,
    },
    /// Block comment (e.g., `/* */`).
    /// Sequences ignored due to a bug in the reference parser also count as
    /// block comments (e.g., voliva ignored arguments).
    BlockComment {
        open: &'s [u8],
        text: &'s [u8],
        close: &'s [u8],
        nested: bool,
        terminated: bool,
    },
    /// A word of uninterpreted meaning.
    Word,
    /// A token enclosed in non-semantic quotes (Burghard).
    Quoted {
        inner: Box<Token<'s>>,
        quotes: QuoteStyle,
    },
    /// Tokens spliced by block comments (Burghard).
    Spliced {
        /// A list of words interspersed with block comments. Only contains
        /// `Word` and `BlockComment`.
        tokens: Vec<Token<'s>>,
        /// The effective token.
        spliced: Box<Token<'s>>,
    },
    /// An erroneous sequence.
    Error(TokenError),
}

/// An integer token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerToken {
    pub value: Integer,
    pub sign: IntegerSign,
    pub base: IntegerBase,
    pub leading_zeros: usize,
    pub errors: EnumSet<IntegerError>,
}

/// The sign of an integer literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerSign {
    /// Implicit positive sign.
    None,
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

/// The base (radix) of an integer literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegerBase {
    /// Base 2.
    Binary = 2,
    /// Base 8.
    Octal = 8,
    /// Base 10.
    Decimal = 10,
    /// Base 16.
    Hexadecimal = 16,
}

/// A parse error for an integer literal.
#[derive(EnumSetType, Debug)]
pub enum IntegerError {
    /// An invalid digit.
    InvalidDigit,
    /// No digits, excluding a possible base prefix.
    NoDigits,
    /// Has a sign that is invalid or not supported.
    InvalidSign,
    /// Has a base that is not supported.
    InvalidBase,
    /// Uses digit separators, which are not supported.
    InvalidDigitSep,
    /// An unpaired parenthesis (Burghard via Haskell `Integer`).
    UnpairedParen,
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

/// The unescaped data of a string literal.
#[derive(Clone, PartialEq, Eq)]
pub enum StringData<'s> {
    /// A string encoded as UTF-8 chars.
    Utf8(Cow<'s, str>),
    /// A string encoded as raw bytes.
    Bytes(Cow<'s, [u8]>),
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

/// A parse error for a label.
#[derive(EnumSetType, Debug)]
pub enum LabelError {
    /// The label has no characters (Palaiologos).
    Empty,
    /// The first character is a digit, which is not allowed (Palaiologos).
    StartsWithDigit,
}

/// A parse error for a line comment.
#[derive(EnumSetType, Debug)]
pub enum LineCommentError {
    /// The line comment is not terminated by a LF (Palaiologos).
    MissingLf,
}

/// A lexical error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenError {
    /// Invalid UTF-8 sequence (Burghard).
    Utf8 {
        /// Length of the invalid sequence.
        error_len: usize,
    },
    /// Unrecognized characters.
    UnrecognizedChar,
}

impl<'s> Token<'s> {
    /// Constructs a new token.
    #[inline]
    pub fn new<T: Into<Cow<'s, [u8]>>>(text: T, kind: TokenKind<'s>) -> Self {
        Token {
            text: text.into(),
            kind,
        }
    }

    /// Unwrap non-semantic splices and quotes.
    pub fn unwrap(&self) -> &Token<'s> {
        let mut tok = self;
        loop {
            match &tok.kind {
                TokenKind::Quoted { inner, .. } | TokenKind::Spliced { spliced: inner, .. } => {
                    tok = inner;
                }
                _ => return tok,
            }
        }
    }

    /// Unwrap non-semantic splices and quotes and return a mutable reference.
    pub fn unwrap_mut(&mut self) -> &mut Token<'s> {
        let mut tok = self;
        loop {
            match tok.kind {
                TokenKind::Quoted { ref mut inner, .. }
                | TokenKind::Spliced {
                    spliced: ref mut inner,
                    ..
                } => {
                    tok = inner;
                }
                _ => return tok,
            }
        }
    }

    /// Trim trailing whitespace in a line comment.
    pub fn line_comment_trim_trailing(&mut self) {
        match &mut self.kind {
            TokenKind::LineComment { prefix, text, .. } => {
                let i = text
                    .iter()
                    .rposition(|&b| b != b' ' && b != b'\t')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                *text = &text[..i];
                match &mut self.text {
                    Cow::Borrowed(text) => *text = &text[..prefix.len() + i],
                    Cow::Owned(text) => text.truncate(prefix.len() + i),
                }
            }
            _ => panic!("not a line comment"),
        }
    }
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

impl HasError for Token<'_> {
    fn has_error(&self) -> bool {
        match &self.kind {
            TokenKind::Opcode(Opcode::Invalid) | TokenKind::Error(_) => true,
            TokenKind::Char { data, quotes } => data.has_error() || quotes.has_error(),
            TokenKind::String { quotes, .. } => quotes.has_error(),
            TokenKind::LineComment { errors, .. } => !errors.is_empty(),
            TokenKind::BlockComment { terminated, .. } => !terminated,
            TokenKind::Quoted { inner, quotes, .. } => inner.has_error() || quotes.has_error(),
            TokenKind::Spliced { tokens, .. } => tokens.iter().any(Token::has_error),
            _ => false,
        }
    }
}

impl HasError for Opcode {
    fn has_error(&self) -> bool {
        matches!(self, Opcode::Invalid)
    }
}

impl HasError for IntegerToken {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
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

impl From<Opcode> for TokenKind<'static> {
    fn from(opcode: Opcode) -> Self {
        TokenKind::Opcode(opcode)
    }
}
impl From<IntegerToken> for TokenKind<'static> {
    fn from(int: IntegerToken) -> Self {
        TokenKind::Integer(int)
    }
}
impl From<TokenError> for TokenKind<'static> {
    fn from(err: TokenError) -> Self {
        TokenKind::Error(err)
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Token")
            .field(&self.text.as_bstr())
            .field(&self.kind)
            .finish()
    }
}

impl Debug for TokenKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Opcode(opcode) => f.debug_tuple("Opcode").field(opcode).finish(),
            TokenKind::Integer(IntegerToken {
                value,
                sign,
                base,
                leading_zeros,
                errors,
            }) => f
                .debug_struct("Integer")
                .field("value", value)
                .field("sign", sign)
                .field("base", base)
                .field("leading_zeros", leading_zeros)
                .field("errors", errors)
                .finish(),
            TokenKind::Char { data, quotes } => f
                .debug_struct("Char")
                .field("data", data)
                .field("quotes", quotes)
                .finish(),
            TokenKind::String { data, quotes } => f
                .debug_struct("String")
                .field("data", data)
                .field("quotes", quotes)
                .finish(),
            TokenKind::Ident { sigil, ident } => f
                .debug_struct("Ident")
                .field("sigil", &sigil.as_bstr())
                .field("ident", &ident.as_bstr())
                .finish(),
            TokenKind::Label {
                sigil,
                label,
                errors,
            } => f
                .debug_struct("Label")
                .field("sigil", &sigil.as_bstr())
                .field("label", &label.as_bstr())
                .field("errors", errors)
                .finish(),
            TokenKind::LabelColon => write!(f, "LabelColon"),
            TokenKind::InstSep => write!(f, "InstSep"),
            TokenKind::ArgSep => write!(f, "ArgSep"),
            TokenKind::Space => write!(f, "Space"),
            TokenKind::LineTerm => write!(f, "LineTerm"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::LineComment {
                prefix,
                text,
                errors,
            } => f
                .debug_struct("LineComment")
                .field("prefix", &prefix.as_bstr())
                .field("text", &text.as_bstr())
                .field("errors", errors)
                .finish(),
            TokenKind::BlockComment {
                open,
                text,
                close,
                nested,
                terminated,
            } => f
                .debug_struct("BlockComment")
                .field("open", &open.as_bstr())
                .field("text", &text.as_bstr())
                .field("close", &close.as_bstr())
                .field("nested", nested)
                .field("terminated", terminated)
                .finish(),
            TokenKind::Word => write!(f, "Word"),
            TokenKind::Quoted { inner, quotes } => f
                .debug_struct("Quoted")
                .field("inner", inner)
                .field("quotes", quotes)
                .finish(),
            TokenKind::Spliced { tokens, spliced } => f
                .debug_struct("Spliced")
                .field("tokens", tokens)
                .field("spliced", spliced)
                .finish(),
            TokenKind::Error(err) => f.debug_tuple("Error").field(err).finish(),
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

impl Debug for StringData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StringData::Utf8(s) => f.debug_tuple("Utf8").field(s).finish(),
            StringData::Bytes(b) => f.debug_tuple("Bytes").field(&b.as_bstr()).finish(),
        }
    }
}
