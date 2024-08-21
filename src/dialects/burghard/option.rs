//! Structuring of option instructions into blocks.

use std::mem;

use crate::{
    dialects::burghard::parse::Parser,
    syntax::{Cst, Dialect, Opcode, OptionBlock},
};

/// A builder, which structures options into blocks.
#[derive(Clone, Debug)]
pub struct OptionNester<'s> {
    root: Vec<Cst<'s>>,
    option_stack: Vec<OptionBlock<'s>>,
}

impl<'s> OptionNester<'s> {
    /// Constructs a builder, which structures options into blocks.
    pub fn new() -> Self {
        OptionNester {
            root: Vec::new(),
            option_stack: Vec::new(),
        }
    }

    /// Nests instructions into structured option blocks.
    pub fn nest(&mut self, parser: &mut Parser<'s, '_>) -> Cst<'s> {
        while let Some(inst) = parser.next() {
            match inst.opcode {
                Opcode::IfOption => {
                    self.option_stack.push(OptionBlock {
                        options: vec![(inst, Vec::new())],
                        end: None,
                    });
                }
                Opcode::ElseIfOption | Opcode::ElseOption => match self.option_stack.last_mut() {
                    Some(block) => {
                        block.options.push((inst, Vec::new()));
                    }
                    None => {
                        self.option_stack.push(OptionBlock {
                            options: vec![(inst, Vec::new())],
                            end: None,
                        });
                    }
                },
                Opcode::EndOption => match self.option_stack.pop() {
                    Some(mut block) => {
                        block.end = Some(inst);
                        self.curr_block().push(Cst::OptionBlock(block));
                    }
                    None => {
                        self.root.push(Cst::OptionBlock(OptionBlock {
                            options: Vec::new(),
                            end: Some(inst),
                        }));
                    }
                },
                _ => self.curr_block().push(Cst::Inst(inst)),
            }
        }
        let mut parent = &mut self.root;
        for block in self.option_stack.drain(..) {
            parent.push(Cst::OptionBlock(block));
            let Cst::OptionBlock(last) = parent.last_mut().unwrap() else {
                unreachable!();
            };
            parent = &mut last.options.last_mut().unwrap().1;
        }
        let nodes = mem::take(&mut self.root);
        Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block { nodes }),
        }
    }

    /// Returns the current block for instructions to be inserted into.
    fn curr_block(&mut self) -> &mut Vec<Cst<'s>> {
        match self.option_stack.last_mut() {
            Some(last) => &mut last.options.last_mut().unwrap().1,
            None => &mut self.root,
        }
    }
}
