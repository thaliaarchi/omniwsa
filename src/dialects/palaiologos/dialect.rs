//! Parsing for the Palaiologos Whitespace assembly dialect.

use crate::{
    dialects::{dialect::DialectState, palaiologos::parse::Parser, Dialect},
    syntax::{Cst, Opcode},
    tokens::{
        integer::{BaseStyle, DigitSep, Integer, IntegerSyntax, SignStyle},
        mnemonics::FoldedStr,
    },
};

/// Palaiologos Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct Palaiologos;

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

impl Dialect for Palaiologos {
    const MNEMONICS: &[(FoldedStr<'_>, &[Opcode])] = MNEMONICS;

    fn parse<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Cst<'s> {
        Parser::new(src, dialect).parse()
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
    fn make_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::Neg,
            base_styles: BaseStyle::Decimal
                | BaseStyle::BinSuffix_b
                | BaseStyle::BinSuffix_B
                | BaseStyle::OctSuffix_o
                | BaseStyle::OctSuffix_O
                | BaseStyle::HexSuffix_h
                | BaseStyle::HexSuffix_H,
            digit_sep: DigitSep::None,
            min_value: Some(Integer::from(i32::MIN)),
            max_value: Some(Integer::from(i32::MAX)),
        }
    }
}

pub(super) const MAX_MNEMONIC_LEN: usize = {
    let mut max_len = 0;
    let mut i = 0;
    while i < MNEMONICS.len() {
        let len = MNEMONICS[i].0.s.len();
        if len > max_len {
            max_len = len;
        }
        i += 1;
    }
    max_len
};
