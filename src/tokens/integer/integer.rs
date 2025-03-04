//! Integer literal.

use std::borrow::Cow;

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;
use enumset::{EnumSet, EnumSetType, enum_set};

use crate::{
    syntax::{HasError, Pretty},
    tokens::integer::Integer,
};

/// An integer literal token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct IntegerToken<'s> {
    /// The literal integer including formatting.
    #[debug("{:?}", literal.as_bstr())]
    pub literal: Cow<'s, [u8]>,
    /// The parsed value represented by the integer literal.
    pub value: Integer,
    /// The sign of the integer literal.
    pub sign: Sign,
    /// The lexical style of the base.
    pub base_style: BaseStyle,
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
pub enum Sign {
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
pub enum Base {
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
    /// No digits, excluding a base prefix.
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
    /// The style of the sign.
    pub sign_style: SignStyle,
    /// The set of valid base styles.
    pub base_styles: EnumSet<BaseStyle>,
    /// The supported digit separator.
    pub digit_sep: DigitSep,
    /// The minimum allowed integer value.
    pub min_value: Option<Integer>,
    /// The maximum allowed integer value.
    pub max_value: Option<Integer>,
}

/// The lexical style of an integer sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignStyle {
    /// Implicit positive and '-' negative.
    Neg,
    /// Implicit positive, '-' negative, and '+' positive.
    NegPos,
    /// Implicit positive and '-' negative, with optional grouping parentheses.
    Haskell,
}

/// The lexical style of an integer base.
#[allow(non_camel_case_types)]
#[derive(Debug, EnumSetType)]
pub enum BaseStyle {
    /// A decimal integer with no prefix or suffix.
    Decimal,
    /// A binary integer with a `0b` prefix.
    BinPrefix_0b,
    /// A binary integer with a `0B` prefix.
    BinPrefix_0B,
    /// A binary integer with a `b` suffix.
    BinSuffix_b,
    /// A binary integer with a `B` suffix.
    BinSuffix_B,
    /// An octal integer with a `0o` prefix.
    OctPrefix_0o,
    /// An octal integer with a `0O` prefix.
    OctPrefix_0O,
    /// An octal integer with a `0` prefix.
    OctPrefix_0,
    /// An octal integer with an `o` suffix.
    OctSuffix_o,
    /// An octal integer with an `O` suffix.
    OctSuffix_O,
    /// A hexadecimal integer with a `0x` prefix.
    HexPrefix_0x,
    /// A hexadecimal integer with a `0X` prefix.
    HexPrefix_0X,
    /// A hexadecimal integer with an `h` suffix.
    HexSuffix_h,
    /// A hexadecimal integer with an `H` suffix.
    HexSuffix_H,
}

/// A style of digit separator in an integer literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigitSep {
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

impl Sign {
    /// The string representation of this sign.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Sign::None => "",
            Sign::Pos => "+",
            Sign::Neg => "-",
        }
    }
}

impl Base {
    /// The base styles which represent this base.
    pub const fn styles(&self) -> EnumSet<BaseStyle> {
        use BaseStyle::*;
        match self {
            Base::Binary => enum_set!(BinPrefix_0b | BinPrefix_0B | BinSuffix_b | BinSuffix_B),
            Base::Octal => {
                enum_set!(OctPrefix_0o | OctPrefix_0O | OctPrefix_0 | OctSuffix_o | OctSuffix_O)
            }
            Base::Decimal => enum_set!(Decimal),
            Base::Hexadecimal => enum_set!(HexPrefix_0x | HexPrefix_0X | HexSuffix_h | HexSuffix_H),
        }
    }
}

