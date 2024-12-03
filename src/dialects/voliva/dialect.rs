//! Parsing for the voliva Whitespace assembly dialect.

use crate::{
    dialects::{define_mnemonics, dialect::DialectState, voliva::lex::Lexer, Dialect},
    lex::Lex,
    syntax::Cst,
    tokens::{
        integer::{BaseStyle, DigitSep, IntegerSyntax, SignStyle},
        Token,
    },
};

// TODO:
// - Handle allowing signs for only decimal integer literals in `IntegerSyntax`.

/// voliva Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct Voliva;

impl Dialect for Voliva {
    define_mnemonics! {
        fold = Exact,
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
        b"dbg" => [VolivaBreakpoint],
        b"include" => [VolivaInclude],
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
    ///     | ("-" | "+") [0-9]+
    ///     | ("0b" | "0B") [01]+
    ///     | ("0o" | "0O") [0-7]+
    ///     | ("0x" | "0X") [0-9 a-f A-F]+
    /// ```
    fn make_integers() -> IntegerSyntax {
        IntegerSyntax {
            // Explicit signs are only allowed for decimal.
            sign_style: SignStyle::NegPos,
            base_styles: BaseStyle::Decimal
                | BaseStyle::BinPrefix_0b
                | BaseStyle::BinPrefix_0B
                | BaseStyle::OctPrefix_0o
                | BaseStyle::OctPrefix_0O
                | BaseStyle::HexPrefix_0x
                | BaseStyle::HexPrefix_0X,
            digit_sep: DigitSep::None,
            min_value: None,
            max_value: None,
        }
    }
}
