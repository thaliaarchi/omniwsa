//! Code generation for Whitespace assembly instructions.

use std::collections::HashSet;

use bstr::ByteSlice;
use rug::integer::MiniInteger;

use crate::{
    codegen::{Inst, IntegerBits, LabelBits, TokenWrite},
    syntax::{Cst, Inst as WsaInst, Opcode},
    tokens::{
        integer::Integer,
        string::{Encoding, StringToken},
        Token, WordToken,
    },
};

// TODO:
// - Handle anything beyond integer literal arguments. Resolve labels.
// - Validate arities.
// - Handle options more robustly.
// - Create an InstStream abstraction, which can be used to wrap tokenwrite,
//   but is useful on its own.
// - Configure PalaiologosRep count upper bound:
//   - Use a loop when it would be shorter.
//   - Use a loop when over some configurable limit, e.g., 10 or i32::MAX like
//     Palaiologos.

impl Cst<'_> {
    /// Generates a stream of Whitespace tokens for this CST.
    pub fn codegen<T: TokenWrite>(
        &self,
        w: &mut T,
        options: &HashSet<&[u8]>,
    ) -> Result<(), T::Error> {
        match self {
            Cst::Inst(inst) => inst.codegen(w),
            Cst::Block { nodes } => {
                for node in nodes {
                    node.codegen(w, options)?;
                }
                Ok(())
            }
            Cst::OptionBlock(block) => {
                for (inst, block) in &block.options {
                    let block = match inst.opcode {
                        Opcode::IfOption | Opcode::ElseIfOption => {
                            if !options.contains(&inst.word(0).word[..]) {
                                continue;
                            }
                            block
                        }
                        Opcode::ElseOption => block,
                        _ => todo!(),
                    };
                    for node in block {
                        node.codegen(w, options)?;
                    }
                }
                Ok(())
            }
            Cst::Dialect { inner, .. } => inner.codegen(w, options),
        }
    }
}

