//! Integer literal.

use std::borrow::Cow;

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{enum_set, EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Pretty},
    tokens::integer::Integer,
};

// TODO:
// - Convert between syntaxes.

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
    pub has_digit_seps: bool,
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
#[derive(Debug, Default, EnumSetType, PartialOrd, Ord)]
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
    /// Starts with a hex letter (Palaiologos).
    StartsWithHex,
    /// An unpaired parenthesis (Haskell).
    UnpairedParen,
}

/// A description of supported syntax features for integer literals, used to
/// convert between styles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerSyntax {
    /// The syntactic style family.
    pub style: IntegerStyle,
    /// Whether explicit positive sign is supported.
    pub explicit_pos: bool,
    /// The supported bases.
    pub bases: EnumSet<IntegerBase>,
    /// The supported digit separator.
    pub digit_sep: IntegerDigitSep,
    /// The minimum allowed integer value.
    pub min_value: Option<Integer>,
    /// The maximum allowed integer value.
    pub max_value: Option<Integer>,
}

/// A family of syntactically related integer styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerStyle {
    /// Haskell-style integers. See the extended syntax under
    /// [`IntegerSyntax::haskell`].
    Haskell,
    /// Palaiologos-style integers. See the extended syntax under
    /// [`IntegerSyntax::palaiologos`].
    Palaiologos,
}

/// A style of digit separator in an integer literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerDigitSep {
    /// No digit separators.
    None,
    /// `_` digit separators.
    Underscore,
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

impl IntegerSyntax {
    /// Integers with the syntax of [`read :: String -> Integer`](https://hackage.haskell.org/package/base/docs/GHC-Read.html)
    /// in Haskell.
    ///
    /// # Syntax
    ///
    /// Octal literals are prefixed with `0o` or `0O` and hexadecimal literals
    /// with `0x` or `0X`. Binary literals with `0b` or `0B` are not supported.
    /// A leading zero is interpreted as decimal, not octal. It may have a
    /// negative sign. It may be surrounded by any number of parentheses.
    /// Unicode whitespace characters may occur around the digits, sign, or
    /// parentheses. Positive signs, underscore digit separators, and exponents
    /// are not allowed.
    ///
    /// Haskell's `String` must be UTF-8 and excludes surrogate halves, so it is
    /// equivalent to Rust strings and validation happens outside of `read`.
    ///
    /// ```bnf
    /// read ::=
    ///     | space* "(" read ")" space*
    ///     | space* "-"? space* integer space*
    /// integer ::=
    ///     | [0-9]+
    ///     | "0" [oO] [0-7]+
    ///     | "0" [xX] [0-9 a-f A-F]+
    /// space ::= \p{White_Space} NOT (U+0085 | U+2028 | U+2029)
    /// ```
    ///
    /// In addition, `IntegerSyntax` recognizes positive signs, signs before
    /// parentheses, binary literals, and `_` digit separators, matching the
    /// following grammar. Any extensions are marked as errors.
    ///
    /// ```bnf
    /// read ::=
    ///     | space* sign* "(" read ")" space*
    ///     | space* sign* integer space*
    /// sign ::= ("-" | "+") space*
    /// integer ::=
    ///     | [0-9 _]*
    ///     | "0" [bB] [01 _]*
    ///     | "0" [oO] [0-7 _]*
    ///     | "0" [xX] [0-9 a-f A-F _]*
    /// space ::= \p{White_Space} NOT (U+0085 | U+2028 | U+2029)
    /// ```
    ///
    /// # Compliance
    ///
    /// It has been tested to match the behavior of at least GHC 8.8.4 and 9.4.4
    /// and matches the source of GHC 9.8.1 by inspection.
    pub const fn haskell() -> Self {
        IntegerSyntax {
            style: IntegerStyle::Haskell,
            explicit_pos: false,
            bases: enum_set!(IntegerBase::Decimal | IntegerBase::Octal | IntegerBase::Hexadecimal),
            digit_sep: IntegerDigitSep::None,
            min_value: None,
            max_value: None,
        }
    }

    /// Integers with the syntax of the Palaiologos Whitespace assembly dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::=
    ///     | "-"? [0-9]+
    ///     | "-"? [01]+ [bB]
    ///     | "-"? [0-9] [0-9 a-f A-F]* [hH]
    /// ```
    ///
    /// In addition, `IntegerSyntax` recognizes positive signs, octal literals,
    /// hex literals starting with letters, and `_` digit separators, matching
    /// the following grammar. Any extensions are marked as errors.
    ///
    /// ```bnf
    /// integer ::=
    ///     | [-+]? [0-9 _]*
    ///     | [-+]? [01 _]* [bB]
    ///     | [-+]? [0-7 _]* [oO]
    ///     | [-+]? [0-9 a-f A-F _]* [hH]
    /// ```
    pub fn palaiologos() -> Self {
        IntegerSyntax {
            style: IntegerStyle::Palaiologos,
            explicit_pos: false,
            bases: IntegerBase::Decimal | IntegerBase::Binary | IntegerBase::Hexadecimal,
            digit_sep: IntegerDigitSep::None,
            min_value: Some(Integer::from(i32::MIN + 1)),
            max_value: Some(Integer::from(i32::MAX)),
        }
    }
}
