//! Instruction syntax tree nodes.

use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::{HasError, Opcode, Overload},
    tokens::{Token, spaces::Spaces, words::Words},
};

// TODO:
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
    /// The layout that its arguments are syntactically arranged in.
    pub arg_layout: ArgLayout,
    /// An overloaded interpretation of its arguments.
    pub overload: Option<Overload>,
    /// Errors from parsing this instruction.
    pub errors: EnumSet<InstError>,
}

/// A layout that arguments are syntactically arranged in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgLayout {
    /// A mnemonic followed by arguments.
    Mnemonic,
    /// Arguments alone, without a mnemonic (e.g., label definitions and
    /// Palaiologos mnemonic-less `push`).
    Bare,
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
            arg_layout: ArgLayout::Bare,
            overload: None,
            errors: EnumSet::empty(),
        }
    }

    /// Gets the argument at the given index. Panics if out of range.
    pub fn arg(&self, index: usize) -> &Token<'s> {
        &self.words[index + (self.arg_layout == ArgLayout::Mnemonic) as usize]
    }

    /// Gets a mutable reference to the argument at the given index. Panics if
    /// out of range.
    pub fn arg_mut(&mut self, index: usize) -> &Token<'s> {
        &mut self.words[index + (self.arg_layout == ArgLayout::Mnemonic) as usize]
    }

    /// Returns the number of arguments.
    pub fn len_args(&self) -> usize {
        self.words
            .len()
            .saturating_sub((self.arg_layout == ArgLayout::Mnemonic) as usize)
    }
}

impl HasError for Inst<'_> {
    fn has_error(&self) -> bool {
        self.words.has_error() || !self.errors.is_empty()
    }
}
