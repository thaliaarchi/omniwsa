//! Parsing for the Palaiologos Whitespace assembly dialect.

// TODO:
// - Create another table mapping opcode to argument types and the canonical
//   mnemonic.

use std::collections::HashMap;

use crate::{
    dialects::palaiologos::parse::Parser,
    syntax::{Cst, Opcode},
    tokens::mnemonics::AsciiLower,
};

/// State for parsing the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Palaiologos {
    mnemonics: HashMap<AsciiLower<'static>, &'static [Opcode]>,
}

static MNEMONICS: &[(&str, &[Opcode])] = &[
    ("psh", &[Opcode::Push, Opcode::Push0]),
    ("push", &[Opcode::Push, Opcode::Push0]),
    ("dup", &[Opcode::Dup]),
    ("copy", &[Opcode::Copy]),
    ("take", &[Opcode::Copy]),
    ("pull", &[Opcode::Copy]),
    ("xchg", &[Opcode::Swap]),
    ("swp", &[Opcode::Swap]),
    ("swap", &[Opcode::Swap]),
    ("drop", &[Opcode::Drop]),
    ("dsc", &[Opcode::Drop]),
    ("slide", &[Opcode::Slide]),
    ("add", &[Opcode::Add, Opcode::AddConstRhs]),
    ("sub", &[Opcode::Sub, Opcode::SubConstRhs]),
    ("mul", &[Opcode::Mul, Opcode::MulConstRhs]),
    ("div", &[Opcode::Div, Opcode::DivConstRhs]),
    ("mod", &[Opcode::Mod, Opcode::ModConstRhs]),
    (
        "sto",
        &[
            Opcode::Store,
            Opcode::StoreConstRhs,
            Opcode::StoreConstConst,
        ],
    ),
    ("rcl", &[Opcode::Retrieve, Opcode::RetrieveConst]),
    ("call", &[Opcode::Call]),
    ("gosub", &[Opcode::Call]),
    ("jsr", &[Opcode::Call]),
    ("jmp", &[Opcode::Jmp]),
    ("j", &[Opcode::Jmp]),
    ("b", &[Opcode::Jmp]),
    ("jz", &[Opcode::Jz]),
    ("bz", &[Opcode::Jz]),
    ("jltz", &[Opcode::Jn]),
    ("bltz", &[Opcode::Jn]),
    ("ret", &[Opcode::Ret]),
    ("end", &[Opcode::End]),
    ("putc", &[Opcode::Printc, Opcode::PrintcConst]),
    ("putn", &[Opcode::Printi, Opcode::PrintiConst]),
    ("getc", &[Opcode::Readc, Opcode::ReadcConst]),
    ("getn", &[Opcode::Readi, Opcode::ReadiConst]),
    ("rep", &[Opcode::PalaiologosRep]),
];

impl Palaiologos {
    pub(super) const MAX_MNEMONIC_LEN: usize = 5;

    /// Constructs state for the Palaiologos dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Palaiologos {
            mnemonics: MNEMONICS
                .iter()
                .map(|&(mnemonic, opcodes)| (AsciiLower(mnemonic.as_bytes()), opcodes))
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Palaiologos dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        Parser::new(src, &Palaiologos::new()).parse()
    }

    /// Gets the overloaded opcodes for a mnemonic.
    pub(super) fn get_opcodes(&self, mnemonic: &[u8]) -> Option<&'static [Opcode]> {
        self.mnemonics.get(&AsciiLower(mnemonic)).copied()
    }
}
