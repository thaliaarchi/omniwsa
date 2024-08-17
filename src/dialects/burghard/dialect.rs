//! Parsing for the Burghard Whitespace assembly dialect.

use std::{collections::HashMap, str};

use crate::{
    dialects::burghard::{option::OptionNester, parse::Parser},
    syntax::{Cst, Opcode},
    tokens::mnemonics::Utf8LowerToAscii,
};

// TODO:
// - Move Cst macros to syntax.

/// State for parsing the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Burghard {
    mnemonics: HashMap<Utf8LowerToAscii<'static>, &'static [Opcode]>,
}

static MNEMONICS: &[(&str, &[Opcode])] = &[
    ("push", &[Opcode::Push]),
    ("pushs", &[Opcode::PushString0]),
    ("doub", &[Opcode::Dup]),
    ("swap", &[Opcode::Swap]),
    ("pop", &[Opcode::Drop]),
    ("add", &[Opcode::Add, Opcode::AddConstRhs]),
    ("sub", &[Opcode::Sub, Opcode::SubConstRhs]),
    ("mul", &[Opcode::Mul, Opcode::MulConstRhs]),
    ("div", &[Opcode::Div, Opcode::DivConstRhs]),
    ("mod", &[Opcode::Mod, Opcode::ModConstRhs]),
    ("store", &[Opcode::Store, Opcode::StoreConstLhs]),
    ("retrive", &[Opcode::Retrieve, Opcode::RetrieveConst]),
    ("label", &[Opcode::Label]),
    ("call", &[Opcode::Call]),
    ("jump", &[Opcode::Jmp]),
    ("jumpz", &[Opcode::Jz]),
    ("jumpn", &[Opcode::Jn]),
    ("jumpp", &[Opcode::BurghardJmpP]),
    ("jumpnp", &[Opcode::BurghardJmpNP]),
    ("jumppn", &[Opcode::BurghardJmpNP]),
    ("jumpnz", &[Opcode::BurghardJmpNZ]),
    ("jumppz", &[Opcode::BurghardJmpPZ]),
    ("ret", &[Opcode::Ret]),
    ("exit", &[Opcode::End]),
    ("outC", &[Opcode::Printc]),
    ("outN", &[Opcode::Printi]),
    ("inC", &[Opcode::Readc]),
    ("inN", &[Opcode::Readi]),
    ("debug_printstack", &[Opcode::BurghardPrintStack]),
    ("debug_printheap", &[Opcode::BurghardPrintHeap]),
    ("test", &[Opcode::BurghardTest]),
    ("valueinteger", &[Opcode::BurghardValueInteger]),
    ("valuestring", &[Opcode::BurghardValueString]),
    ("include", &[Opcode::BurghardInclude]),
    ("option", &[Opcode::DefineOption]),
    ("ifoption", &[Opcode::IfOption]),
    ("elseifoption", &[Opcode::ElseIfOption]),
    ("elseoption", &[Opcode::ElseOption]),
    ("endoption", &[Opcode::EndOption]),
];

