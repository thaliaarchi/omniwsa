//! Concrete syntax tree for interoperable Whitespace assembly.

use crate::token::{Token, TokenKind};

// TODO:
// - Macro definitions and invocations.

/// A node in a concrete syntax tree for interoperable Whitespace assembly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// A line with no instructions.
    Empty(InstSep<'s>),
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
    pub space_before: Space<'s>,
    pub mnemonic: Token<'s>,
    pub args: Vec<(ArgSep<'s>, Token<'s>)>,
    pub inst_sep: InstSep<'s>,
}

/// A sequence of whitespace and block comments.
#[derive(Clone, Debug, PartialEq, Eq)]
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
