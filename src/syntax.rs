//! Concrete syntax tree for interoperable Whitespace assembly.

use std::fmt::{self, Debug, Formatter};

use crate::token::{Opcode, Token, TokenKind};

// TODO:
// - Macro definitions and invocations.
// - Use bit flags for errors.

/// A node in a concrete syntax tree for interoperable Whitespace assembly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// A line with no instructions.
    Empty(InstSep<'s>),
    /// Sequence of nodes.
    Block { nodes: Vec<Cst<'s>> },
    /// Conditionally compiled block.
    OptionBlock(OptionBlock<'s>),
    /// Marker for the dialect of the contained CST.
    Dialect {
        dialect: Dialect,
        inner: Box<Cst<'s>>,
    },
}

/// An instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst<'s> {
    pub space_before: Space<'s>,
    pub opcode: Token<'s>,
    pub args: Vec<(ArgSep<'s>, Token<'s>)>,
    pub inst_sep: InstSep<'s>,
    pub valid_arity: bool,
    pub valid_types: bool,
}

/// A sequence of whitespace and block comments.
#[derive(Clone, PartialEq, Eq)]
pub struct Space<'s> {
    pub tokens: Vec<Token<'s>>,
}

/// A token surrounded by optional whitespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spaced<'s, T> {
    pub space_before: Space<'s>,
    pub inner: T,
    pub space_after: Space<'s>,
}

/// Argument separator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgSep<'s> {
    Space(Space<'s>),
    Sep(Spaced<'s, Token<'s>>),
}

/// Instruction separator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstSep<'s> {
    LineTerm {
        space_before: Space<'s>,
        line_comment: Option<Token<'s>>,
        line_term: Token<'s>,
    },
    Sep(Spaced<'s, Token<'s>>),
}

/// A conditionally compiled block
/// (Burghard `ifoption`/`elseifoption`/`elseoption`/`endoption` and
/// Respace `@ifdef`/`@else`/`@endif`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionBlock<'s> {
    pub options: Vec<(Inst<'s>, Vec<Cst<'s>>)>,
    pub end: Option<Inst<'s>>,
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

pub trait HasError {
    /// Returns whether this contains any syntax errors.
    fn has_error(&self) -> bool;
}

impl Inst<'_> {
    /// Returns the opcode for this instruction. Panics if `self.opcode` is not
    /// an opcode token.
    pub fn opcode(&self) -> Opcode {
        match self.opcode.unwrap().kind {
            TokenKind::Opcode(opcode) => opcode,
            _ => panic!("not an opcode"),
        }
    }
}

impl<'s> Space<'s> {
    /// Constructs a new, empty space sequence.
    pub fn new() -> Self {
        Space { tokens: Vec::new() }
    }

    /// Pushes a whitespace or block comment token to the sequence.
    pub fn push(&mut self, token: Token<'s>) {
        Self::assert_space(&token);
        self.tokens.push(token)
    }

    fn assert_space(token: &Token<'s>) {
        debug_assert!(matches!(
            token.kind,
            TokenKind::Space | TokenKind::BlockComment { .. }
        ));
    }
}

impl<'s> From<Vec<Token<'s>>> for Space<'s> {
    fn from(tokens: Vec<Token<'s>>) -> Self {
        tokens.iter().for_each(Self::assert_space);
        Space { tokens }
    }
}

impl<'s> From<Token<'s>> for Space<'s> {
    fn from(token: Token<'s>) -> Self {
        Self::assert_space(&token);
        Space {
            tokens: vec![token],
        }
    }
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

impl HasError for Inst<'_> {
    fn has_error(&self) -> bool {
        self.space_before.has_error()
            || self.opcode.has_error()
            || self
                .args
                .iter()
                .any(|(sep, arg)| sep.has_error() || arg.has_error())
            || self.inst_sep.has_error()
            || !self.valid_arity
            || !self.valid_types
    }
}

impl HasError for Space<'_> {
    fn has_error(&self) -> bool {
        self.tokens.has_error()
    }
}

impl<T: HasError> HasError for Spaced<'_, T> {
    fn has_error(&self) -> bool {
        self.space_before.has_error() || self.inner.has_error() || self.space_after.has_error()
    }
}

impl HasError for ArgSep<'_> {
    fn has_error(&self) -> bool {
        match self {
            ArgSep::Space(space) => space.has_error(),
            ArgSep::Sep(sep) => sep.has_error(),
        }
    }
}

impl HasError for InstSep<'_> {
    fn has_error(&self) -> bool {
        match self {
            InstSep::LineTerm {
                space_before,
                line_comment,
                line_term,
            } => space_before.has_error() || line_comment.has_error() || line_term.has_error(),
            InstSep::Sep(sep) => sep.has_error(),
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

impl Debug for Space<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Space ")?;
        f.debug_list().entries(&self.tokens).finish()
    }
}
