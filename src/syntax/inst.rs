//! Instruction syntax tree nodes.

use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Opcode},
    tokens::{spaces::Spaces, words::Words},
};

// TODO:
// - Make an `enum ArgShape` for Palaiologos mnemonic-less `push` and `label`.
// - How to represent nonsensical sequences of tokens, that can't be structured
//   into an instruction? A Cst variant for a token list with an error?

/// An instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst<'s> {
    /// The resolved opcode of this instruction.
    pub opcode: Opcode,
    /// The mnemonic and arguments in this instruction, separated and surrounded
    /// by optional spaces. Words may possibly be wrapped in `Quoted` or
    /// `Spliced`, if from the Burghard dialect.
    pub words: Words<'s>,
    /// Errors from parsing this instruction.
    pub errors: EnumSet<InstError>,
}

/// A parse error for an instruction.
#[derive(EnumSetType, Debug)]
pub enum InstError {
    /// The is not the correct number of arguments for the opcode.
    InvalidArity,
    /// The arguments do not have valid types for this opcode.
    InvalidTypes,
}

impl<'s> Inst<'s> {
    /// Constructs an instruction with no operation.
    pub fn nop(spaces: Spaces<'s>) -> Self {
        Inst {
            opcode: Opcode::Nop,
            words: Words::new(spaces),
            errors: EnumSet::empty(),
        }
    }
}

impl HasError for Inst<'_> {
    fn has_error(&self) -> bool {
        self.words.has_error() || !self.errors.is_empty()
    }
}
