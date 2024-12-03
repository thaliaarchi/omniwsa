//! Parsing for the wsf Whitespace assembly dialect.

use crate::{
    dialects::{define_mnemonics, dialect::DialectState, wsf::lex::Lexer, Dialect},
    lex::Lex,
    syntax::Cst,
    tokens::{
        integer::{BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        Token,
    },
};

/// wsf Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct Wsf;

impl Dialect for Wsf {
    define_mnemonics! {
        fold = Exact,
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
    }

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
    /// integer ::=
    ///     | "-"? [0-9]+
    ///     | "-"? ("0x" | "0X") [0-9 a-f A-F]+
    ///     | "-"? ("0b" | "0B") [01]+
    /// ```
    fn make_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::Neg,
            base_styles: BaseStyle::Decimal
                | BaseStyle::BinPrefix_0b
                | BaseStyle::BinPrefix_0B
                | BaseStyle::HexPrefix_0x
                | BaseStyle::HexPrefix_0X,
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
