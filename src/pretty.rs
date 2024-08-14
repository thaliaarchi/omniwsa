// Pretty-printing for CST nodes.

use crate::{
    syntax::{ArgSep, Cst, Inst, InstSep, OptionBlock, Space, Spaced},
    tokens::Token,
};

/// Pretty-prints this node as Whitespace assembly syntax.
pub trait Pretty {
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
        self.space_before.pretty(buf);
        self.opcode.pretty(buf);
        self.args.iter().for_each(|(sep, arg)| {
            sep.pretty(buf);
            arg.pretty(buf);
        });
        self.inst_sep.pretty(buf);
    }
}

impl Pretty for Space<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.tokens.iter().for_each(|tok| tok.pretty(buf))
    }
}

impl<T: Pretty> Pretty for Spaced<'_, T> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.space_before.pretty(buf);
        self.inner.pretty(buf);
        self.space_after.pretty(buf);
    }
}

impl Pretty for ArgSep<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match self {
            ArgSep::Space(space) => space.pretty(buf),
            ArgSep::Sep(sep) => sep.pretty(buf),
        }
    }
}

impl Pretty for InstSep<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        match self {
            InstSep::LineTerm {
                space_before,
                line_comment,
                line_term,
            } => {
                space_before.pretty(buf);
                line_comment.pretty(buf);
                line_term.pretty(buf);
            }
            InstSep::Sep(sep) => sep.pretty(buf),
        }
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