impl Burghard {
    /// Constructs state for the Burghard dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Burghard {
            mnemonics: MNEMONICS
                .iter()
                .map(|&(mnemonic, sigs)| (Utf8LowerToAscii(mnemonic.as_bytes()), sigs))
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Burghard dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        OptionNester::new().nest(&mut Parser::new(src, self))
    }

    /// Gets the overloaded opcodes for a mnemonic.
    pub(super) fn get_opcodes(&self, mnemonic: &[u8]) -> Option<&'static [Opcode]> {
        self.mnemonics.get(&Utf8LowerToAscii(mnemonic)).copied()
    }
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;

    use crate::{
        dialects::Burghard,
        syntax::{Cst, Dialect, Inst, Opcode, OptionBlock},
        tokens::{
            comment::{BlockCommentStyle, BlockCommentToken},
            integer::{Integer, IntegerToken},
            mnemonics::MnemonicToken,
            spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken, Spaces},
            string::{QuoteStyle, QuotedError, QuotedToken, StringData, StringToken},
            words::Words,
            SplicedToken, Token, WordToken,
        },
    };

    macro_rules! root[($($node:expr),* $(,)?) => {
        Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![$($node),*],
            }),
        }
    }];
    macro_rules! mnemonic(($mnemonic:expr, $opcode:expr $(,)?) => {
        Token::new(
            $mnemonic,
            MnemonicToken {
                mnemonic: $mnemonic.into(),
                opcode: $opcode,
            },
        )
    });
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
    macro_rules! space(($space:literal) => {
        Spaces::from(Token::new($space, SpaceToken::from($space)))
    });
    macro_rules! lf(() => {
        Spaces::from(Token::new(b"\n", LineTermToken::from(LineTermStyle::Lf)))
    });
    macro_rules! eof(() => {
        Spaces::from(Token::new(b"", EofToken))
    });

    #[test]
    fn spliced() {
        let src = b" {-c1-}hello{-splice-}world{-c2-}\t!";
        let cst = Burghard::new().parse(src);
        let expect = root![Cst::Inst(Inst {
            words: Words {
                space_before: Spaces::from(vec![
                    Token::new(b" ", SpaceToken::from(b" ")),
                    block_comment!("c1"),
                ]),
                words: vec![
                    (
                        Token::new(
                            b"hello{-splice-}world",
                            SplicedToken {
                                tokens: vec![
                                    Token::new(b"hello", WordToken),
                                    block_comment!("splice"),
                                    Token::new(b"world", WordToken),
                                ],
                                spliced: Box::new(mnemonic!(b"helloworld", Opcode::Invalid)),
                            },
                        ),
                        Spaces::from(vec![
                            block_comment!("c2"),
                            Token::new(b"\t", SpaceToken::from(b"\t")),
                        ]),
                    ),
                    (
                        Token::new(
                            b"!",
                            StringToken {
                                literal: b"!".into(),
                                unescaped: StringData::Utf8("!".into()),
                                quotes: QuoteStyle::Bare,
                                errors: EnumSet::empty(),
                            },
                        ),
                        eof!(),
                    ),
                ],
            },
            valid_arity: true,
            valid_types: true,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn mnemonic_utf8_folding() {
        let cst = Burghard::new().parse("\"Debug_PrİntStacK".as_bytes());
        let expect = root![Cst::Inst(Inst {
            words: Words {
                space_before: Spaces::new(),
                words: vec![(
                    Token::new(
                        "\"Debug_PrİntStacK".as_bytes(),
                        QuotedToken {
                            inner: Box::new(mnemonic!(
                                "Debug_PrİntStacK".as_bytes(),
                                Opcode::BurghardPrintStack,
                            )),
                            quotes: QuoteStyle::Double,
                            errors: QuotedError::Unterminated.into(),
                        },
                    ),
                    eof!(),
                )],
            },
            valid_arity: true,
            valid_types: true,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn bad_args() {
        let cst = Burghard::new().parse(b"valueinteger \"1\" \"2\"");
        let expect = root![Cst::Inst(Inst {
            words: Words {
                space_before: Spaces::new(),
                words: vec![
                    (
                        mnemonic!(b"valueinteger", Opcode::BurghardValueInteger),
                        space!(b" "),
                    ),
                    (
                        Token::new(
                            b"\"1\"",
                            StringToken {
                                literal: b"1".into(),
                                unescaped: StringData::Utf8("1".into()),
                                quotes: QuoteStyle::Double,
                                errors: EnumSet::empty(),
                            },
                        ),
                        space!(b" "),
                    ),
                    (
                        Token::new(
                            b"\"2\"",
                            QuotedToken {
                                inner: Box::new(Token::new(
                                    b"2",
                                    IntegerToken {
                                        literal: b"2".into(),
                                        value: Integer::from(2),
                                        ..Default::default()
                                    },
                                )),
                                quotes: QuoteStyle::Double,
                                errors: EnumSet::empty(),
                            },
                        ),
                        eof!(),
                    ),
                ],
            },
            valid_arity: true,
            valid_types: false,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn option_blocks() {
        macro_rules! letter(($letter:literal) => {
            Cst::Inst(Inst {
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!($letter, Opcode::Invalid), lf!())],
                },
                valid_arity: true,
                valid_types: true,
            })
        });
        macro_rules! ifoption(($option:literal) => {
            Inst {
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![
                        (mnemonic!(b"ifoption", Opcode::IfOption), space!(b" ")),
                        (Token::new($option, WordToken), lf!()),
                    ],
                },
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! elseifoption(($option:literal) => {
            Inst {
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![
                        (mnemonic!(b"elseifoption", Opcode::ElseIfOption), space!(b" ")),
                        (Token::new($option, WordToken), lf!()),
                    ],
                },
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! elseoption(() => {
            Inst {
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!(b"elseoption", Opcode::ElseOption), lf!())],
                },
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! endoption(() => {
            Inst {
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!(b"endoption", Opcode::EndOption), lf!())],
                },
                valid_arity: true,
                valid_types: true,
            }
        });
        let src = b"a
endoption
b
ifoption x
c
elseoption
d
elseifoption y
e
endoption
f
endoption
g
elseoption
";
        let cst = Burghard::new().parse(src);
        let expect = root![
            letter!(b"a"),
            Cst::OptionBlock(OptionBlock {
                options: vec![],
                end: Some(endoption!()),
            }),
            letter!(b"b"),
            Cst::OptionBlock(OptionBlock {
                options: vec![
                    (ifoption!(b"x"), vec![letter!(b"c")]),
                    (elseoption!(), vec![letter!(b"d")]),
                    (elseifoption!(b"y"), vec![letter!(b"e")]),
                ],
                end: Some(endoption!()),
            }),
            letter!(b"f"),
            Cst::OptionBlock(OptionBlock {
                options: vec![],
                end: Some(endoption!()),
            }),
            letter!(b"g"),
            Cst::OptionBlock(OptionBlock {
                options: vec![(elseoption!(), vec![])],
                end: None,
            }),
        ];
        assert_eq!(cst, expect);
    }
}
