//! Concrete syntax tree for interoperable Whitespace assembly.

use std::fmt::{self, Debug, Formatter};

use crate::syntax::{Inst, Opcode};

// TODO:
// - Macro definitions and invocations.
// - Use bit flags for errors.
// - Rename `Cst` -> `Node` and combine `Node::Block` and `Node::Dialect` as
//   `struct Cst`.
// - Make a `SourceSet` to store program sources from several files, manage
//   positions, and be referenced by the CST.
// - `Inst` and `Empty` could be unified as just `Words`, renamed to `Inst`,
//   where `Empty` is `Nop`.

/// A node in a concrete syntax tree for interoperable Whitespace assembly.
#[derive(Clone, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// Sequence of nodes.
    Block {
        /// The nodes in this block.
        nodes: Vec<Cst<'s>>,
    },
    /// Conditionally compiled block.
    OptionBlock(OptionBlock<'s>),
}

/// A conditionally compiled block (Burghard `ifoption` and Respace `@ifdef`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionBlock<'s> {
    /// The branches of this option block
    /// (Burghard `ifoption`/`elseifoption`/`elseoption`
    /// and Respace `@ifdef`/`@else`/`@endif`).
    pub options: Vec<(Inst<'s>, Vec<Cst<'s>>)>,
    /// The instruction closing this option block (Burghard `endoption` and
    /// Respace `@endif`). When not present, it is an error.
    pub end: Option<Inst<'s>>,
}

/// A Whitespace assembly dialect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    /// Burghard Whitespace assembly.
    Burghard,
    /// Lime Whitespace assembly.
    Lime,
    /// littleBugHunter Whitespace assembly.
    LittleBugHunter,
    /// Palaiologos Whitespace assembly.
    Palaiologos,
    /// rdebath Whitespace assembly.
    Rdebath,
    /// Respace Whitespace assembly.
    Respace,
    /// voliva Whitespace assembly.
    Voliva,
    /// Whitelips Whitespace assembly.
    Whitelips,
}

/// A type that can report whether it contains any syntax errors.
pub trait HasError {
    /// Returns whether this contains any syntax errors.
    fn has_error(&self) -> bool;
}

impl Dialect {
    /// The name of this dialect.
    pub fn name(&self) -> &'static str {
        match self {
            Dialect::Burghard => "Burghard",
            Dialect::Lime => "Lime",
            Dialect::LittleBugHunter => "littleBugHunter",
            Dialect::Palaiologos => "Palaiologos",
            Dialect::Rdebath => "rdebath",
            Dialect::Respace => "Respace",
            Dialect::Voliva => "voliva",
            Dialect::Whitelips => "Whitelips",
        }
    }

    /// A shortened name for this dialect, for use in filenames.
    pub fn short_name(&self) -> &'static str {
        match self {
            Dialect::Burghard => "burg",
            Dialect::Lime => "lime",
            Dialect::LittleBugHunter => "lbug",
            Dialect::Palaiologos => "palo",
            Dialect::Rdebath => "rdb",
            Dialect::Respace => "resp",
            Dialect::Voliva => "voli",
            Dialect::Whitelips => "wlip",
        }
    }
}

impl HasError for Cst<'_> {
    fn has_error(&self) -> bool {
        match self {
            Cst::Inst(inst) => inst.has_error(),
            Cst::Block { nodes } => nodes.has_error(),
            Cst::OptionBlock(block) => block.has_error(),
        }
    }
}

impl HasError for OptionBlock<'_> {
    fn has_error(&self) -> bool {
        self.options.is_empty()
            || self.options.first().unwrap().0.opcode != Opcode::IfOption
            || self
                .options
                .iter()
                .any(|(option, block)| option.has_error() || block.has_error())
            || self.end.is_none()
            || self.end.has_error()
    }
}

impl<T: HasError> HasError for Option<T> {
    fn has_error(&self) -> bool {
        self.as_ref().is_some_and(T::has_error)
    }
}

impl<T: HasError> HasError for Vec<T> {
    fn has_error(&self) -> bool {
        self.iter().any(T::has_error)
    }
}

impl<'s> From<Inst<'s>> for Cst<'s> {
    fn from(inst: Inst<'s>) -> Self {
        Cst::Inst(inst)
    }
}

impl Debug for Cst<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Cst::Inst(inst) => Debug::fmt(inst, f),
            Cst::Block { nodes } => {
                write!(f, "Block ")?;
                f.debug_list().entries(nodes).finish()
            }
            Cst::OptionBlock(block) => Debug::fmt(block, f),
        }
    }
}
