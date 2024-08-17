//! Concrete syntax tree for interoperable Whitespace assembly.

use crate::{
    syntax::{Inst, Opcode},
    tokens::spaces::Spaces,
};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// A line with no instructions.
    Empty(Spaces<'s>),
    /// Sequence of nodes.
    Block {
        /// The nodes in this block.
        nodes: Vec<Cst<'s>>,
    },
    /// Conditionally compiled block.
    OptionBlock(OptionBlock<'s>),
    /// Marker for the dialect of the contained CST.
    Dialect {
        /// The dialect of the contained CST.
        dialect: Dialect,
        /// The contained CST.
        inner: Box<Cst<'s>>,
    },
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
            Cst::Empty(sep) => sep.has_error(),
            Cst::Block { nodes } => nodes.has_error(),
            Cst::OptionBlock(block) => block.has_error(),
            Cst::Dialect { dialect: _, inner } => inner.has_error(),
        }
    }
}

impl HasError for OptionBlock<'_> {
    fn has_error(&self) -> bool {
        self.options.is_empty()
            || self.options.first().unwrap().0.opcode() != Opcode::IfOption
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
