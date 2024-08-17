//! Lexical tokens for interoperable Whitespace assembly.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
};

use bstr::ByteSlice;
use derive_more::{Debug as DebugCustom, From};

use crate::{
    syntax::{HasError, Opcode},
    tokens::{
        comment::{BlockCommentToken, LineCommentToken},
        integer::IntegerToken,
        label::{LabelColonToken, LabelToken},
        spaces::{ArgSepToken, EofToken, InstSepToken, LineTermToken, SpaceToken},
        string::{CharToken, QuotedToken, StringToken},
    },
};

// TODO:
// - Whitelips, Lime, and Respace macro definitions.
// - Respace `@define`.
// - Store byte string uniformly, instead of a mix of &[u8] and Cow.
//   - Create utilities for slicing and manipulating easier than Cow.
//   - Display it as conventionally UTF-8.
// - Move `Token::text` into token variants, so text is not stored redundantly,
//   and rename `TokenKind` -> `Token`. For example, the line comment prefix
//   needs to be manipulated in both `Token::text` and `LineComment::prefix`.
// - Extract each token as a struct to manage manipulation routines.
// - Make UTF-8 error a first-class token.
// - Add LineTerm kind (i.e., LF, CRLF, CR).

/// A lexical token, a unit of scanned text, in interoperable Whitespace
/// assembly.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct Token<'s> {
    /// The raw text of this token.
    #[debug("{:?}", text.as_bstr())]
    pub text: Cow<'s, [u8]>,
    /// The data of this token, including its kind.
    pub kind: TokenKind<'s>,
}

