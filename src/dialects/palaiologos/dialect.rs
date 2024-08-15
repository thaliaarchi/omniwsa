//! Parsing for the Palaiologos Whitespace assembly dialect.

// TODO:
// - Create another table mapping opcode to argument types and the canonical
//   mnemonic.

use std::collections::HashMap;

use crate::{
    dialects::palaiologos::lex::Lexer,
    mnemonics::AsciiLower,
    tokens::{Opcode, Token, TokenKind},
};

/// State for parsing the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Palaiologos {
    mnemonics: HashMap<AsciiLower<'static>, Opcode>,
}

macro_rules! mnemonics[($($mnemonic:literal => $opcode:ident,)*) => {
    &[$(($mnemonic, Opcode::$opcode),)+]
}];
static MNEMONICS: &[(&'static str, Opcode)] = mnemonics![
    "psh" => Push,
    "push" => Push,
    "dup" => Dup,
    "copy" => Copy,
    "take" => Copy,
    "pull" => Copy,
    "xchg" => Swap,
    "swp" => Swap,
    "swap" => Swap,
    "drop" => Drop,
    "dsc" => Drop,
    "slide" => Slide,
    "add" => Add,
    "sub" => Sub,
    "mul" => Mul,
    "div" => Div,
    "mod" => Mod,
    "sto" => Store,
    "rcl" => Retrieve,
    "call" => Call,
    "gosub" => Call,
    "jsr" => Call,
    "jmp" => Jmp,
    "j" => Jmp,
    "b" => Jmp,
    "jz" => Jz,
    "bz" => Jz,
    "jltz" => Jn,
    "bltz" => Jn,
    "ret" => Ret,
    "end" => End,
    "putc" => Printc,
    "putn" => Printi,
    "getc" => Readc,
    "getn" => Readi,
    "rep" => PalaiologosRep,
];

impl Palaiologos {
    pub(super) const MAX_MNEMONIC_LEN: usize = 5;

    /// Constructs state for the Palaiologos dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Palaiologos {
            mnemonics: MNEMONICS
                .iter()
                .map(|&(mnemonic, opcode)| (AsciiLower(mnemonic.as_bytes()), opcode))
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Palaiologos dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Vec<Token<'s>> {
        let dialect = Palaiologos::new();
        let mut lex = Lexer::new(src, &dialect);
        let mut tokens = Vec::new();
        loop {
            let tok = lex.next_token();
            if tok.kind == TokenKind::Eof {
                return tokens;
            }
            tokens.push(tok);
        }
    }

    /// Gets the opcode for a mnemonic.
    pub(super) fn get_opcode(&self, mnemonic: &[u8]) -> Option<Opcode> {
        self.mnemonics.get(&AsciiLower(mnemonic)).copied()
    }
}
