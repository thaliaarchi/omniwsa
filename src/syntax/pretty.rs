//! Pretty-printing for CST nodes.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, OptionBlock},
    tokens::{Token, spaces::Spaces, words::Words},
};

/// Pretty-prints this node as Whitespace assembly syntax.
pub trait Pretty {
    /// Pretty-prints this node as Whitespace assembly syntax, written to the
    /// given buffer.
    fn pretty(&self, buf: &mut Vec<u8>);
}

impl Pretty for Token<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match self {
            Token::Mnemonic(m) => m.pretty(buf),
            Token::Integer(i) => i.pretty(buf),
            Token::String(s) => s.pretty(buf),
            Token::Char(c) => c.pretty(buf),
            Token::Variable(v) => v.pretty(buf),
            Token::Label(l) => l.pretty(buf),
            Token::LabelColon(l) => l.pretty(buf),
            Token::Space(s) => s.pretty(buf),
            Token::LineTerm(l) => l.pretty(buf),
            Token::Eof(e) => e.pretty(buf),
            Token::InstSep(i) => i.pretty(buf),
            Token::ArgSep(a) => a.pretty(buf),
            Token::LineComment(l) => l.pretty(buf),
            Token::BlockComment(b) => b.pretty(buf),
            Token::Word(w) => w.pretty(buf),
            Token::Group(g) => g.pretty(buf),
            Token::Splice(s) => s.pretty(buf),
            Token::Error(e) => e.pretty(buf),
            Token::Placeholder => panic!("placeholder"),
        }
    }
}

impl Pretty for Cst<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match self {
            Cst::Inst(inst) => inst.pretty(buf),
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