/// A kind of token.
#[derive(Clone, PartialEq, Eq, From)]
pub enum TokenKind<'s> {
    /// Instruction or predefined macro opcode.
    Opcode(Opcode),
    /// Integer literal.
    Integer(IntegerToken),
    /// String literal.
    String(StringToken<'s>),
    /// Character literal.
    Char(CharToken<'s>),
    /// Variable identifier.
    Variable(VariableToken<'s>),
    /// Label.
    Label(LabelToken<'s>),
    /// Label colon marker (i.e., `:`).
    LabelColon(LabelColonToken),
    /// Instruction separator (e.g., Respace `;` or Palaiologos `/`).
    InstSep(InstSepToken),
    /// Argument separator (e.g., Palaiologos `,`).
    ArgSep(ArgSepToken),
    /// Horizontal whitespace.
    Space(SpaceToken),
    /// Line terminator.
    LineTerm(LineTermToken),
    /// End of file.
    Eof(EofToken),
    /// Line comment (e.g., `#` or `//`).
    LineComment(LineCommentToken<'s>),
    /// Block comment (e.g., `{- -}` or `/* */`).
    BlockComment(BlockCommentToken<'s>),
    /// A word of uninterpreted meaning.
    Word(WordToken),
    /// A token enclosed in non-semantic quotes (Burghard).
    Quoted(QuotedToken<'s>),
    /// Tokens spliced by block comments (Burghard).
    Spliced(SplicedToken<'s>),
    /// An erroneous sequence.
    Error(ErrorToken),
}

/// Variable identifier token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct VariableToken<'s> {
    /// A prefix sigil to mark identifiers (e.g., Burghard `_`).
    #[debug("{:?}", sigil.as_bstr())]
    pub sigil: &'s [u8],
    /// The identifier with its sigil removed.
    #[debug("{:?}", ident.as_bstr())]
    pub ident: Cow<'s, [u8]>,
}

/// A word token of uninterpreted meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordToken;

/// Tokens spliced by block comments (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SplicedToken<'s> {
    /// A list of words interspersed with block comments. Only contains
    /// `Word` and `BlockComment`.
    pub tokens: Vec<Token<'s>>,
    /// The effective token.
    pub spliced: Box<Token<'s>>,
}

/// A lexical error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorToken {
    /// Invalid UTF-8 sequence (Burghard).
    InvalidUtf8 {
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
                | TokenKind::Spliced(SplicedToken { spliced: inner, .. }) => {
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
                | TokenKind::Spliced(SplicedToken {
                    spliced: ref mut inner,
                    ..
                }) => {
                    tok = inner;
                }
                _ => return tok,
            }
        }
    }

    /// Trim trailing whitespace in a line comment.
    pub fn line_comment_trim_trailing(&mut self) {
        match &mut self.kind {
            TokenKind::LineComment(c) => {
                let i = c
                    .text
                    .iter()
                    .rposition(|&b| b != b' ' && b != b'\t')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                c.text = &c.text[..i];
                match &mut self.text {
                    Cow::Borrowed(text) => *text = &text[..c.style.prefix().len() + i],
                    Cow::Owned(text) => text.truncate(c.style.prefix().len() + i),
                }
            }
            _ => panic!("not a line comment"),
        }
    }
}

impl HasError for Token<'_> {
    fn has_error(&self) -> bool {
        match &self.kind {
            TokenKind::Opcode(o) => o.has_error(),
            TokenKind::Integer(i) => i.has_error(),
            TokenKind::String(s) => s.has_error(),
            TokenKind::Char(c) => c.has_error(),
            TokenKind::Variable(v) => v.has_error(),
            TokenKind::Label(l) => l.has_error(),
            TokenKind::LabelColon(l) => l.has_error(),
            TokenKind::InstSep(i) => i.has_error(),
            TokenKind::ArgSep(a) => a.has_error(),
            TokenKind::Space(s) => s.has_error(),
            TokenKind::LineTerm(l) => l.has_error(),
            TokenKind::Eof(e) => e.has_error(),
            TokenKind::LineComment(l) => l.has_error(),
            TokenKind::BlockComment(b) => b.has_error(),
            TokenKind::Word(w) => w.has_error(),
            TokenKind::Quoted(q) => q.has_error(),
            TokenKind::Spliced(s) => s.has_error(),
            TokenKind::Error(e) => e.has_error(),
        }
    }
}

impl HasError for Opcode {
    fn has_error(&self) -> bool {
        matches!(self, Opcode::Invalid)
    }
}
impl HasError for VariableToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}
impl HasError for WordToken {
    fn has_error(&self) -> bool {
        false
    }
}
impl HasError for SplicedToken<'_> {
    fn has_error(&self) -> bool {
        self.tokens.iter().any(Token::has_error)
    }
}
impl HasError for ErrorToken {
    fn has_error(&self) -> bool {
        true
    }
}

impl Debug for TokenKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Opcode(opcode) => f.debug_tuple("Opcode").field(opcode).finish(),
            TokenKind::Integer(i) => Debug::fmt(i, f),
            TokenKind::Char(c) => Debug::fmt(c, f),
            TokenKind::String(s) => Debug::fmt(s, f),
            TokenKind::Variable(v) => Debug::fmt(v, f),
            TokenKind::Label(l) => Debug::fmt(l, f),
            TokenKind::LabelColon(l) => Debug::fmt(l, f),
            TokenKind::InstSep(u) => Debug::fmt(u, f),
            TokenKind::ArgSep(a) => Debug::fmt(a, f),
            TokenKind::Space(s) => Debug::fmt(s, f),
            TokenKind::LineTerm(l) => Debug::fmt(l, f),
            TokenKind::Eof(e) => Debug::fmt(e, f),
            TokenKind::LineComment(l) => Debug::fmt(l, f),
            TokenKind::BlockComment(b) => Debug::fmt(b, f),
            TokenKind::Word(w) => Debug::fmt(w, f),
            TokenKind::Quoted(q) => Debug::fmt(q, f),
            TokenKind::Spliced(s) => Debug::fmt(s, f),
            TokenKind::Error(err) => f.debug_tuple("Error").field(err).finish(),
        }
    }
}
