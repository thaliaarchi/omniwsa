//! A transformation that normalizes strange, non-portable constructs.

use std::mem;

use crate::{
    syntax::{Cst, Inst},
    tokens::{spaces::Spaces, TokenKind, WordToken},
    transform::Visitor,
};

impl<'s> Cst<'s> {
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
            match word.kind {
                TokenKind::Quoted(_) => {
                    // Remove non-semantic quotes.
                    let TokenKind::Quoted(q) = mem::replace(&mut word.kind, WordToken.into())
                    else {
                        unreachable!();
                    };
                    *word = *q.inner;
                }
                TokenKind::Spliced(_) => {
                    // Move block comments out of a token splice to after it.
                    let (_, word, space_after) = inst.words.get_spaced_mut(i);
                    let TokenKind::Spliced(mut s) = mem::replace(&mut word.kind, WordToken.into())
                    else {
                        unreachable!();
                    };
                    s.tokens
                        .retain(|tok| matches!(tok.kind, TokenKind::BlockComment(_)));
                    s.tokens.append(&mut space_after.tokens);
                    space_after.tokens = s.tokens;
                    *word = *s.spliced;
                }
                _ => {}
            }
        }
    }

    fn visit_empty(&mut self, _empty: &mut Spaces<'s>) {}
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;

    use crate::{
        dialects::Burghard,
        syntax::{Cst, Dialect, Inst, Opcode},
        tokens::{
            comment::{BlockCommentStyle, BlockCommentToken},
            integer::{Integer, IntegerToken},
            spaces::{EofToken, SpaceToken, Spaces},
            words::Words,
            Token, TokenKind,
        },
    };

    macro_rules! block_comment(($text:literal) => {
        Token::new(
            // TODO: Use concat_bytes! once stabilized.
            concat!("{-", $text, "-}").as_bytes(),
            BlockCommentToken {
                text: $text.as_bytes(),
                style: BlockCommentStyle::Haskell,
                errors: EnumSet::empty(),
            },
        )
    });

    #[test]
    fn unquote_and_unsplice() {
        let src = b" {-h-}p{-e-}u{-l-}s{-l-}h{-o-} {-!-}\"42\"";
        let mut cst = Burghard::new().parse(src);
        cst.normalize_strange();
        let expect = Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![Cst::Inst(Inst {
                    words: Words {
                        space_before: Spaces::from(vec![
                            Token::new(b" ", SpaceToken::from(b" ")),
                            block_comment!("h"),
                        ]),
                        words: vec![
                            (
                                Token::new(b"push", Opcode::Push),
                                Spaces::from(vec![
                                    block_comment!("e"),
                                    block_comment!("l"),
                                    block_comment!("l"),
                                    block_comment!("o"),
                                    Token::new(b" ", SpaceToken::from(b" ")),
                                    block_comment!("!"),
                                ]),
                            ),
                            (
                                Token::new(
                                    b"42",
                                    TokenKind::Integer(IntegerToken {
                                        literal: b"42".into(),
                                        value: Integer::from(42),
                                        ..Default::default()
                                    }),
                                ),
                                Spaces::from(Token::new(b"", EofToken)),
                            ),
                        ],
                    },
                    valid_arity: true,
                    valid_types: true,
                })],
            }),
        };
        assert_eq!(cst, expect);
    }
}
