//! A transformation that normalizes strange, non-portable constructs.

use std::mem;

use crate::{
    syntax::{Cst, Inst},
    tokens::Token,
    transform::Visitor,
};

impl Cst<'_> {
    /// Normalizes strange, non-portable constructs.
    ///
    /// - Removes non-semantic quotes (Burghard).
    /// - Moves block comments out of a token splice to after it (Burghard).
    pub fn normalize_strange(&mut self) {
        self.visit(&mut StrangeVisitor);
    }
}

struct StrangeVisitor;

impl<'s> Visitor<'s> for StrangeVisitor {
    fn visit_inst(&mut self, inst: &mut Inst<'s>) {
        for i in 0..inst.words.len() {
            let word = &mut inst.words.words[i].0;
            match word {
                Token::Group(_) => {
                    // Remove non-semantic groups.
                    let Token::Group(g) = mem::take(word) else {
                        unreachable!();
                    };
                    *word = *g.inner;
                }
                Token::Splice(_) => {
                    // Move block comments out of a token splice to after it.
                    let (_, word, space_after) = inst.words.get_spaced_mut(i);
                    let Token::Splice(mut s) = mem::take(word) else {
                        unreachable!();
                    };
                    s.tokens.retain(|tok| matches!(tok, Token::BlockComment(_)));
                    s.tokens.append(&mut space_after.tokens);
                    space_after.tokens = s.tokens;
                    *word = *s.spliced;
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;

    use crate::{
        dialects::{Burghard, Dialect as _},
        syntax::{ArgLayout, Cst, Inst, Opcode},
        tokens::{
            Token,
            comment::{BlockCommentStyle, BlockCommentToken},
            integer::{BaseStyle, Integer, IntegerToken, Sign},
            mnemonics::MnemonicToken,
            spaces::{EofToken, SpaceToken, Spaces},
            words::Words,
        },
    };

    macro_rules! block_comment(($text:literal) => {
        Token::from(BlockCommentToken {
            text: $text,
            style: BlockCommentStyle::Burghard,
            errors: EnumSet::empty(),
        })
    });

    #[test]
    fn unquote_and_unsplice() {
        let src = b" {-h-}p{-e-}u{-l-}s{-l-}h{-o-} {-!-}\"42\"";
        let mut cst = Burghard::new().parse(src);
        cst.normalize_strange();
        let expect = Cst::Block {
            nodes: vec![Cst::Inst(Inst {
                opcode: Opcode::Push,
                words: Words {
                    space_before: Spaces::from(vec![
                        Token::from(SpaceToken::from(b" ")),
                        block_comment!(b"h"),
                    ]),
                    words: vec![
                        (
                            Token::from(MnemonicToken {
                                mnemonic: b"push".into(),
                                opcode: Opcode::Push,
                            }),
                            Spaces::from(vec![
                                block_comment!(b"e"),
                                block_comment!(b"l"),
                                block_comment!(b"l"),
                                block_comment!(b"o"),
                                Token::from(SpaceToken::from(b" ")),
                                block_comment!(b"!"),
                            ]),
                        ),
                        (
                            Token::from(Token::Integer(IntegerToken {
                                literal: b"42".into(),
                                value: Integer::from(42),
                                sign: Sign::None,
                                base_style: BaseStyle::Decimal,
                                leading_zeros: 0,
                                has_digit_seps: false,
                                errors: EnumSet::empty(),
                            })),
                            Spaces::from(Token::from(EofToken)),
                        ),
                    ],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
            })],
        };
        assert_eq!(cst, expect);
    }
}