impl BaseStyle {
    /// The prefix marker for this base, or `""` if it has none.
    pub const fn prefix(&self) -> &'static str {
        match self {
            BaseStyle::BinPrefix_0b => "0b",
            BaseStyle::BinPrefix_0B => "0B",
            BaseStyle::OctPrefix_0o => "0o",
            BaseStyle::OctPrefix_0O => "0O",
            BaseStyle::OctPrefix_0 => "0",
            BaseStyle::HexPrefix_0x => "0x",
            BaseStyle::HexPrefix_0X => "0X",
            BaseStyle::Decimal
            | BaseStyle::BinSuffix_b
            | BaseStyle::BinSuffix_B
            | BaseStyle::OctSuffix_o
            | BaseStyle::OctSuffix_O
            | BaseStyle::HexSuffix_h
            | BaseStyle::HexSuffix_H => "",
        }
    }

    /// The suffix marker for this base, or `""` if it has none.
    pub const fn suffix(&self) -> &'static str {
        match self {
            BaseStyle::BinSuffix_b => "b",
            BaseStyle::BinSuffix_B => "B",
            BaseStyle::OctSuffix_o => "o",
            BaseStyle::OctSuffix_O => "O",
            BaseStyle::HexSuffix_h => "h",
            BaseStyle::HexSuffix_H => "H",
            BaseStyle::Decimal
            | BaseStyle::BinPrefix_0b
            | BaseStyle::BinPrefix_0B
            | BaseStyle::OctPrefix_0o
            | BaseStyle::OctPrefix_0O
            | BaseStyle::OctPrefix_0
            | BaseStyle::HexPrefix_0x
            | BaseStyle::HexPrefix_0X => "",
        }
    }

    /// The base this style represents.
    pub const fn base(&self) -> Base {
        match self {
            BaseStyle::Decimal => Base::Decimal,
            BaseStyle::BinPrefix_0b
            | BaseStyle::BinPrefix_0B
            | BaseStyle::BinSuffix_b
            | BaseStyle::BinSuffix_B => Base::Binary,
            BaseStyle::OctPrefix_0o
            | BaseStyle::OctPrefix_0O
            | BaseStyle::OctPrefix_0
            | BaseStyle::OctSuffix_o
            | BaseStyle::OctSuffix_O => Base::Octal,
            BaseStyle::HexPrefix_0x
            | BaseStyle::HexPrefix_0X
            | BaseStyle::HexSuffix_h
            | BaseStyle::HexSuffix_H => Base::Hexadecimal,
        }
    }

    /// The family of C-like base prefixes: `0x`/`0X` for hexadecimal, `0b`/`0B`
    /// for binary, `0` for octal, and no prefix for decimal.
    pub const fn c_family() -> EnumSet<Self> {
        enum_set!(
            BaseStyle::Decimal
                | BaseStyle::BinPrefix_0b
                | BaseStyle::BinPrefix_0B
                | BaseStyle::OctPrefix_0
                | BaseStyle::HexPrefix_0x
                | BaseStyle::HexPrefix_0X
        )
    }

    /// The family of Rust-like base prefixes: `0x`/`0X` for hexadecimal,
    /// `0b`/`0B` for binary, `0o`/`0O` for octal, and no prefix for decimal.
    pub const fn rust_family() -> EnumSet<Self> {
        enum_set!(
            BaseStyle::Decimal
                | BaseStyle::BinPrefix_0b
                | BaseStyle::BinPrefix_0B
                | BaseStyle::OctPrefix_0o
                | BaseStyle::OctPrefix_0O
                | BaseStyle::HexPrefix_0x
                | BaseStyle::HexPrefix_0X
        )
    }

    /// The family of base prefixes.
    pub const fn prefix_family() -> EnumSet<Self> {
        enum_set!(
            BaseStyle::BinPrefix_0b
                | BaseStyle::BinPrefix_0B
                | BaseStyle::OctPrefix_0o
                | BaseStyle::OctPrefix_0O
                | BaseStyle::OctPrefix_0
                | BaseStyle::HexPrefix_0x
                | BaseStyle::HexPrefix_0X
        )
    }

    /// The family of base suffixes.
    pub const fn suffix_family() -> EnumSet<Self> {
        enum_set!(
            BaseStyle::BinSuffix_b
                | BaseStyle::BinSuffix_B
                | BaseStyle::OctSuffix_o
                | BaseStyle::OctSuffix_O
                | BaseStyle::HexSuffix_h
                | BaseStyle::HexSuffix_H
        )
    }

    /// The bases represented in these base styles.
    pub fn bases(styles: EnumSet<BaseStyle>) -> EnumSet<Base> {
        const DECIMAL: EnumSet<BaseStyle> = Base::Decimal.styles();
        const BINARY: EnumSet<BaseStyle> = Base::Binary.styles();
        const OCTAL: EnumSet<BaseStyle> = Base::Octal.styles();
        const HEXADECIMAL: EnumSet<BaseStyle> = Base::Hexadecimal.styles();
        let mut bases = EnumSet::empty();
        if !(styles & DECIMAL).is_empty() {
            bases |= Base::Decimal;
        }
        if !(styles & BINARY).is_empty() {
            bases |= Base::Binary;
        }
        if !(styles & OCTAL).is_empty() {
            bases |= Base::Octal;
        }
        if !(styles & HEXADECIMAL).is_empty() {
            bases |= Base::Hexadecimal;
        }
        bases
    }

    /// Folds this base style to its lowercase equivalent.
    pub const fn to_lower(&self) -> Self {
        match self {
            BaseStyle::Decimal => BaseStyle::Decimal,
            BaseStyle::BinPrefix_0b | BaseStyle::BinPrefix_0B => BaseStyle::BinPrefix_0b,
            BaseStyle::BinSuffix_b | BaseStyle::BinSuffix_B => BaseStyle::BinSuffix_b,
            BaseStyle::OctPrefix_0 => BaseStyle::OctPrefix_0,
            BaseStyle::OctPrefix_0o | BaseStyle::OctPrefix_0O => BaseStyle::OctPrefix_0o,
            BaseStyle::OctSuffix_o | BaseStyle::OctSuffix_O => BaseStyle::OctSuffix_o,
            BaseStyle::HexPrefix_0x | BaseStyle::HexPrefix_0X => BaseStyle::HexPrefix_0x,
            BaseStyle::HexSuffix_h | BaseStyle::HexSuffix_H => BaseStyle::HexSuffix_h,
        }
    }

    /// Folds this base style to its uppercase equivalent.
    pub const fn to_upper(&self) -> Self {
        match self {
            BaseStyle::Decimal => BaseStyle::Decimal,
            BaseStyle::BinPrefix_0b | BaseStyle::BinPrefix_0B => BaseStyle::BinPrefix_0B,
            BaseStyle::BinSuffix_b | BaseStyle::BinSuffix_B => BaseStyle::BinSuffix_B,
            BaseStyle::OctPrefix_0 => BaseStyle::OctPrefix_0,
            BaseStyle::OctPrefix_0o | BaseStyle::OctPrefix_0O => BaseStyle::OctPrefix_0O,
            BaseStyle::OctSuffix_o | BaseStyle::OctSuffix_O => BaseStyle::OctSuffix_O,
            BaseStyle::HexPrefix_0x | BaseStyle::HexPrefix_0X => BaseStyle::HexPrefix_0X,
            BaseStyle::HexSuffix_h | BaseStyle::HexSuffix_H => BaseStyle::HexSuffix_H,
        }
    }

    /// Folds this base style to its prefix equivalent.
    pub const fn to_prefix(&self) -> Self {
        match self {
            BaseStyle::BinSuffix_b => BaseStyle::BinPrefix_0b,
            BaseStyle::BinSuffix_B => BaseStyle::BinPrefix_0B,
            BaseStyle::OctSuffix_o => BaseStyle::OctPrefix_0o,
            BaseStyle::OctSuffix_O => BaseStyle::OctPrefix_0O,
            BaseStyle::HexSuffix_h => BaseStyle::HexPrefix_0x,
            BaseStyle::HexSuffix_H => BaseStyle::HexPrefix_0X,
            BaseStyle::Decimal
            | BaseStyle::BinPrefix_0b
            | BaseStyle::BinPrefix_0B
            | BaseStyle::OctPrefix_0o
            | BaseStyle::OctPrefix_0O
            | BaseStyle::OctPrefix_0
            | BaseStyle::HexPrefix_0x
            | BaseStyle::HexPrefix_0X => *self,
        }
    }

    /// Folds this base style to its suffix equivalent.
    pub const fn to_suffix(&self) -> Self {
        match self {
            BaseStyle::BinPrefix_0b => BaseStyle::BinSuffix_b,
            BaseStyle::BinPrefix_0B => BaseStyle::BinSuffix_B,
            BaseStyle::OctPrefix_0o | BaseStyle::OctPrefix_0 => BaseStyle::OctSuffix_o,
            BaseStyle::OctPrefix_0O => BaseStyle::OctSuffix_O,
            BaseStyle::HexPrefix_0x => BaseStyle::HexSuffix_h,
            BaseStyle::HexPrefix_0X => BaseStyle::HexSuffix_H,
            BaseStyle::Decimal
            | BaseStyle::BinSuffix_b
            | BaseStyle::BinSuffix_B
            | BaseStyle::OctSuffix_o
            | BaseStyle::OctSuffix_O
            | BaseStyle::HexSuffix_h
            | BaseStyle::HexSuffix_H => *self,
        }
    }

    /// Returns the nearest style to this in the set of styles.
    pub fn to_nearest(&self, styles: EnumSet<BaseStyle>) -> BaseStyle {
        if styles.contains(*self) || styles.is_empty() {
            return *self;
        }
        let lower = self.to_lower();
        if styles.contains(lower) {
            return lower;
        }
        let upper = self.to_upper();
        if styles.contains(upper) {
            return upper;
        }
        let prefix = self.to_prefix();
        if styles.contains(prefix) {
            return prefix;
        }
        let suffix = self.to_suffix();
        if styles.contains(suffix) {
            return suffix;
        }
        if let Some(style) = (self.base().styles() & styles).iter().next() {
            return style;
        }
        if let Some(style) = (Base::Hexadecimal.styles() & styles).iter().next() {
            return style;
        }
        if let Some(style) = (Base::Decimal.styles() & styles).iter().next() {
            return style;
        }
        if let Some(style) = (Base::Binary.styles() & styles).iter().next() {
            return style;
        }
        if let Some(style) = (Base::Octal.styles() & styles).iter().next() {
            return style;
        }
        panic!("no base matched");
    }
}
