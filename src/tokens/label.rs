//! Label tokens.

use std::borrow::Cow;

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::syntax::HasError;

/// Label token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct LabelToken<'s> {
    /// A prefix sigil to mark labels (e.g., Palaiologos `@` and `%`).
    #[debug("{:?}", sigil.as_bstr())]
    pub sigil: &'s [u8],
    /// The label with its sigil removed.
    #[debug("{:?}", label.as_bstr())]
    pub label: Cow<'s, [u8]>,
    /// All errors from parsing this label.
    pub errors: EnumSet<LabelError>,
}

/// A parse error for a label.
#[derive(EnumSetType, Debug)]
pub enum LabelError {
    /// The label has no characters (Palaiologos).
    Empty,
    /// The first character is a digit, which is not allowed (Palaiologos).
    StartsWithDigit,
}

/// Label colon marker token (i.e., `:`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabelColonToken;

impl HasError for LabelToken<'_> {
    fn has_error(&self) -> bool {
        false
    }
}

impl HasError for LabelColonToken {
    fn has_error(&self) -> bool {
        false
    }
}
