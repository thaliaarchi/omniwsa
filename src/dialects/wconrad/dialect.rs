//! Parsing for the wconrad Whitespace assembly dialect.

use crate::{
    dialects::{Dialect, DialectState, define_mnemonics, wconrad::lex::Lexer},
    lex::Lex,
    syntax::Cst,
    tokens::{
        Token,
        integer::{BaseStyle, DigitSep, IntegerSyntax, SignStyle},
    },
};

// TODO:
// - Create a generic word scanner, which splits by Unicode spaces. Further
//   validation is then done by the dialect.
// - Create classes of integers, so that numbers can have signs, but labels
//   can't.

/// wconrad Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct WConrad;

impl Dialect for WConrad {
    define_mnemonics! {
        fold = Exact,
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
    }

    fn parse<'s>(_src: &'s [u8], _dialect: &DialectState<Self>) -> Cst<'s> {
        todo!()
    }

    fn lex<'s>(src: &'s [u8], _dialect: &DialectState<Self>) -> Vec<Token<'s>> {
        let mut lex = Lexer::new(src);
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
    /// integer ::= [-+]? [0-9]+
    /// ```
    ///
    /// wconrad has no bases, so choose a style close to Ruby, but without `0`
    /// octal. Ruby has `0x`/`0X` for hexadecimal, `0`/`0o`/`0O` for octal,
    /// `0b`/`0B` for binary, and `0d`/`0D` or bare for decimal.
    fn make_integers() -> IntegerSyntax {
        IntegerSyntax {
            sign_style: SignStyle::NegPos,
            base_styles: BaseStyle::Decimal.into(),
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
