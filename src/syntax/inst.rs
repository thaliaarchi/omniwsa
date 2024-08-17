//! Instruction syntax tree nodes.

use crate::{
    syntax::{HasError, Opcode},
    tokens::{words::Words, TokenKind},
};

// TODO:
// - Make an `enum ArgShape` for Palaiologos mnemonic-less `push` and `label`.
// - How to represent nonsensical sequences of tokens, that can't be structured
//   into an instruction? A Cst variant for a token list with an error?

/// An instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst<'s> {
    /// The mnemonic and arguments in this instruction, separated and surrounded
    /// by optional spaces. Words may possibly be wrapped in `Quoted` or
    /// `Spliced`, if from the Burghard dialect.
    pub words: Words<'s>,
    /// Whether it has the correct number of arguments for the opcode.
    pub valid_arity: bool,
    /// Whether the arguments have valid types for the opcode.
    pub valid_types: bool,
}

impl<'s> Inst<'s> {
    /// Returns the opcode for this instruction. Panics if `self.opcode` is not
    /// an opcode token.
    pub fn opcode(&self) -> Opcode {
        match &self.words[0].unwrap().kind {
            TokenKind::Mnemonic(m) => m.opcode,
            _ => panic!("not a mnemonic"),
        }
    }
}

impl HasError for Inst<'_> {
    fn has_error(&self) -> bool {
        self.words.has_error() || !self.valid_arity || !self.valid_types
    }
}
