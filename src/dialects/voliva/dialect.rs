//! Parsing for the voliva Whitespace assembly dialect.

use enumset::enum_set;

use crate::{
    dialects::voliva::lex::Lexer,
    lex::Lex,
    syntax::Opcode,
    tokens::{
        integer::{BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        mnemonics::{FoldedStr, MnemonicMap},
        Token,
    },
};

// TODO:
// - Handle allowing signs for only decimal integer literals in `IntegerSyntax`.

/// State for parsing the voliva Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Voliva {
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
    b"add" => [Add, AddConstRhs],
    b"sub" => [Sub, SubConstRhs],
    b"mul" => [Mul, MulConstRhs],
    b"div" => [Div, DivConstRhs],
    b"mod" => [Mod, ModConstRhs],
    b"or" => [VolivaOr, VolivaOrConstRhs],
    b"not" => [VolivaNot],
    b"and" => [VolivaAnd, VolivaOrConstRhs],
    b"store" => [Store, StoreConstLhs],
    b"storestr" => [StoreString0],
    b"retrieve" => [Retrieve, RetrieveConst],
    b"label" => [Label],
    b"call" => [Call],
    b"jump" => [Jmp],
    b"jumpz" => [Jz],
    b"jumpn" => [Jn],
    b"jumpp" => [VolivaJmpPos],
    b"jumppn" => [VolivaJmpNonZero],
    b"jumpnp" => [VolivaJmpNonZero],
    b"jumpnz" => [VolivaJmpNonPos],
    b"jumppz" => [VolivaJmpNonNeg],
    b"ret" => [Ret],
    b"exit" => [End],
    b"outn" => [Printi],
    b"outc" => [Printc],
    b"readn" => [Readi],
    b"readc" => [Readc],
    b"valueinteger" => [VolivaValueInteger],
    b"valuestring" => [VolivaValueString],
    b"debugger" => [VolivaBreakpoint],
    b"include" => [VolivaInclude],
};

impl Voliva {
    /// Constructs state for the voliva dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Voliva {
            mnemonics: MnemonicMap::from(MNEMONICS),
            integers: Voliva::new_integers(),
        }
    }

    /// Lexes a Whitespace assembly program in the voliva dialect.
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
    /// See [`Voliva::new_integers`] for the grammar.
    pub fn integers(&self) -> &IntegerSyntax {
        &self.integers
    }

    /// Constructs an integer syntax description for this dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::=
    ///     | ("-" | "+") [0-9]+
    ///     | ("0b" | "0B") [01]+
    ///     | ("0o" | "0O") [0-7]+
    ///     | ("0x" | "0X") [0-9 a-f A-F]+
    /// ```
    pub const fn new_integers() -> IntegerSyntax {
        IntegerSyntax {
            // Explicit signs are only allowed for decimal.
            sign_style: SignStyle::NegPos,
            base_styles: enum_set!(
                BaseStyle::Decimal
                    | BaseStyle::BinPrefix_0b
                    | BaseStyle::BinPrefix_0B
                    | BaseStyle::OctPrefix_0o
                    | BaseStyle::OctPrefix_0O
                    | BaseStyle::HexPrefix_0x
                    | BaseStyle::HexPrefix_0X
            ),
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}