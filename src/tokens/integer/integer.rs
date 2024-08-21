//! Integer literal.

use std::borrow::Cow;

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Pretty},
    tokens::integer::Integer,
};

// TODO:
// - Create a integer syntax description struct for dialects to construct, to
//   make parsing and conversions modular.

/// An integer literal token.
#[derive(Clone, DebugCustom, Default, PartialEq, Eq)]
pub struct IntegerToken<'s> {
    /// The literal integer including formatting.
    #[debug("{:?}", literal.as_bstr())]
    pub literal: Cow<'s, [u8]>,
    /// The parsed value represented by the integer literal.
    pub value: Integer,
    /// The sign of the integer literal.
    pub sign: IntegerSign,
    /// The base of the integer literal.
    pub base: IntegerBase,
    /// The number of leading zeros, excluding a base prefix, written in the
    /// integer literal.
    pub leading_zeros: usize,
    /// Whether the integer literal has any `_` digit separators.
    pub has_digit_sep: bool,
    /// All errors from parsing this integer literal. When any errors are
    /// present, the other fields are best-effort.
    pub errors: EnumSet<IntegerError>,
}

/// The sign of an integer literal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntegerSign {
    /// Implicit positive sign.
    #[default]
    None,
    /// Positive sign.
    Pos,
    /// Negative sign.
    Neg,
}

/// The base (radix) of an integer literal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegerBase {
    /// Base 2.
    Binary = 2,
    /// Base 8.
    Octal = 8,
    /// Base 10.
    #[default]
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
    /// Value out of range.
    Range,
    /// Has a sign that is invalid or not supported.
    InvalidSign,
    /// Has a base that is not supported.
    InvalidBase,
    /// Uses digit separators, which are not supported.
    InvalidDigitSep,
    /// An unpaired parenthesis (Burghard via Haskell `Integer`).
    UnpairedParen,
}

impl IntegerToken<'_> {
    /// Constructs a new, empty integer token.
    pub fn new() -> Self {
        IntegerToken::default()
    }
}

impl HasError for IntegerToken<'_> {
    fn has_error(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Pretty for IntegerToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.literal.pretty(buf);
    }
}
