//! Comment tokens.

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::syntax::{HasError, Pretty};

// TODO:
// - Block comments should be parsed into a list or even hierarchy of tokens.
//   - Line comments within block comments could be highlighted differently.

/// Line comment token (e.g., `#` or `//`).
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct LineCommentToken<'s> {
    /// The comment text after the prefix marker, including any leading spaces.
    #[debug("{:?}", text.as_bstr())]
    pub text: &'s [u8],
    /// The style of this line comment.
    pub style: LineCommentStyle,
    /// All errors from parsing this line comment.
    pub errors: EnumSet<LineCommentError>,
}

/// The style of a line comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineCommentStyle {
    /// `;` line comment (Burghard, Lime, rdebath, voliva, Whitelips).
    Semi,
    /// `#` line comment (rdebath, Respace, Whitelips).
    Hash,
    /// `//` line comment (Lime, littleBugHunter).
    SlashSlash,
    /// `--` line comment (Burghard, Whitelips).
    DashDash,
}

/// A parse error for a line comment.
#[derive(EnumSetType, Debug)]
pub enum LineCommentError {
    /// The line comment contains invalid UTF-8.
    InvalidUtf8,
}

/// Block comment token (e.g., `{- -}` or `/* */`).
/// Sequences ignored due to a bug in the reference parser also count as block
/// comments (e.g., voliva ignored arguments).
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct BlockCommentToken<'s> {
    /// The text contained within the comment delimiters, including any nested
    /// block comments.
    #[debug("{:?}", text.as_bstr())]
    pub text: &'s [u8],
    /// The style of this block comment.
    pub style: BlockCommentStyle,
    /// All errors from parsing this block comment.
    pub errors: EnumSet<BlockCommentError>,
}

/// The style of a block comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockCommentStyle {
    /// C-style `/* */` block comment (Lime).
    C,
    /// Haskell-style `{- -}` nested block comment (Whitelips).
    Haskell,
    /// Burghard-style `{- -}` nested block comment, where `--` and `;` line
    /// comments take precedence over block comment delimiters  (Burghard).
    Burghard,
}

/// A parse error for a block comment.
#[derive(EnumSetType, Debug)]
pub enum BlockCommentError {
    /// The block comment is not terminated by a closing delimiter.
    Unterminated,
    /// A closing delimiter has no paired opening delimiter.
    Unopened,
    /// The block comment contains invalid UTF-8.
    InvalidUtf8,
}

impl LineCommentStyle {
    /// The prefix marker (e.g., `#` or `//`).
    pub const fn prefix(&self) -> &'static str {
        match self {
            LineCommentStyle::Semi => ";",
            LineCommentStyle::Hash => "#",
            LineCommentStyle::SlashSlash => "//",
            LineCommentStyle::DashDash => "--",
        }
    }
}

impl LineCommentToken<'_> {
    /// Trims trailing whitespace characters in the line comment text.
    pub fn trim_trailing(&mut self) {
        let i = self
            .text
            .iter()
            .rposition(|&b| b != b' ' && b != b'\t')
            .map(|i| i + 1)
            .unwrap_or(0);
        self.text = &self.text[..i];
    }
}

impl BlockCommentStyle {
    /// Returns whether this style of block comment can contain nested block
    /// comments.
    pub const fn can_nest(&self) -> bool {
        match self {
            BlockCommentStyle::C => false,
            BlockCommentStyle::Haskell | BlockCommentStyle::Burghard => true,
        }
    }

    /// The opening delimiter (e.g., `{-` or `/*`).
    pub const fn open(&self) -> &'static str {
        match self {
            BlockCommentStyle::C => "/*",
            BlockCommentStyle::Haskell | BlockCommentStyle::Burghard => "{-",
        }
    }

    /// The closing delimiter (e.g., `-}` or `*/`).
    pub const fn close(&self) -> &'static str {
        match self {
            BlockCommentStyle::C => "*/",
            BlockCommentStyle::Haskell | BlockCommentStyle::Burghard => "-}",
        }
    }
}

impl HasError for LineCommentToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for BlockCommentToken<'_> {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Pretty for LineCommentToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.prefix().pretty(buf);
        self.text.pretty(buf);
    }
}

impl Pretty for BlockCommentToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        if !self.errors.contains(BlockCommentError::Unopened) {
            self.style.open().pretty(buf);
        }
        self.text.pretty(buf);
        if !self.errors.contains(BlockCommentError::Unterminated) {
            self.style.close().pretty(buf);
        }
    }
}
