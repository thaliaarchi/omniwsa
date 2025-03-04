//! Lexical tokens for interoperable Whitespace assembly.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Formatter},
};

use bstr::ByteSlice;
use derive_more::{Debug as DebugCustom, From};
use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Pretty},
    tokens::{
        comment::{BlockCommentToken, LineCommentToken},
        integer::IntegerToken,
        label::{LabelColonToken, LabelToken},
        mnemonics::MnemonicToken,
        spaces::{ArgSepToken, EofToken, InstSepToken, LineTermToken, SpaceToken, Spaces},
        string::{CharToken, StringToken},
    },
};

// TODO:
// - Store spans in tokens.
//   - `Spanned<T>` or `Loc<T>`.
//   - Use length-based spans, instead of offsets, like Rowan. Since the root
//     node has the length of the full file, it needs to be `u64`, not shorter.
// - Organization:
//   - Merge `Token` into CST.
//   - Remove Token suffix from type names. What about `StringToken`?
//   - Possibly reference enum variants as `cst::Variant` like `ty::FnDef(_, _)`
//     for `rustc_middle::ty::TyKind::FnDef(_, _)`.
// - Features:
//   - Whitelips, Lime, and Respace macro definitions.
//   - Respace `@define`.
// - Store byte string uniformly (named Text?), instead of a mix of &[u8] and
//   Cow.
//   - Create utilities for slicing and manipulating easier than Cow.
//   - Display it as conventionally UTF-8.

/// A lexical token, a unit of scanned text, in interoperable Whitespace
/// assembly.
#[derive(Clone, Default, PartialEq, Eq, From)]
pub enum Token<'s> {
    /// Instruction or predefined macro opcode.
    Mnemonic(MnemonicToken<'s>),
    /// Integer literal.
    Integer(IntegerToken<'s>),
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
    /// Horizontal whitespace.
    Space(SpaceToken<'s>),
    /// Line terminator.
    LineTerm(LineTermToken),
    /// End of file.
    Eof(EofToken),
    /// Instruction separator (e.g., Respace `;` or Palaiologos `/`).
    InstSep(InstSepToken),
    /// Argument separator (e.g., Palaiologos `,`).
    ArgSep(ArgSepToken),
    /// Line comment (e.g., `#` or `//`).
    LineComment(LineCommentToken<'s>),
    /// Block comment (e.g., `{- -}` or `/* */`).
    BlockComment(BlockCommentToken<'s>),
    /// A word of uninterpreted meaning.
    Word(WordToken<'s>),
    /// A token enclosed in parentheses or non-semantic quotes (Burghard).
    Group(GroupToken<'s>),
    /// Tokens spliced by block comments (Burghard).
    Splice(SpliceToken<'s>),
    /// An erroneous sequence.
    Error(ErrorToken<'s>),
    /// A placeholder variant for internal use.
    #[default]
    Placeholder,
}

/// Variable identifier token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct VariableToken<'s> {
    /// The identifier with its sigil removed.
    #[debug("{:?}", ident.as_bstr())]
    pub ident: Cow<'s, [u8]>,
    /// The style of this variable.
    pub style: VariableStyle,
}

/// The style of a variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariableStyle {
    /// `_` prefix sigil (Burghard).
    UnderscoreSigil,
}

/// A word token of uninterpreted meaning.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct WordToken<'s> {
    /// The text of the word.
    #[debug("{:?}", word.as_bstr())]
    pub word: Cow<'s, [u8]>,
    /// All errors from parsing this word.
    pub errors: EnumSet<WordError>,
}

/// A parse error for a word.
#[derive(EnumSetType, Debug)]
pub enum WordError {
    /// The word contains invalid UTF-8.
    InvalidUtf8,
}

/// A token enclosed in parentheses or non-semantic quotes (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupToken<'s> {
    /// The style of the delimiter enclosing this token.
    pub delim: GroupStyle,
    /// Spaces between the opening delimiter and the inner token.
    pub space_before: Spaces<'s>,
    /// The effective token.
    pub inner: Box<Token<'s>>,
    /// Spaces between the inner token and the closing delimiter.
    pub space_after: Spaces<'s>,
    /// All errors from parsing this token.
    pub errors: EnumSet<GroupError>,
}

/// The style of a non-semantic group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupStyle {
    /// Parentheses. Integers in the Burghard dialect may be wrapped in
    /// parentheses.
    Parens,
    /// `"`-quotes. Any word in the Burghard dialect may be wrapped in
    /// non-semantic quotes.
    DoubleQuotes,
}

/// A parse error for a group token.
#[derive(EnumSetType, Debug)]
pub enum GroupError {
    /// Has no closing delimiter.
    Unterminated,
}

/// Tokens spliced by block comments (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceToken<'s> {
    /// A list of words interspersed with block comments. Only contains
    /// `Word` and `BlockComment`.
    pub tokens: Vec<Token<'s>>,
    /// The effective token.
    pub spliced: Box<Token<'s>>,
}

/// A sequence that could not be lexed.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct ErrorToken<'s> {
    /// The unrecognized sequence.
    #[debug("{:?}", text.as_bstr())]
    pub text: Cow<'s, [u8]>,
}

impl<'s> Token<'s> {
    /// Unwraps non-semantic groups and splices.
    pub fn peel_groups(&self) -> &Token<'s> {
        let mut tok = self;
        while let Token::Group(GroupToken { inner, .. })
        | Token::Splice(SpliceToken { spliced: inner, .. }) = tok
        {
            tok = inner;
        }
        tok
    }

    /// Unwraps non-semantic groups and splices and returns a mutable reference.
    pub fn peel_groups_mut(&mut self) -> &mut Token<'s> {
        let mut tok = self;
        while let Token::Group(GroupToken { inner, .. })
        | Token::Splice(SpliceToken { spliced: inner, .. }) = tok
        {
            tok = inner;
        }
        tok
    }
}

