//! Parsing for the CensoredUsername Whitespace assembly dialect.

use crate::{
    dialects::{censoredusername::lex::Lexer, dialect::DialectState, Dialect},
    lex::Lex,
    syntax::{Cst, Opcode},
    tokens::{
        integer::{BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        mnemonics::FoldedStr,
        Token,
    },
};

/// State for parsing the CensoredUsername Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct CensoredUsername;

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

impl Dialect for CensoredUsername {
    const MNEMONICS: &[(FoldedStr<'_>, &[Opcode])] = MNEMONICS;

    fn parse<'s>(_src: &'s [u8], _dialect: &DialectState<Self>) -> Cst<'s> {
        todo!()
    }

    fn lex<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Vec<Token<'s>> {
        let mut lex = Lexer::new(src, dialect);
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

    /// Constructs an integer syntax description for this dialect.
    ///
    /// # Syntax
    ///
    /// ```bnf
    /// integer ::= "-"? [0-9]+
    /// ```
    fn make_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::Neg,
            base_styles: BaseStyle::Decimal.into(),
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
