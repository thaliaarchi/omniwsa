//! Comment tokens.

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::syntax::HasError;

/// Line comment token (e.g., `#` or `//`).
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct LineCommentToken<'s> {
    /// The prefix marker (e.g., `#` or `//`).
    #[debug("{:?}", prefix.as_bstr())]
    pub prefix: &'s [u8],
    /// The comment text after the marker, including any leading spaces.
    #[debug("{:?}", text.as_bstr())]
    pub text: &'s [u8],
    /// Errors for this line comment.
    pub errors: EnumSet<LineCommentError>,
}

/// A parse error for a line comment.
#[derive(EnumSetType, Debug)]
pub enum LineCommentError {
    /// The line comment is not terminated by a line feed (Palaiologos).
    MissingLf,
}

/// Block comment token (e.g., `{- -}` or `/* */`).
/// Sequences ignored due to a bug in the reference parser also count as block
/// comments (e.g., voliva ignored arguments).
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct BlockCommentToken<'s> {
    /// The opening marker (e.g., `{-` or `/*`).
    #[debug("{:?}", open.as_bstr())]
    pub open: &'s [u8],
    /// The text contained within the comment markers, including any nested
    /// block comments.
    #[debug("{:?}", text.as_bstr())]
    pub text: &'s [u8],
    /// The closing marker, or nothing if it is not terminated (e.g., `-}` or
    /// `*/`).
    #[debug("{:?}", close.as_bstr())]
    pub close: &'s [u8],
    /// Whether the kind of block comment allows nesting.
    pub nested: bool,
    /// Whether this block comment is correctly closed.
    pub terminated: bool,
}

impl HasError for LineCommentToken<'_> {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl HasError for BlockCommentToken<'_> {
    fn has_error(&self) -> bool {
        !self.terminated
    }
}
