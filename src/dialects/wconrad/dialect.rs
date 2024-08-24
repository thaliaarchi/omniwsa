//! Parsing for the wconrad Whitespace assembly dialect.

use enumset::enum_set;

use crate::{
    syntax::Opcode,
    tokens::{
        integer::{Base, BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        mnemonics::{FoldedStr, MnemonicMap},
    },
};

// TODO:
// - Create a generic word scanner, which splits by Unicode spaces. Further
//   validation is then done by the dialect.
// - Create classes of integers, so that numbers can have signs, but labels
//   can't.

/// State for parsing the wconrad Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct WConrad {
    mnemonics: MnemonicMap,
    integers: IntegerSyntax,
}

macro_rules! mnemonics{($($mnemonic:literal => [$($opcode:ident),+],)+) => {
    &[$((FoldedStr::exact($mnemonic), &[$(Opcode::$opcode),+])),+]
}}
static MNEMONICS: &[(FoldedStr<'_>, &[Opcode])] = mnemonics! {
    b"push" => [Push],
    b"dup" => [Dup],
    b"copy" => [Copy],
    b"swap" => [Swap],
    b"discard" => [Drop],
    b"slide" => [Slide],
    b"add" => [Add],
    b"sub" => [Sub],
    b"mul" => [Mul],
    b"div" => [Div],
    b"mod" => [Mod],
    b"store" => [Store],
    b"retrieve" => [Retrieve],
    b"label" => [Label],
    b"call" => [Call],
    b"jump" => [Jmp],
    b"jz" => [Jz],
    b"jn" => [Jn],
    b"ret" => [Ret],
    b"exit" => [End],
    b"outchar" => [Printc],
    b"outnum" => [Printi],
    b"readchar" => [Readc],
    b"readnum" => [Readi],
};

impl WConrad {
    /// Constructs state for the wconrad dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        WConrad {
            mnemonics: MnemonicMap::from(MNEMONICS),
            integers: Self::new_integers(),
        }
    }

    /// Gets the integer syntax description for this dialect.
    ///
    /// See [`WConrad::new_integers`] for the grammar.
    pub fn integers(&self) -> &IntegerSyntax {
        &self.integers
    }

    /// Gets the integer syntax description for this dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::= [-+]? [0-9]+
    /// ```
    ///
    /// wconrad has no bases, so choose a style close to Ruby, but without `0`
    /// octal. Ruby has `0x`/`0X` for hexadecimal, `0`/`0o`/`0O` for octal,
    /// `0b`/`0B` for binary, and `0d`/`0D` or bare for decimal.
    pub const fn new_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::NegPos,
            base_style: BaseStyle::Rust,
            bases: enum_set!(Base::Decimal),
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
