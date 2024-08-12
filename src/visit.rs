//! A visitor for traversing CST nodes.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, InstSep, Space},
    token::{Opcode, Token, TokenKind},
};

// TODO:
// - Adapt allowed indentation according to dialect.
// - Keep unindented comments unindented.

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

    /// Sets the indentation on every line to `indent` and unindents labels.
    pub fn set_indent(&mut self, indent: Cow<'s, [u8]>) {
        assert!(indent.iter().all(|&b| b == b' ' || b == b'\t'));
        self.visit(&mut IndentVisitor { indent });
    }
}

struct IndentVisitor<'s> {
    indent: Cow<'s, [u8]>,
}

impl<'s> Visitor<'s> for IndentVisitor<'s> {
    fn visit_inst(&mut self, inst: &mut Inst<'s>) {
        let indented = inst.opcode() != Opcode::Label;
        self.set_indent(&mut inst.space_before, indented);
    }

    fn visit_empty(&mut self, empty: &mut InstSep<'s>) {
        match empty {
            InstSep::LineTerm {
                space_before,
                line_comment,
                ..
            } => {
                self.set_indent(space_before, line_comment.is_some());
            }
            InstSep::Sep(_) => {}
        }
    }
}

impl<'s> IndentVisitor<'s> {
    fn set_indent(&self, space_before: &mut Space<'s>, indented: bool) {
        let tokens = &mut space_before.tokens;
        let i = tokens
            .iter()
            .position(|tok| tok.kind != TokenKind::Space)
            .unwrap_or(tokens.len());
        let indent = Token::new(self.indent.clone(), TokenKind::Space);
        if indented {
            if i == 0 {
                tokens.insert(0, indent);
            } else {
                tokens[0] = indent;
                tokens.drain(1..i);
            }
        } else {
            tokens.drain(..i);
        }
    }
}

#[cfg(test)]
mod tests {
    use rug::Integer;

    use crate::{
        dialects::Burghard,
        syntax::{ArgSep, Cst, Dialect, Inst, InstSep, Space},
        token::{IntegerBase, IntegerSign, Opcode, Token, TokenKind},
    };

    #[test]
    fn set_indent() {
        let src = b" label start \n \t{-1-}  push 1\npush 2\t";
        let mut cst = Burghard::new().parse(src);
        cst.set_indent(b"    ".into());
        let expect = Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![
                    Cst::Inst(Inst {
                        space_before: Space::new(),
                        opcode: Token::new(b"label", TokenKind::Opcode(Opcode::Label)),
                        args: vec![(
                            ArgSep::Space(Space::from(vec![Token::new(b" ", TokenKind::Space)])),
                            Token::new(
                                b"start",
                                TokenKind::Label {
                                    sigil: b"",
                                    label: b"start".into(),
                                },
                            ),
                        )],
                        inst_sep: InstSep::LineTerm {
                            space_before: Space::from(vec![Token::new(b" ", TokenKind::Space)]),
                            line_comment: None,
                            line_term: Token::new(b"\n", TokenKind::LineTerm),
                        },
                        valid_arity: true,
                        valid_types: true,
                    }),
                    Cst::Inst(Inst {
                        space_before: Space::from(vec![
                            Token::new(b"    ", TokenKind::Space),
                            Token::new(
                                b"{-1-}",
                                TokenKind::BlockComment {
                                    open: b"{-",
                                    text: b"1",
                                    close: b"-}",
                                    nested: true,
                                    terminated: true,
                                },
                            ),
                            Token::new(b"  ", TokenKind::Space),
                        ]),
                        opcode: Token::new(b"push", TokenKind::Opcode(Opcode::Push)),
                        args: vec![(
                            ArgSep::Space(Space::from(vec![Token::new(b" ", TokenKind::Space)])),
                            Token::new(
                                b"1",
                                TokenKind::Integer {
                                    value: Integer::from(1),
                                    sign: IntegerSign::None,
                                    base: IntegerBase::Decimal,
                                },
                            ),
                        )],
                        inst_sep: InstSep::LineTerm {
                            space_before: Space::new(),
                            line_comment: None,
                            line_term: Token::new(b"\n", TokenKind::LineTerm),
                        },
                        valid_arity: true,
                        valid_types: true,
                    }),
                    Cst::Inst(Inst {
                        space_before: Space::from(vec![Token::new(b"    ", TokenKind::Space)]),
                        opcode: Token::new(b"push", TokenKind::Opcode(Opcode::Push)),
                        args: vec![(
                            ArgSep::Space(Space::from(vec![Token::new(b" ", TokenKind::Space)])),
                            Token::new(
                                b"2",
                                TokenKind::Integer {
                                    value: Integer::from(2),
                                    sign: IntegerSign::None,
                                    base: IntegerBase::Decimal,
                                },
                            ),
                        )],
                        inst_sep: InstSep::LineTerm {
                            space_before: Space::from(vec![Token::new(b"\t", TokenKind::Space)]),
                            line_comment: None,
                            line_term: Token::new(b"", TokenKind::Eof),
                        },
                        valid_arity: true,
                        valid_types: true,
                    }),
                ],
            }),
        };
        assert_eq!(cst, expect);
    }
}
