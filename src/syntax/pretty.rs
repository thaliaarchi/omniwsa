//! Pretty-printing for CST nodes.

use crate::{
    syntax::{Cst, Inst, OptionBlock},
    tokens::{spaces::Spaces, words::Words, Token},
};

/// Pretty-prints this node as Whitespace assembly syntax.
pub trait Pretty {
    /// Pretty-prints this node as Whitespace assembly syntax, written to the
    /// given buffer.
    fn pretty(&self, buf: &mut Vec<u8>);
}

impl Pretty for Token<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.text);
    }
}

impl Pretty for Cst<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match self {
            Cst::Inst(inst) => inst.pretty(buf),
            Cst::Empty(sep) => sep.pretty(buf),
            Cst::Block { nodes } => nodes.iter().for_each(|node| node.pretty(buf)),
            Cst::OptionBlock(block) => block.pretty(buf),
            Cst::Dialect { dialect: _, inner } => inner.pretty(buf),
        }
    }
}

impl Pretty for Inst<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.words.pretty(buf);
    }
}

impl Pretty for Words<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.space_before.pretty(buf);
        self.words.iter().for_each(|(word, space)| {
            word.pretty(buf);
            space.pretty(buf);
        });
    }
}

impl Pretty for Spaces<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.tokens().iter().for_each(|tok| tok.pretty(buf))
    }
}

impl Pretty for OptionBlock<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.options.iter().for_each(|(option, block)| {
            option.pretty(buf);
            block.iter().for_each(|node| node.pretty(buf));
        });
        self.end.pretty(buf);
    }
}

impl<T: Pretty> Pretty for Option<T> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.as_ref().inspect(|v| v.pretty(buf));
    }
}
