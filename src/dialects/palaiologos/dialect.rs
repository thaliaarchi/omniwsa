//! Parsing for the Palaiologos Whitespace assembly dialect.

use crate::{
    dialects::{
        Dialect, define_mnemonics,
        dialect::DialectState,
        palaiologos::{lex::Lexer, parse::Parser},
    },
    lex::Lex,
    syntax::Cst,
    tokens::{
        Token,
        integer::{BaseStyle, DigitSep, Integer, IntegerSyntax, SignStyle},
    },
};

/// Palaiologos Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct Palaiologos;

impl Dialect for Palaiologos {
    define_mnemonics! {
        fold = Ascii,
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
        b"add" => [Add], // Overload::BinaryConstRhs
        b"sub" => [Sub], // Overload::BinaryConstRhs
        b"mul" => [Mul], // Overload::BinaryConstRhs
        b"div" => [Div], // Overload::BinaryConstRhs
        b"mod" => [Mod], // Overload::BinaryConstRhs
        b"sto" => [Store], // Overload::BinaryConstRhs, Overload::BinaryConstConst
        b"rcl" => [Retrieve], // Overload::UnaryConst
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
        b"putc" => [Printc], // Overload::UnaryConst
        b"putn" => [Printi], // Overload::UnaryConst
        b"getc" => [Readc], // Overload::UnaryConst
        b"getn" => [Readi], // Overload::UnaryConst
        b"rep" => [PalaiologosRep],
    }

    fn parse<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Cst<'s> {
        Parser::new(src, dialect).parse()
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
    while i < Palaiologos::MNEMONICS.len() {
        let len = Palaiologos::MNEMONICS[i].0.bytes.len();
        if len > max_len {
            max_len = len;
        }
        i += 1;
    }
    max_len
};
