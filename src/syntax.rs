//! Concrete syntax tree for interoperable Whitespace assembly.

use std::fmt::{self, Debug, Formatter};

pub use crate::pretty::Pretty;
use crate::tokens::{Opcode, Token, TokenKind};

// TODO:
// - Macro definitions and invocations.
// - Use bit flags for errors.
// - Rename `Cst` -> `Node` and combine `Node::Block` and `Node::Dialect` as
//   `struct Cst`.
// - Make a `SourceSet` to store program sources from several files, manage
//   positions, and be referenced by the CST.
// - Make a Cst variant for Palaiologos mnemonic-less `push` and `label`.
// - How to represent nonsensical sequences of tokens, that can't be structured
//   into an instruction? A Cst variant for a token list with an error?

/// A node in a concrete syntax tree for interoperable Whitespace assembly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cst<'s> {
    /// Instruction.
    Inst(Inst<'s>),
    /// A line with no instructions.
    Empty(InstSep<'s>),
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

/// An instruction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inst<'s> {
    /// Any whitespace preceding the mnemonic.
    pub space_before: Space<'s>,
    /// The mnemonic denoting the opcode of this instruction. It is always
    /// `Opcode`, possibly wrapped in `Quoted` or `Spliced`, if from the
    /// Burghard dialect.
    pub opcode: Token<'s>,
    /// The arguments and preceding separators in this instruction. Each
    /// argument token may possibly be wrapped in `Quoted` or `Spliced`, if from
    /// the Burghard dialect.
    pub args: Vec<(ArgSep<'s>, Token<'s>)>,
    /// The separator following the last argument.
    pub inst_sep: InstSep<'s>,
    /// Whether it has the correct number of arguments for the opcode.
    pub valid_arity: bool,
    /// Whether the arguments have valid types for the opcode.
    pub valid_types: bool,
}

/// A sequence of whitespace and block comments.
#[derive(Clone, PartialEq, Eq)]
pub struct Space<'s> {
    /// The contained tokens. All are always `Space` or `BlockComment`.
    pub tokens: Vec<Token<'s>>,
}

/// A token surrounded by optional whitespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spaced<'s, T> {
    /// The whitespace tokens preceding this token.
    pub space_before: Space<'s>,
    /// The contained non-whitespace token.
    pub inner: T,
    /// The whitespace tokens following this token.
    pub space_after: Space<'s>,
}

/// Argument separator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArgSep<'s> {
    /// Whitespace only.
    Space(Space<'s>),
    /// A non-whitespace token that separates arguments. Always `ArgSep`.
    Sep(Spaced<'s, Token<'s>>),
}

/// Instruction separator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstSep<'s> {
    /// A line terminator.
    LineTerm {
        /// The whitespace tokens preceding the line comment.
        space_before: Space<'s>,
        /// An optional line comment. Always `LineComment`.
        line_comment: Option<Token<'s>>,
        /// The line terminator or EOF. Always `LineTerm` or `Eof`.
        line_term: Token<'s>,
    },
    /// A non-whitespace token that separates instructions. Always `InstSep`.
    Sep(Spaced<'s, Token<'s>>),
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

impl<'s> Inst<'s> {
    /// Returns the opcode for this instruction. Panics if `self.opcode` is not
    /// an opcode token.
    pub fn opcode(&self) -> Opcode {
        match self.opcode.unwrap().kind {
            TokenKind::Opcode(opcode) => opcode,
            _ => panic!("not an opcode"),
        }
    }

    /// Returns a mutable reference to the opcode and the space space after it.
    pub fn opcode_space_after_mut(&mut self) -> (&mut Token<'s>, &mut Space<'s>) {
        let space_after = match self.args.first_mut() {
            Some((sep, _)) => sep.space_before_mut(),
            None => self.inst_sep.space_before_mut(),
        };
        (&mut self.opcode, space_after)
    }

    /// Returns a mutable reference to the numbered argument and the space after
    /// it.
    pub fn arg_space_after_mut(&mut self, arg: usize) -> (&mut Token<'s>, &mut Space<'s>) {
        let (l, r) = self.args.split_at_mut(arg + 1);
        let space_after = match r {
            [next, ..] => next.0.space_before_mut(),
            [] => self.inst_sep.space_before_mut(),
        };
        (&mut l[arg].1, space_after)
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

    /// Trims leading spaces.
    pub fn trim_leading(&mut self) {
        let i = self
            .tokens
            .iter()
            .position(|tok| tok.kind != TokenKind::Space)
            .unwrap_or(self.tokens.len());
        self.tokens.drain(..i);
    }

    /// Trims trailing spaces.
    pub fn trim_trailing(&mut self) {
        let i = self
            .tokens
            .iter()
            .rposition(|tok| tok.kind != TokenKind::Space)
            .map(|i| i + 1)
            .unwrap_or(0);
        self.tokens.drain(i..);
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

impl<'s> ArgSep<'s> {
    /// Returns a mutable reference to the space before the argument separator.
    pub fn space_before_mut(&mut self) -> &mut Space<'s> {
        match self {
            ArgSep::Space(space) => space,
            ArgSep::Sep(sep) => &mut sep.space_before,
        }
    }

    /// Returns a mutable reference to the space after the argument separator.
    pub fn space_after_mut(&mut self) -> &mut Space<'s> {
        match self {
            ArgSep::Space(space) => space,
            ArgSep::Sep(sep) => &mut sep.space_after,
        }
    }
}

impl<'s> InstSep<'s> {
    /// Returns a mutable reference to the space before the instruction
    /// separator.
    pub fn space_before_mut(&mut self) -> &mut Space<'s> {
        match self {
            InstSep::LineTerm { space_before, .. } => space_before,
            InstSep::Sep(sep) => &mut sep.space_before,
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
