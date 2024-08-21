//! A transformation that normalizes whitespace.

use std::borrow::Cow;

use crate::{
    syntax::{Cst, Inst, Opcode},
    tokens::{spaces::SpaceToken, Token},
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
        let leading = inst.words.leading_spaces_mut();
        let len_before = leading.len();
        leading.trim_leading();
        let mut should_indent = inst.opcode != Opcode::Label;
        if let Some(Token::LineComment(comment)) = leading.tokens_mut().first_mut() {
            comment.trim_trailing();
            should_indent = leading.len() != len_before;
        }

        let trailing = inst.words.trailing_spaces_mut();
        trailing.trim_trailing();
        if let Some(Token::LineComment(comment)) = trailing.tokens_mut().last_mut() {
            comment.trim_trailing();
        }

        if should_indent {
            let indent = Token::from(SpaceToken::from(self.indent.clone()));
            inst.words.leading_spaces_mut().push_front(indent);
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
            label::{LabelStyle, LabelToken},
            mnemonics::MnemonicToken,
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
                    Cst::Inst(Inst::nop(Spaces::from(vec![
                        Token::from(LineCommentToken {
                            text: b" start",
                            style: LineCommentStyle::Semi,
                            errors: EnumSet::empty(),
                        }),
                        Token::from(LineTermToken::from(LineTermStyle::Lf)),
                    ]))),
                    Cst::Inst(Inst {
                        opcode: Opcode::Label,
                        words: Words {
                            space_before: Spaces::new(),
                            words: vec![
                                (
                                    Token::from(MnemonicToken {
                                        mnemonic: b"label".into(),
                                        opcode: Opcode::Label,
                                    }),
                                    Spaces::from(Token::from(SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::from(LabelToken {
                                        label: b"start".into(),
                                        style: LabelStyle::NoSigil,
                                        errors: EnumSet::empty(),
                                    }),
                                    Spaces::from(Token::from(LineTermToken::from(
                                        LineTermStyle::Lf,
                                    ))),
                                ),
                            ],
                        },
                        errors: EnumSet::empty(),
                    }),
                    Cst::Inst(Inst {
                        opcode: Opcode::Push,
                        words: Words {
                            space_before: Spaces::from(vec![
                                Token::from(SpaceToken::from(b"    ")),
                                Token::from(BlockCommentToken {
                                    text: b"1",
                                    style: BlockCommentStyle::Haskell,
                                    errors: EnumSet::empty(),
                                }),
                                Token::from(SpaceToken::from(b"  ")),
                            ]),
                            words: vec![
                                (
                                    Token::from(MnemonicToken {
                                        mnemonic: b"push".into(),
                                        opcode: Opcode::Push,
                                    }),
                                    Spaces::from(Token::from(SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::from(IntegerToken {
                                        literal: b"1".into(),
                                        value: Integer::from(1),
                                        ..Default::default()
                                    }),
                                    Spaces::from(Token::from(LineTermToken::from(
                                        LineTermStyle::Lf,
                                    ))),
                                ),
                            ],
                        },
                        errors: EnumSet::empty(),
                    }),
                    Cst::Inst(Inst::nop(Spaces::from(vec![
                        Token::from(SpaceToken::from(b"    ")),
                        Token::from(LineCommentToken {
                            text: b" 2",
                            style: LineCommentStyle::Semi,
                            errors: EnumSet::empty(),
                        }),
                        Token::from(LineTermToken::from(LineTermStyle::Lf)),
                    ]))),
                    Cst::Inst(Inst {
                        opcode: Opcode::Push,
                        words: Words {
                            space_before: Spaces::from(Token::from(SpaceToken::from(b"    "))),
                            words: vec![
                                (
                                    Token::from(MnemonicToken {
                                        mnemonic: b"push".into(),
                                        opcode: Opcode::Push,
                                    }),
                                    Spaces::from(Token::from(SpaceToken::from(b" "))),
                                ),
                                (
                                    Token::from(IntegerToken {
                                        literal: b"2".into(),
                                        value: Integer::from(2),
                                        ..Default::default()
                                    }),
                                    Spaces::from(vec![
                                        Token::from(BlockCommentToken {
                                            text: b"2",
                                            style: BlockCommentStyle::Haskell,
                                            errors: EnumSet::empty(),
                                        }),
                                        Token::from(EofToken),
                                    ]),
                                ),
                            ],
                        },
                        errors: EnumSet::empty(),
                    }),
                ],
            }),
        };
        assert_eq!(cst, expect);
    }
}
