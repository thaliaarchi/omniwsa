//! Lexical tokens for interoperable Whitespace assembly.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
};

use bstr::ByteSlice;
use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Opcode},
    tokens::{
        integer::IntegerToken,
        string::{CharToken, QuotedToken, StringToken},
    },
};

// TODO:
// - Whitelips, Lime, and Respace macro definitions.
// - Respace `@define`.
// - Store byte string uniformly, instead of a mix of &[u8] and Cow.
//   - Create utilities for slicing and manipulating easier than Cow.
// - Move `Token::text` into token variants, so text is not stored redundantly,
//   and rename `TokenKind` -> `Token`. For example, the line comment prefix
//   needs to be manipulated in both `Token::text` and `LineComment::prefix`.
// - Extract each token as a struct to manage manipulation routines.
// - Make UTF-8 error a first-class token.

/// A lexical token, a unit of scanned text, in interoperable Whitespace
/// assembly.
#[derive(Clone, PartialEq, Eq)]
pub struct Token<'s> {
    /// The raw text of this token.
    pub text: Cow<'s, [u8]>,
    /// The data of this token, including its kind.
    pub kind: TokenKind<'s>,
}

/// A kind of token.
#[derive(Clone, PartialEq, Eq)]
pub enum TokenKind<'s> {
    /// Instruction or predefined macro opcode.
    Opcode(Opcode),
    /// Integer literal.
    Integer(IntegerToken),
    /// String literal.
    String(StringToken<'s>),
    /// Character literal.
    Char(CharToken),
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
    /// Line comment (e.g., `#` or `//`).
    LineComment {
        /// The prefix marker (e.g., `#` or `//`).
        prefix: &'s [u8],
        /// The comment text after the marker, including any leading spaces.
        text: &'s [u8],
        /// Errors for this line comment.
        errors: EnumSet<LineCommentError>,
    },
    /// Block comment (e.g., `{- -}` or `/* */`).
    /// Sequences ignored due to a bug in the reference parser also count as
    /// block comments (e.g., voliva ignored arguments).
    BlockComment {
        /// The opening marker (e.g., `{-` or `/*`).
        open: &'s [u8],
        /// The text contained within the comment markers, including any nested
        /// block comments.
        text: &'s [u8],
        /// The closing marker, or nothing if it is not terminated (e.g., `-}`
        /// or `*/`).
        close: &'s [u8],
        /// Whether the kind of block comment allows nesting.
        nested: bool,
        /// Whether this block comment is correctly closed.
        terminated: bool,
    },
    /// A word of uninterpreted meaning.
    Word,
    /// A token enclosed in non-semantic quotes (Burghard).
    Quoted(QuotedToken<'s>),
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
    /// The line comment is not terminated by a line feed (Palaiologos).
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
    pub fn new<S: Into<Cow<'s, [u8]>>, T: Into<TokenKind<'s>>>(text: S, kind: T) -> Self {
        Token {
            text: text.into(),
            kind: kind.into(),
        }
    }

    /// Unwrap non-semantic splices and quotes.
    pub fn unwrap(&self) -> &Token<'s> {
        let mut tok = self;
        loop {
            match &tok.kind {
                TokenKind::Quoted(QuotedToken { inner, .. })
                | TokenKind::Spliced { spliced: inner, .. } => {
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
                TokenKind::Quoted(QuotedToken { ref mut inner, .. })
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

impl HasError for Token<'_> {
    fn has_error(&self) -> bool {
        match &self.kind {
            TokenKind::Opcode(Opcode::Invalid) | TokenKind::Error(_) => true,
            TokenKind::Char(c) => c.data.has_error() || c.quotes.has_error(),
            TokenKind::String(s) => s.quotes.has_error(),
            TokenKind::LineComment { errors, .. } => !errors.is_empty(),
            TokenKind::BlockComment { terminated, .. } => !terminated,
            TokenKind::Quoted(q) => q.inner.has_error() || q.quotes.has_error(),
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

impl From<Opcode> for TokenKind<'_> {
    fn from(opcode: Opcode) -> Self {
        TokenKind::Opcode(opcode)
    }
}
impl From<IntegerToken> for TokenKind<'_> {
    fn from(int: IntegerToken) -> Self {
        TokenKind::Integer(int)
    }
}
impl<'s> From<StringToken<'s>> for TokenKind<'s> {
    fn from(s: StringToken<'s>) -> Self {
        TokenKind::String(s)
    }
}
impl From<CharToken> for TokenKind<'_> {
    fn from(c: CharToken) -> Self {
        TokenKind::Char(c)
    }
}
impl<'s> From<QuotedToken<'s>> for TokenKind<'s> {
    fn from(quoted: QuotedToken<'s>) -> Self {
        TokenKind::Quoted(quoted)
    }
}
impl From<TokenError> for TokenKind<'_> {
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
            TokenKind::Integer(i) => Debug::fmt(i, f),
            TokenKind::Char(c) => Debug::fmt(c, f),
            TokenKind::String(s) => Debug::fmt(s, f),
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
            TokenKind::Quoted(q) => Debug::fmt(q, f),
            TokenKind::Spliced { tokens, spliced } => f
                .debug_struct("Spliced")
                .field("tokens", tokens)
                .field("spliced", spliced)
                .finish(),
            TokenKind::Error(err) => f.debug_tuple("Error").field(err).finish(),
        }
    }
}
