//! Parsing for the CensoredUsername Whitespace assembly dialect.

use enumset::enum_set;

use crate::{
    dialects::censoredusername::lex::Lexer,
    lex::Lex,
    syntax::Opcode,
    tokens::{
        integer::{Base, BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        mnemonics::{FoldedStr, MnemonicMap},
        Token,
    },
};

/// State for parsing the CensoredUsername Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct CensoredUsername {
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
    b"pop" => [Drop],
    b"slide" => [Slide],
    b"add" => [Add],
    b"sub" => [Sub],
    b"mul" => [Mul],
    b"div" => [Div],
    b"mod" => [Mod],
    b"set" => [Store],
    b"get" => [Retrieve],
    b"lbl" => [Label],
    b"call" => [Call],
    b"jmp" => [Jmp],
    b"jz" => [Jz],
    b"jn" => [Jn],
    b"ret" => [Ret],
    b"exit" => [End],
    b"pchr" => [Printc],
    b"pnum" => [Printi],
    b"ichr" => [Readc],
    b"inum" => [Readi],
};

impl CensoredUsername {
    /// Constructs state for the CensoredUsername dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        CensoredUsername {
            mnemonics: MnemonicMap::from(MNEMONICS),
            integers: CensoredUsername::new_integers(),
        }
    }

    /// Lexes a Whitespace assembly program in the CensoredUsername dialect.
    pub fn lex<'s>(&self, src: &'s [u8]) -> Vec<Token<'s>> {
        let mut lex = Lexer::new(src, self);
        let mut toks = Vec::new();
        loop {
            let tok = lex.next_token();
            if let Token::Eof(_) = tok {
                break;
            }
            toks.push(tok);
        }
        toks
    }

    /// Gets the mnemonic map for this dialect.
    pub fn mnemonics(&self) -> &MnemonicMap {
        &self.mnemonics
    }

    /// Gets the integer syntax description for this dialect.
    ///
    /// See [`CensoredUsername::new_integers`] for the grammar.
    pub fn integers(&self) -> &IntegerSyntax {
        &self.integers
    }

    /// Constructs an integer syntax description for this dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::= "-"? [0-9]+
    /// ```
    pub const fn new_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::Neg,
            base_style: BaseStyle::Rust,
            bases: enum_set!(Base::Decimal),
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