impl VariableStyle {
    /// The prefix sigil.
    pub const fn sigil(&self) -> &'static str {
        match self {
            VariableStyle::UnderscoreSigil => "_",
        }
    }
}

impl GroupStyle {
    /// The opening delimiter.
    pub const fn open(&self) -> &'static str {
        match self {
            GroupStyle::Parens => "(",
            GroupStyle::DoubleQuotes => "\"",
        }
    }

    /// The closing delimiter.
    pub const fn close(&self) -> &'static str {
        match self {
            GroupStyle::Parens => ")",
            GroupStyle::DoubleQuotes => "\"",
        }
    }
}

impl<'s, T: Into<Cow<'s, [u8]>>> From<T> for ErrorToken<'s> {
    fn from(text: T) -> Self {
        ErrorToken { text: text.into() }
    }
}

impl HasError for Token<'_> {
    fn has_error(&self) -> bool {
        match self {
            Token::Mnemonic(m) => m.has_error(),
            Token::Integer(i) => i.has_error(),
            Token::String(s) => s.has_error(),
            Token::Char(c) => c.has_error(),
            Token::Variable(v) => v.has_error(),
            Token::Label(l) => l.has_error(),
            Token::LabelColon(l) => l.has_error(),
            Token::InstSep(i) => i.has_error(),
            Token::ArgSep(a) => a.has_error(),
            Token::Space(s) => s.has_error(),
            Token::LineTerm(l) => l.has_error(),
            Token::Eof(e) => e.has_error(),
            Token::LineComment(l) => l.has_error(),
            Token::BlockComment(b) => b.has_error(),
            Token::Word(w) => w.has_error(),
            Token::Group(g) => g.has_error(),
            Token::Splice(s) => s.has_error(),
            Token::Error(e) => e.has_error(),
            Token::Placeholder => panic!("placeholder"),
        }
    }
}

impl HasError for VariableToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for WordToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for GroupToken<'_> {
    fn has_error(&self) -> bool {
        self.space_before.has_error()
            || self.inner.has_error()
            || self.space_after.has_error()
            || !self.errors.is_empty()
    }
}

impl HasError for SpliceToken<'_> {
    fn has_error(&self) -> bool {
        self.tokens.iter().any(Token::has_error)
    }
}

impl HasError for ErrorToken<'_> {
    fn has_error(&self) -> bool {
        true
    }
}

impl Pretty for VariableToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.sigil().pretty(buf);
        self.ident.pretty(buf);
    }
}

impl Pretty for WordToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.word.pretty(buf);
    }
}

impl Pretty for GroupToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.delim.open().pretty(buf);
        self.inner.pretty(buf);
        if !self.errors.contains(GroupError::Unterminated) {
            self.delim.close().pretty(buf);
        }
    }
}

impl Pretty for SpliceToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.tokens.iter().for_each(|tok| tok.pretty(buf));
    }
}

impl Pretty for ErrorToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.text.pretty(buf)
    }
}

impl Debug for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Mnemonic(m) => Debug::fmt(m, f),
            Token::Integer(i) => Debug::fmt(i, f),
            Token::Char(c) => Debug::fmt(c, f),
            Token::String(s) => Debug::fmt(s, f),
            Token::Variable(v) => Debug::fmt(v, f),
            Token::Label(l) => Debug::fmt(l, f),
            Token::LabelColon(l) => Debug::fmt(l, f),
            Token::Space(s) => Debug::fmt(s, f),
            Token::LineTerm(l) => Debug::fmt(l, f),
            Token::Eof(e) => Debug::fmt(e, f),
            Token::InstSep(u) => Debug::fmt(u, f),
            Token::ArgSep(a) => Debug::fmt(a, f),
            Token::LineComment(l) => Debug::fmt(l, f),
            Token::BlockComment(b) => Debug::fmt(b, f),
            Token::Word(w) => Debug::fmt(w, f),
            Token::Group(g) => Debug::fmt(g, f),
            Token::Splice(s) => Debug::fmt(s, f),
            Token::Error(e) => Debug::fmt(e, f),
            Token::Placeholder => write!(f, "Placeholder"),
        }
    }
}
