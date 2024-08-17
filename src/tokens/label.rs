//! Label tokens.

use std::borrow::Cow;

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::syntax::{HasError, Pretty};

/// Label token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct LabelToken<'s> {
    /// The label with its sigil removed.
    #[debug("{:?}", label.as_bstr())]
    pub label: Cow<'s, [u8]>,
    /// The style of this label.
    pub style: LabelStyle,
    /// All errors from parsing this label.
    pub errors: EnumSet<LabelError>,
}

/// The style of a label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelStyle {
    /// No sigil (Burghard).
    NoSigil,
    /// `@` prefix sigil (Palaiologos).
    AtSigil,
    /// `%` prefix sigil (Palaiologos).
    PercentSigil,
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

impl LabelStyle {
    /// The prefix sigil.
    pub fn sigil(&self) -> &'static str {
        match self {
            LabelStyle::NoSigil => "",
            LabelStyle::AtSigil => "@",
            LabelStyle::PercentSigil => "%",
        }
    }
}

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

impl Pretty for LabelToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.style.sigil().pretty(buf);
        self.label.pretty(buf);
    }
}

impl Pretty for LabelColonToken {
    fn pretty(&self, buf: &mut Vec<u8>) {
        ":".pretty(buf);
    }
}
