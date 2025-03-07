//! Syntax for expressions.

use enumset::{EnumSet, EnumSetType};

use crate::{
    syntax::Pretty,
    tokens::{Token, spaces::Spaces},
};

/// A unary expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryExpr<'s> {
    /// The unary operator applied to the operand.
    pub op: UnaryOp,
    /// Spaces between the operator and the operand.
    pub space: Spaces<'s>,
    /// The operand.
    pub value: Box<Token<'s>>,
}

/// A unary operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation `-`.
    Neg,
    /// Positive sign `+`.
    Plus,
}

/// A token enclosed in parentheses or non-semantic quotes (Burghard).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupToken<'s> {
    /// The style of the delimiter enclosing this token.
    pub delim: GroupStyle,
    /// Spaces between the opening delimiter and the inner token.
    pub space_before: Spaces<'s>,
    /// The effective token.
    pub inner: Box<Token<'s>>,
    /// Spaces between the inner token and the closing delimiter.
    pub space_after: Spaces<'s>,
    /// All errors from parsing this token.
    pub errors: EnumSet<GroupError>,
}

/// The style of a non-semantic group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupStyle {
    /// Parentheses. Integers in the Burghard dialect may be wrapped in
    /// parentheses.
    Parens,
    /// `"`-quotes. Any word in the Burghard dialect may be wrapped in
    /// non-semantic quotes.
    DoubleQuotes,
}

/// A parse error for a group token.
#[derive(EnumSetType, Debug)]
pub enum GroupError {
    /// Has no opening delimiter
    Unopened,
    /// Has no closing delimiter.
    Unclosed,
}

impl UnaryOp {
    /// The unary operator.
    pub fn op(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Plus => "+",
        }
    }
}

impl GroupStyle {
    /// The opening delimiter.
    pub const fn open(&self) -> &'static str {
        match self {
            GroupStyle::Parens => "(",
            GroupStyle::DoubleQuotes => "\"",
        }
    }

    /// The closing delimiter.
    pub const fn close(&self) -> &'static str {
        match self {
            GroupStyle::Parens => ")",
            GroupStyle::DoubleQuotes => "\"",
        }
    }
}

impl Pretty for GroupToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        if !self.errors.contains(GroupError::Unopened) {
            self.delim.open().pretty(buf);
        }
        self.inner.pretty(buf);
        if !self.errors.contains(GroupError::Unclosed) {
            self.delim.close().pretty(buf);
        }
    }
}
