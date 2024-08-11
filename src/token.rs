//! Lexical tokens for interoperable Whitespace assembly.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
};

use bstr::ByteSlice;
use rug::Integer;

pub use crate::mnemonics::Mnemonic;

// TODO:
// - Whitelips, Lime, and Respace macro definitions.
// - Respace `@define`.
// - How to represent escapes in strings and chars?
// - How to represent equivalent integers?

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
    /// Instruction or predefined macro mnemonic.
    Mnemonic(Mnemonic),
    /// Integer.
    Integer {
        value: Integer,
        sign: IntegerSign,
        base: IntegerBase,
    },
    /// Character.
    Char { value: char, terminated: bool },
    /// String.
    String {
        unquoted: Cow<'s, [u8]>,
        kind: StringKind,
        terminated: bool,
    },
    /// Identifier.
    Ident {
        /// A prefix sigil to mark identifiers (e.g., Burghard `_`).
        sigil: &'s [u8],
        /// The identifier with its sigil removed.
        ident: &'s [u8],
    },
    /// Label colon marker (i.e., `:`).
    LabelColon,
    /// Label definition.
    LabelDef {
        /// A prefix sigil to mark label definitions (e.g., Palaiologos `@`).
        sigil: &'s [u8],
        /// The label with its sigil removed.
        label: &'s [u8],
    },
    /// Label reference.
    LabelRef {
        /// A prefix sigil to mark label references (e.g., Palaiologos `%`).
        sigil: &'s [u8],
        /// The label with its sigil removed.
        label: &'s [u8],
    },
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
    LineComment { prefix: &'s [u8], text: &'s [u8] },
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
        terminated: bool,
    },
    /// Tokens spliced by block comments (Burghard).
    Spliced {
        tokens: Vec<Token<'s>>,
        /// The effective token.
        spliced: Box<Token<'s>>,
    },
    /// An erroneous sequence.
    Error(TokenError),
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

/// The base of an integer literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerBase {
    /// Base 10.
    Decimal,
    /// Base 2.
    Binary,
    /// Base 8.
    Octal,
    /// Base 16.
    Hex,
}

/// The style of a string literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringKind {
    /// A string enclosed in quotes (Burghard).
    Quoted,
    /// A string not enclosed in quotes (Burghard).
    Unquoted,
}

/// A lexical error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenError {
    /// Invalid UTF-8 sequence (Burghard).
    InvalidUtf8 {
        /// Length of the invalid sequence.
        error_len: usize,
    },
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

    /// Returns whether the token is invalid.
    pub fn is_error(&self) -> bool {
        match &self.kind {
            TokenKind::Mnemonic(Mnemonic::Error) | TokenKind::Error(_) => true,
            TokenKind::Char { terminated, .. }
            | TokenKind::String { terminated, .. }
            | TokenKind::BlockComment { terminated, .. } => !terminated,
            TokenKind::Quoted {
                inner, terminated, ..
            } => !terminated || inner.is_error(),
            TokenKind::Spliced { tokens, .. } => tokens.iter().any(Token::is_error),
            _ => false,
        }
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
            TokenKind::Mnemonic(mnemonic) => f.debug_tuple("Mnemonic").field(mnemonic).finish(),
            TokenKind::Integer { value, sign, base } => f
                .debug_struct("Integer")
                .field("value", value)
                .field("sign", sign)
                .field("base", base)
                .finish(),
            TokenKind::Char { value, terminated } => f
                .debug_struct("Char")
                .field("value", value)
                .field("terminated", terminated)
                .finish(),
            TokenKind::String {
                unquoted,
                kind,
                terminated,
            } => f
                .debug_struct("String")
                .field("unquoted", &unquoted.as_bstr())
                .field("kind", kind)
                .field("terminated", terminated)
                .finish(),
            TokenKind::Ident { sigil, ident } => f
                .debug_struct("Ident")
                .field("sigil", &sigil.as_bstr())
                .field("ident", &ident.as_bstr())
                .finish(),
            TokenKind::LabelColon => write!(f, "LabelColon"),
            TokenKind::LabelDef { sigil, label } => f
                .debug_struct("LabelDef")
                .field("sigil", &sigil.as_bstr())
                .field("label", &label.as_bstr())
                .finish(),
            TokenKind::LabelRef { sigil, label } => f
                .debug_struct("LabelRef")
                .field("sigil", &sigil.as_bstr())
                .field("label", &label.as_bstr())
                .finish(),
            TokenKind::InstSep => write!(f, "InstSep"),
            TokenKind::ArgSep => write!(f, "ArgSep"),
            TokenKind::Space => write!(f, "Space"),
            TokenKind::LineTerm => write!(f, "LineTerm"),
            TokenKind::Eof => write!(f, "Eof"),
            TokenKind::LineComment { prefix, text } => f
                .debug_struct("LineComment")
                .field("prefix", &prefix.as_bstr())
                .field("text", &text.as_bstr())
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
            TokenKind::Quoted { inner, terminated } => f
                .debug_struct("Quoted")
                .field("inner", inner)
                .field("terminated", terminated)
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
