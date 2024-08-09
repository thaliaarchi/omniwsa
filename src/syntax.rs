//! Concrete syntax tree for interoperable Whitespace assembly.

use crate::token::Token;

// TODO:
// - Macro definitions and invocations.
// - Include comments.

/// A node in a concrete syntax tree for interoperable Whitespace assembly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// Sequence of nodes.
    Block { nodes: Vec<Cst<'s>> },
    /// Conditional compilation
    /// (Burghard `ifoption`/`elseifoption`/`elseoption`/`endoption` and
    /// Respace `@ifdef`/`@else`/`@endif`).
    OptionBlock {
        options: Vec<(Inst<'s>, Vec<Cst<'s>>)>,
        end: Inst<'s>,
    },
    /// Marker for the dialect of the contained CST.
    Dialect {
        dialect: Dialect,
        inner: Box<Cst<'s>>,
    },
}

/// Instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst<'s> {
    pub space_before: Option<Token<'s>>,
    pub mnemonic: Token<'s>,
    pub args: Vec<(Sep<'s>, Token<'s>)>,
    pub space_after: Option<Token<'s>>,
    pub inst_sep: Token<'s>,
}

/// Argument separator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sep<'s> {
    Space {
        space: Token<'s>,
    },
    Sep {
        space_before: Option<Token<'s>>,
        sep: Token<'s>,
        space_after: Option<Token<'s>>,
    },
}

/// A Whitespace assembly dialect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    Burghard,
    Lime,
    LittleBugHunter,
    Palaiologos,
    Rdebath,
    Respace,
    Voliva,
    Whitelips,
}
