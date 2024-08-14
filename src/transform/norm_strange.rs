//! A transformation that normalizes strange, non-portable constructs.

use std::mem;

use crate::{
    syntax::{Cst, Inst, InstSep, Space},
    token::{Token, TokenKind},
    visit::Visitor,
};

// TODO:
// - Normalize Burghard quoted words.

impl<'s> Cst<'s> {
    /// Normalizes strange, non-portable constructs.
    ///
    /// - Moves block comments out of a token splice to after it (Burghard).
    pub fn normalize_strange(&mut self) {
        self.visit(&mut StrangeVisitor);
    }
}

struct StrangeVisitor;

impl<'s> Visitor<'s> for StrangeVisitor {
    fn visit_inst(&mut self, inst: &mut Inst<'s>) {
        if matches!(inst.opcode.kind, TokenKind::Spliced { .. }) {
            unsplice(inst.opcode_space_after_mut());
        }
        for arg in 0..inst.args.len() {
            if matches!(inst.args[arg].1.kind, TokenKind::Spliced { .. }) {
                unsplice(inst.arg_space_after_mut(arg));
            }
        }
    }

    fn visit_empty(&mut self, _empty: &mut InstSep<'s>) {}
}

/// Moves block comments out of a token splice to after it.
fn unsplice<'s>((word, space_after): (&mut Token<'s>, &mut Space<'s>)) {
    let TokenKind::Spliced {
        mut tokens,
        spliced,
    } = mem::replace(&mut word.kind, TokenKind::Word)
    else {
        panic!("not spliced");
    };
    tokens.retain(|tok| matches!(tok.kind, TokenKind::BlockComment { .. }));
    tokens.append(&mut space_after.tokens);
    space_after.tokens = tokens;
    *word = *spliced;
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;
    use rug::Integer;

    use crate::{
        dialects::Burghard,
        syntax::{ArgSep, Cst, Dialect, Inst, InstSep, Space},
        token::{IntegerBase, IntegerSign, IntegerToken, Opcode, Token, TokenKind},
    };

    macro_rules! block_comment(($text:literal) => {
        Token::new(
            // TODO: Use concat_bytes! once stabilized.
            concat!("{-", $text, "-}").as_bytes(),
            TokenKind::BlockComment {
                open: b"{-",
                text: $text.as_bytes(),
                close: b"-}",
                nested: true,
                terminated: true,
            },
        )
    });

    #[test]
    fn unsplice() {
        let src = b" {-h-}p{-e-}u{-l-}s{-l-}h{-o-} {-!-}42";
        let mut cst = Burghard::new().parse(src);
        cst.normalize_strange();
        let expect = Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![Cst::Inst(Inst {
                    space_before: Space::from(vec![
                        Token::new(b" ", TokenKind::Space),
                        block_comment!("h"),
                    ]),
                    opcode: Token::new(b"push", TokenKind::Opcode(Opcode::Push)),
                    args: vec![(
                        ArgSep::Space(Space::from(vec![
                            block_comment!("e"),
                            block_comment!("l"),
                            block_comment!("l"),
                            block_comment!("o"),
                            Token::new(b" ", TokenKind::Space),
                            block_comment!("!"),
                        ])),
                        Token::new(
                            b"42",
                            TokenKind::Integer(IntegerToken {
                                value: Integer::from(42),
                                sign: IntegerSign::None,
                                base: IntegerBase::Decimal,
                                leading_zeros: 0,
                                errors: EnumSet::empty(),
                            }),
                        ),
                    )],
                    inst_sep: InstSep::LineTerm {
                        space_before: Space::new(),
                        line_comment: None,
                        line_term: Token::new(b"", TokenKind::Eof),
                    },
                    valid_arity: true,
                    valid_types: true,
                })],
            }),
        };
        assert_eq!(cst, expect);
    }
}
