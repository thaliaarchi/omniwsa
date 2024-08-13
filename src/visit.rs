//! A visitor for traversing CST nodes.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, InstSep},
    token::{Opcode, Token, TokenKind},
};

// TODO:
// - Adapt allowed indentation according to dialect.
// - Multi-instruction lines are incorrectly indented.
// - Fold multiple adjacent blank lines and add a final line terminator when
//   missing.

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

    /// Normalizes whitespace. Indentation is normalized to `indent`, except for
    /// labels, which are unindented. Trailing whitespace is stripped.
    pub fn normalize_whitespace(&mut self, indent: Cow<'s, [u8]>) {
        assert!(indent.iter().all(|&b| b == b' ' || b == b'\t'));
        self.visit(&mut IndentVisitor { indent });
    }
}

struct IndentVisitor<'s> {
    indent: Cow<'s, [u8]>,
}

impl<'s> Visitor<'s> for IndentVisitor<'s> {
    fn visit_inst(&mut self, inst: &mut Inst<'s>) {
        inst.space_before.trim_leading();
        if inst.opcode() != Opcode::Label {
            let indent = Token::new(self.indent.clone(), TokenKind::Space);
            inst.space_before.tokens.insert(0, indent);
        }
        match &mut inst.inst_sep {
            InstSep::LineTerm {
                space_before,
                line_comment,
                ..
            } => {
                if let Some(line_comment) = line_comment {
                    line_comment.line_comment_trim_trailing();
                } else {
                    space_before.trim_trailing();
                }
            }
            InstSep::Sep(_) => {}
        }
    }

    fn visit_empty(&mut self, empty: &mut InstSep<'s>) {
        match empty {
            InstSep::LineTerm {
                space_before,
                line_comment,
                ..
            } => {
                let len = space_before.tokens.len();
                space_before.trim_leading();
                if let Some(line_comment) = line_comment {
                    if space_before.tokens.len() != len {
                        let indent = Token::new(self.indent.clone(), TokenKind::Space);
                        space_before.tokens.insert(0, indent);
                    }
                    line_comment.line_comment_trim_trailing();
                }
            }
            InstSep::Sep(_) => {}
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
    fn normalize_whitespace() {
        let src = b"; start \n label start \n \t{-1-}  push 1\n ; 2\npush 2{-2-}\t";
        let mut cst = Burghard::new().parse(src);
        cst.normalize_whitespace(b"    ".into());
        let expect = Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![
                    Cst::Empty(InstSep::LineTerm {
                        space_before: Space::new(),
                        line_comment: Some(Token::new(
                            b"; start",
                            TokenKind::LineComment {
                                prefix: b";",
                                text: b" start",
                            },
                        )),
                        line_term: Token::new(b"\n", TokenKind::LineTerm),
                    }),
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
                            space_before: Space::new(),
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
                    Cst::Empty(InstSep::LineTerm {
                        space_before: Space::from(vec![Token::new(b"    ", TokenKind::Space)]),
                        line_comment: Some(Token::new(
                            b"; 2",
                            TokenKind::LineComment {
                                prefix: b";",
                                text: b" 2",
                            },
                        )),
                        line_term: Token::new(b"\n", TokenKind::LineTerm),
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
                            space_before: Space::from(vec![Token::new(
                                b"{-2-}",
                                TokenKind::BlockComment {
                                    open: b"{-",
                                    text: b"2",
                                    close: b"-}",
                                    nested: true,
                                    terminated: true,
                                },
                            )]),
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
