//! A visitor for traversing CST nodes.

use crate::syntax::{Cst, Inst, InstSep};

// TODO:
// - Create pass that transforms instructions equivalent to a macro expansion
//   into the macro. For example, `push n add` => Burghard `add n` or
//   `jz l1 jmp l2 l1:` => Burghard `jumpnp` if `l1` is otherwise unused.
// - Creates pass that normalizes mnemonics.

/// A visitor for traversing nodes in a `Cst`.
pub trait Visitor<'s> {
    /// Called when an instruction is visited.
    fn visit_inst(&mut self, _inst: &mut Inst<'s>) {}

    /// Called when an instruction separator is visited.
    fn visit_empty(&mut self, _empty: &mut InstSep<'s>) {}
}

impl<'s> Cst<'s> {
    /// Traverses this `Cst` and calls the visitor when kinds of nodes are
    /// encountered.
    pub fn visit<V: Visitor<'s>>(&mut self, visitor: &mut V) {
        match self {
            Cst::Inst(inst) => visitor.visit_inst(inst),
            Cst::Empty(empty) => visitor.visit_empty(empty),
            Cst::Block { nodes } => {
                nodes.iter_mut().for_each(|node| node.visit(visitor));
            }
            Cst::OptionBlock(block) => {
                block.options.iter_mut().for_each(|(option, block)| {
                    visitor.visit_inst(option);
                    block.iter_mut().for_each(|node| node.visit(visitor));
                });
                block.end.as_mut().map(|end| visitor.visit_inst(end));
            }
            Cst::Dialect { dialect: _, inner } => inner.visit(visitor),
        }
    }
}
