//! Parsing for the Palaiologos Whitespace assembly dialect.

use crate::{
    dialects::palaiologos::parse::Parser,
    syntax::{Cst, Opcode},
    tokens::{
        integer::{Base, BaseStyle, DigitSep, Integer, IntegerSyntax, SignStyle},
        mnemonics::{FoldedStr, MnemonicMap},
    },
};

/// State for parsing the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Palaiologos {
    mnemonics: MnemonicMap,
    integers: IntegerSyntax,
}

macro_rules! mnemonics{($($mnemonic:literal => [$($opcode:ident),+],)+) => {
    &[$((FoldedStr::ascii($mnemonic), &[$(Opcode::$opcode),+])),+]
}}
static MNEMONICS: &[(FoldedStr<'_>, &[Opcode])] = mnemonics! {
    b"psh" => [Push, Push0],
    b"push" => [Push, Push0],
    b"dup" => [Dup],
    b"copy" => [Copy],
    b"take" => [Copy],
    b"pull" => [Copy],
    b"xchg" => [Swap],
    b"swp" => [Swap],
    b"swap" => [Swap],
    b"drop" => [Drop],
    b"dsc" => [Drop],
    b"slide" => [Slide],
    b"add" => [Add, AddConstRhs],
    b"sub" => [Sub, SubConstRhs],
    b"mul" => [Mul, MulConstRhs],
    b"div" => [Div, DivConstRhs],
    b"mod" => [Mod, ModConstRhs],
    b"sto" => [Store, StoreConstRhs, StoreConstConst],
    b"rcl" => [Retrieve, RetrieveConst],
    b"call" => [Call],
    b"gosub" => [Call],
    b"jsr" => [Call],
    b"jmp" => [Jmp],
    b"j" => [Jmp],
    b"b" => [Jmp],
    b"jz" => [Jz],
    b"bz" => [Jz],
    b"jltz" => [Jn],
    b"bltz" => [Jn],
    b"ret" => [Ret],
    b"end" => [End],
    b"putc" => [Printc, PrintcConst],
    b"putn" => [Printi, PrintiConst],
    b"getc" => [Readc, ReadcConst],
    b"getn" => [Readi, ReadiConst],
    b"rep" => [PalaiologosRep],
};

impl Palaiologos {
    pub(super) const MAX_MNEMONIC_LEN: usize = 5;

    /// Constructs state for the Palaiologos dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Palaiologos {
            mnemonics: MnemonicMap::from(MNEMONICS),
            integers: Self::new_integers(),
        }
    }

    /// Parses a Whitespace assembly program in the Palaiologos dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        Parser::new(src, &Palaiologos::new()).parse()
    }

    /// Gets the mnemonic map for this dialect.
    pub(super) fn mnemonics(&self) -> &MnemonicMap {
        &self.mnemonics
    }

    /// Gets the integer syntax description for this dialect.
    ///
    /// See [`Palaiologos::new_integers`] for the grammar.
    pub fn integers(&self) -> &IntegerSyntax {
        &self.integers
    }

    /// Constructs an integer syntax description for this dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::=
    ///     | "-"? [0-9]+
    ///     | "-"? [01]+ [bB]
    ///     | "-"? [0-7]+ [oO]
    ///     | "-"? [0-9] [0-9 a-f A-F]* [hH]
    /// ```
    ///
    /// In addition, `omniwsa` recognizes positive signs, hex literals starting
    /// with letters, and `_` digit separators, matching the following grammar.
    /// Any extensions are marked as errors.
    ///
    /// ```bnf
    /// integer ::=
    ///     | [-+]? [0-9 _]*
    ///     | [-+]? [01 _]* [bB]
    ///     | [-+]? [0-7 _]* [oO]
    ///     | [-+]? [0-9 a-f A-F _]* [hH]
    /// ```
    pub fn new_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::Neg,
            base_style: BaseStyle::Palaiologos,
            bases: Base::Decimal | Base::Binary | Base::Octal | Base::Hexadecimal,
            digit_sep: DigitSep::None,
            min_value: Some(Integer::from(i32::MIN)),
            max_value: Some(Integer::from(i32::MAX)),
        }
    }
}