impl<'s> WsaInst<'s> {
    /// Generates a stream of Whitespace tokens for this instruction.
    pub fn codegen<T: TokenWrite>(&self, w: &mut T) -> Result<(), T::Error> {
        match self.opcode {
            Opcode::Push => w.write_inst(Inst::Push(self.integer(0))),
            Opcode::Dup => w.write_inst(Inst::Dup),
            Opcode::Copy => w.write_inst(Inst::Copy(self.integer(0))),
            Opcode::Swap => w.write_inst(Inst::Swap),
            Opcode::Drop => w.write_inst(Inst::Drop),
            Opcode::Slide => w.write_inst(Inst::Slide(self.integer(0))),
            Opcode::Add => w.write_inst(Inst::Add),
            Opcode::Sub => w.write_inst(Inst::Sub),
            Opcode::Mul => w.write_inst(Inst::Mul),
            Opcode::Div => w.write_inst(Inst::Div),
            Opcode::Mod => w.write_inst(Inst::Mod),
            Opcode::Store => w.write_inst(Inst::Store),
            Opcode::Retrieve => w.write_inst(Inst::Retrieve),
            Opcode::Label => w.write_inst(Inst::Label(self.label(0))),
            Opcode::Call => w.write_inst(Inst::Call(self.label(0))),
            Opcode::Jmp => w.write_inst(Inst::Jmp(self.label(0))),
            Opcode::Jz => w.write_inst(Inst::Jz(self.label(0))),
            Opcode::Jn => w.write_inst(Inst::Jn(self.label(0))),
            Opcode::Ret => w.write_inst(Inst::Ret),
            Opcode::End => w.write_inst(Inst::End),
            Opcode::Printc => w.write_inst(Inst::Printc),
            Opcode::Printi => w.write_inst(Inst::Printi),
            Opcode::Readc => w.write_inst(Inst::Readc),
            Opcode::Readi => w.write_inst(Inst::Readi),
            Opcode::BurghardPrintStack => w.write_inst(Inst::BurghardPrintStack),
            Opcode::BurghardPrintHeap => w.write_inst(Inst::BurghardPrintHeap),
            Opcode::VolivaOr => w.write_inst(Inst::VolivaOr),
            Opcode::VolivaNot => w.write_inst(Inst::VolivaNot),
            Opcode::VolivaAnd => w.write_inst(Inst::VolivaAnd),
            Opcode::VolivaBreakpoint => w.write_inst(Inst::VolivaBreakpoint),
            Opcode::Push0 => w.write_inst(Inst::Push((&Integer::ZERO).into())),
            Opcode::AddConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Add)
            }
            Opcode::SubConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Sub)
            }
            Opcode::MulConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Mul)
            }
            Opcode::DivConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Div)
            }
            Opcode::ModConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Mod)
            }
            Opcode::VolivaOrConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::VolivaOr)
            }
            Opcode::VolivaAndConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::VolivaAnd)
            }
            Opcode::AddRefRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Add)
            }
            Opcode::SubRefRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Sub)
            }
            Opcode::MulRefRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mul)
            }
            Opcode::DivRefRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Div)
            }
            Opcode::ModRefRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mod)
            }
            Opcode::AddRefConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Add)
            }
            Opcode::SubRefConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Sub)
            }
            Opcode::MulRefConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Mul)
            }
            Opcode::DivRefConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Div)
            }
            Opcode::ModRefConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Mod)
            }
            Opcode::AddConstRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Add)
            }
            Opcode::SubConstRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Sub)
            }
            Opcode::MulConstRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mul)
            }
            Opcode::DivConstRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Div)
            }
            Opcode::ModConstRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mod)
            }
            Opcode::AddRefRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Add)
            }
            Opcode::SubRefRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Sub)
            }
            Opcode::MulRefRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mul)
            }
            Opcode::DivRefRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Div)
            }
            Opcode::ModRefRef => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Retrieve)?;
                w.write_inst(Inst::Mod)
            }
            Opcode::StoreConstLhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Swap)?;
                w.write_inst(Inst::Store)
            }
            Opcode::StoreConstRhs => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Store)
            }
            Opcode::StoreConstConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Push(self.integer(1)))?;
                w.write_inst(Inst::Store)
            }
            Opcode::RetrieveConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Retrieve)
            }
            Opcode::PrintcConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Printc)
            }
            Opcode::PrintiConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Printi)
            }
            Opcode::ReadcConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Readc)
            }
            Opcode::ReadiConst => {
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Readi)
            }
            Opcode::PushString => each_char(self.string(0), |c| w.write_inst(Inst::Push(c.into()))),
            Opcode::PushString0 => {
                each_char(self.string(0), |c| w.write_inst(Inst::Push(c.into())))?;
                w.write_inst(Inst::Push((&Integer::ZERO).into()))
            }
            Opcode::StoreString0 => each_char(self.string(0), |c| {
                w.write_inst(Inst::Dup)?;
                w.write_inst(Inst::Push(c.into()))?;
                w.write_inst(Inst::Store)?;
                w.write_inst(Inst::Push(Integer::ONE.into()))?;
                w.write_inst(Inst::Add)
            }),
            Opcode::BurghardJmpPos
            | Opcode::BurghardJmpNonZero
            | Opcode::BurghardJmpNonPos
            | Opcode::BurghardJmpNonNeg => todo!(),
            Opcode::VolivaJmpPos => {
                w.write_inst(Inst::Push((&Integer::ZERO).into()))?;
                w.write_inst(Inst::Swap)?;
                w.write_inst(Inst::Sub)?;
                w.write_inst(Inst::Jn(self.label(0)))
            }
            Opcode::VolivaJmpNonZero => todo!(),
            Opcode::VolivaJmpNonPos => {
                w.write_inst(Inst::Push(Integer::ONE.into()))?;
                w.write_inst(Inst::Sub)?;
                w.write_inst(Inst::Jn(self.label(0)))
            }
            Opcode::VolivaJmpNonNeg => todo!(),
            Opcode::BurghardTest => {
                w.write_inst(Inst::Dup)?;
                w.write_inst(Inst::Push(self.integer(0)))?;
                w.write_inst(Inst::Sub)
            }
            Opcode::PalaiologosRep => {
                let opcode = match self.arg(0) {
                    Token::Mnemonic(m) => m.opcode,
                    arg => panic!("not a mnemonic: {arg:?}"),
                };
                let count = match self.arg(1) {
                    Token::Integer(count) => &count.value,
                    arg => panic!("not an integer: {arg:?}"),
                };
                let inst = match opcode {
                    Opcode::Dup => Inst::Dup,
                    Opcode::Swap => Inst::Swap,
                    Opcode::Drop => Inst::Drop,
                    Opcode::Add => Inst::Add,
                    Opcode::Sub => Inst::Sub,
                    Opcode::Mul => Inst::Mul,
                    Opcode::Div => Inst::Div,
                    Opcode::Mod => Inst::Mod,
                    Opcode::Store => Inst::Store,
                    Opcode::Retrieve => Inst::Retrieve,
                    Opcode::Ret => Inst::Ret,
                    Opcode::End => Inst::End,
                    Opcode::Printc => Inst::Printc,
                    Opcode::Printi => Inst::Printi,
                    Opcode::Readc => Inst::Readc,
                    Opcode::Readi => Inst::Readi,
                    Opcode::BurghardPrintStack => Inst::BurghardPrintStack,
                    Opcode::BurghardPrintHeap => Inst::BurghardPrintHeap,
                    Opcode::VolivaOr => Inst::VolivaOr,
                    Opcode::VolivaNot => Inst::VolivaNot,
                    Opcode::VolivaAnd => Inst::VolivaAnd,
                    Opcode::VolivaBreakpoint => Inst::VolivaBreakpoint,
                    opcode => panic!("unsupported opcode for `rep`: {opcode:?}"),
                };
                let count = if count.is_negative() {
                    0
                } else if let Some(count) = count.to_usize() {
                    count
                } else {
                    panic!("too many repetitions");
                };
                for _ in 0..count {
                    w.write_inst(inst.clone())?;
                }
                Ok(())
            }
            Opcode::BurghardInclude
            | Opcode::RespaceInclude
            | Opcode::VolivaInclude
            | Opcode::WhitelipsInclude
            | Opcode::DefineOption
            | Opcode::IfOption
            | Opcode::ElseIfOption
            | Opcode::ElseOption
            | Opcode::EndOption
            | Opcode::BurghardValueInteger
            | Opcode::BurghardValueString
            | Opcode::VolivaValueInteger
            | Opcode::VolivaValueString => todo!(),
            Opcode::Nop => Ok(()),
            Opcode::Invalid => panic!("invalid instruction: {self:?}"),
        }
    }

    /// Gets the value of the indexed argument as an integer.
    fn integer(&self, index: usize) -> IntegerBits<'_> {
        match self.arg(index).ungroup() {
            Token::Integer(int) => IntegerBits::from(&int.value),
            arg => panic!("not an integer: {arg:?}"),
        }
    }

    /// Gets the value of the indexed argument as a label.
    fn label(&self, index: usize) -> LabelBits<'_> {
        match self.arg(index).ungroup() {
            Token::Integer(int) => LabelBits::from(&int.value),
            arg => panic!("not an integer label: {arg:?}"),
        }
    }

    /// Gets the value of the indexed argument as a string.
    fn string(&self, index: usize) -> &StringToken<'s> {
        match self.arg(index).ungroup() {
            Token::String(s) => s,
            arg => panic!("not a string: {arg:?}"),
        }
    }

    /// Gets the value of the indexed argument as a word.
    fn word(&self, index: usize) -> &WordToken<'s> {
        match self.arg(index).ungroup() {
            Token::Word(w) => w,
            arg => panic!("not a word: {arg:?}"),
        }
    }
}

/// Iterates the chars or bytes in the string literal.
fn each_char<E, F: FnMut(&Integer) -> Result<(), E>>(
    s: &StringToken<'_>,
    mut f: F,
) -> Result<(), E> {
    let mut int;
    match s.encoding {
        Encoding::Utf8 => {
            for ch in s.unescaped.as_bstr().chars().rev() {
                int = MiniInteger::from(ch as u32);
                f(int.borrow_excl())?;
            }
        }
        Encoding::Bytes => {
            for &b in s.unescaped.iter().rev() {
                int = MiniInteger::from(b);
                f(int.borrow_excl())?;
            }
        }
    }
    Ok(())
}
