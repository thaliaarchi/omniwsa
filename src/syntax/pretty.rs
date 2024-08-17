//! Pretty-printing for CST nodes.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, OptionBlock},
    tokens::{spaces::Spaces, words::Words, Token, TokenKind},
};

/// Pretty-prints this node as Whitespace assembly syntax.
pub trait Pretty {
    /// Pretty-prints this node as Whitespace assembly syntax, written to the
    /// given buffer.
    fn pretty(&self, buf: &mut Vec<u8>);
}

impl Pretty for Token<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match &self.kind {
            TokenKind::Mnemonic(m) => m.pretty(buf),
            TokenKind::Integer(i) => i.pretty(buf),
            TokenKind::String(s) => s.pretty(buf),
            TokenKind::Char(c) => c.pretty(buf),
            TokenKind::Variable(v) => v.pretty(buf),
            TokenKind::Label(l) => l.pretty(buf),
            TokenKind::LabelColon(l) => l.pretty(buf),
            TokenKind::Space(s) => s.pretty(buf),
            TokenKind::LineTerm(l) => l.pretty(buf),
            TokenKind::Eof(e) => e.pretty(buf),
            TokenKind::InstSep(i) => i.pretty(buf),
            TokenKind::ArgSep(a) => a.pretty(buf),
            TokenKind::LineComment(l) => l.pretty(buf),
            TokenKind::BlockComment(b) => b.pretty(buf),
            TokenKind::Word(w) => w.pretty(buf),
            TokenKind::Quoted(q) => q.pretty(buf),
            TokenKind::Spliced(s) => s.pretty(buf),
            TokenKind::Error(e) => e.pretty(buf),
            TokenKind::Placeholder => panic!("placeholder"),
        }
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

impl Pretty for &[u8] {
    fn pretty(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self);
    }
}

impl Pretty for Cow<'_, [u8]> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_ref());
    }
}

impl Pretty for &str {
    fn pretty(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}
