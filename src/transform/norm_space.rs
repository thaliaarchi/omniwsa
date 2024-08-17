//! A transformation that normalizes whitespace.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, Opcode},
    tokens::{
        spaces::{SpaceToken, Spaces},
        Token, TokenKind,
    },
    transform::Visitor,
};

// TODO:
// - Adapt allowed indentation according to dialect.
// - Multi-instruction lines are incorrectly indented.
// - Fold multiple adjacent blank lines and add a final line terminator when
//   missing.
// - Handle styles with not exactly one instruction per line. trim_leading and
//   trim_trailing are not sufficient, because multiple lines can be in the same
//   Space.

impl<'s> Cst<'s> {
    /// Normalizes whitespace. Indentation is normalized to `indent`, except for
    /// labels, which are unindented. Trailing whitespace is stripped.
    pub fn normalize_whitespace(&mut self, indent: Cow<'s, [u8]>) {
        assert!(indent.iter().all(|&b| b == b' ' || b == b'\t'));
        self.visit(&mut SpaceVisitor { indent });
    }
}

struct SpaceVisitor<'s> {
    indent: Cow<'s, [u8]>,
}

impl<'s> Visitor<'s> for SpaceVisitor<'s> {
    fn visit_inst(&mut self, inst: &mut Inst<'s>) {
        inst.words.leading_spaces_mut().trim_leading();
        if inst.opcode() != Opcode::Label {
            let indent = Token::new(self.indent.clone(), SpaceToken::from(self.indent.clone()));
            inst.words.leading_spaces_mut().push_front(indent);
        }
        let trailing = inst.words.trailing_spaces_mut();
        trailing.trim_trailing();
        if let Some(tok) = trailing.tokens_mut().last_mut() {
            if matches!(tok.kind, TokenKind::LineComment(_)) {
                tok.line_comment_trim_trailing();
            }
        }
    }

    fn visit_empty(&mut self, empty: &mut Spaces<'s>) {
        let len_before = empty.len();
        empty.trim_leading();
        if let Some(tok) = empty.tokens_mut().first_mut() {
            if matches!(tok.kind, TokenKind::LineComment(_)) {
                tok.line_comment_trim_trailing();
                if empty.len() != len_before {
                    let indent =
                        Token::new(self.indent.clone(), SpaceToken::from(self.indent.clone()));
                    empty.push_front(indent);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;

    use crate::{
        dialects::Burghard,
        syntax::{Cst, Dialect, Inst, Opcode},
        tokens::{
            comment::{BlockCommentStyle, BlockCommentToken, LineCommentStyle, LineCommentToken},
            integer::{Integer, IntegerToken},
            label::LabelToken,
            spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken, Spaces},
            words::Words,
            Token,
        },
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
                    Cst::Empty(Spaces::from(vec![
                        Token::new(
                            b"; start",
                            LineCommentToken {
                                text: b" start",
                                style: LineCommentStyle::Semi,
                                errors: EnumSet::empty(),
                            },
                        ),
                        Token::new(b"\n", LineTermToken::from(LineTermStyle::Lf)),
                    ])),
                    Cst::Inst(Inst {
                        words: Words {
                            space_before: Spaces::new(),
                            words: vec![
                                (
                                    Token::new(b"label", Opcode::Label),
                                    Spaces::from(Token::new(b" ", SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::new(
                                        b"start",
                                        LabelToken {
                                            sigil: b"",
                                            label: b"start".into(),
                                            errors: EnumSet::empty(),
                                        },
                                    ),
                                    Spaces::from(Token::new(
                                        b"\n",
                                        LineTermToken::from(LineTermStyle::Lf),
                                    )),
                                ),
                            ],
                        },
                        valid_arity: true,
                        valid_types: true,
                    }),
                    Cst::Inst(Inst {
                        words: Words {
                            space_before: Spaces::from(vec![
                                Token::new(b"    ", SpaceToken::from(b"    ")),
                                Token::new(
                                    b"{-1-}",
                                    BlockCommentToken {
                                        text: b"1",
                                        style: BlockCommentStyle::Haskell,
                                        errors: EnumSet::empty(),
                                    },
                                ),
                                Token::new(b"  ", SpaceToken::from(b"  ")),
                            ]),
                            words: vec![
                                (
                                    Token::new(b"push", Opcode::Push),
                                    Spaces::from(Token::new(b" ", SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::new(
                                        b"1",
                                        IntegerToken {
                                            value: Integer::from(1),
                                            ..Default::default()
                                        },
                                    ),
                                    Spaces::from(Token::new(
                                        b"\n",
                                        LineTermToken::from(LineTermStyle::Lf),
                                    )),
                                ),
                            ],
                        },
                        valid_arity: true,
                        valid_types: true,
                    }),
                    Cst::Empty(Spaces::from(vec![
                        Token::new(b"    ", SpaceToken::from(b"    ")),
                        Token::new(
                            b"; 2",
                            LineCommentToken {
                                text: b" 2",
                                style: LineCommentStyle::Semi,
                                errors: EnumSet::empty(),
                            },
                        ),
                        Token::new(b"\n", LineTermToken::from(LineTermStyle::Lf)),
                    ])),
                    Cst::Inst(Inst {
                        words: Words {
                            space_before: Spaces::from(Token::new(
                                b"    ",
                                SpaceToken::from(b"    "),
                            )),
                            words: vec![
                                (
                                    Token::new(b"push", Opcode::Push),
                                    Spaces::from(Token::new(b" ", SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::new(
                                        b"2",
                                        IntegerToken {
                                            value: Integer::from(2),
                                            ..Default::default()
                                        },
                                    ),
                                    Spaces::from(vec![
                                        Token::new(
                                            b"{-2-}",
                                            BlockCommentToken {
                                                text: b"2",
                                                style: BlockCommentStyle::Haskell,
                                                errors: EnumSet::empty(),
                                            },
                                        ),
                                        Token::new(b"", EofToken),
                                    ]),
                                ),
                            ],
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
