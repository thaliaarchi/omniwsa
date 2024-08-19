//! Parsing for the Burghard Whitespace assembly dialect.

use std::{collections::HashMap, str};

use crate::{
    dialects::burghard::{option::OptionNester, parse::Parser},
    syntax::{Cst, Opcode},
    tokens::mnemonics::FoldedStr,
};

// TODO:
// - Move Cst macros to syntax.

/// State for parsing the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Burghard {
    mnemonics: HashMap<FoldedStr<'static>, &'static [Opcode]>,
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
                .map(|&(mnemonic, sigs)| (FoldedStr::ascii_ik(mnemonic.as_bytes()), sigs))
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Burghard dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        OptionNester::new().nest(&mut Parser::new(src, self))
    }

    /// Gets the overloaded opcodes for a mnemonic.
    pub(super) fn get_opcodes(&self, mnemonic: &[u8]) -> Option<&'static [Opcode]> {
        self.mnemonics.get(&FoldedStr::exact(mnemonic)).copied()
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
        Token::from(MnemonicToken {
            mnemonic: $mnemonic.into(),
            opcode: $opcode,
        })
    });
    macro_rules! block_comment(($text:literal) => {
        Token::from(BlockCommentToken {
            text: $text,
            style: BlockCommentStyle::Haskell,
            errors: EnumSet::empty(),
        })
    });
    macro_rules! space(($space:literal) => {
        Spaces::from(Token::from(SpaceToken::from($space)))
    });
    macro_rules! lf(() => {
        Spaces::from(Token::from(LineTermToken::from(LineTermStyle::Lf)))
    });
    macro_rules! eof(() => {
        Spaces::from(Token::from(EofToken))
    });

    #[test]
    fn spliced() {
        let src = b" {-c1-}hello{-splice-}world{-c2-}\t!";
        let cst = Burghard::new().parse(src);
        let expect = root![Cst::Inst(Inst {
            words: Words {
                space_before: Spaces::from(vec![
                    Token::from(SpaceToken::from(b" ")),
                    block_comment!(b"c1"),
                ]),
                words: vec![
                    (
                        Token::from(SplicedToken {
                            tokens: vec![
                                Token::from(WordToken {
                                    word: b"hello".into()
                                }),
                                block_comment!(b"splice"),
                                Token::from(WordToken {
                                    word: b"world".into()
                                }),
                            ],
                            spliced: Box::new(mnemonic!(b"helloworld", Opcode::Invalid)),
                        }),
                        Spaces::from(vec![
                            block_comment!(b"c2"),
                            Token::from(SpaceToken::from(b"\t")),
                        ]),
                    ),
                    (
                        Token::from(StringToken {
                            literal: b"!".into(),
                            unescaped: StringData::Utf8("!".into()),
                            quotes: QuoteStyle::Bare,
                            errors: EnumSet::empty(),
                        }),
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
                    Token::from(QuotedToken {
                        inner: Box::new(mnemonic!(
                            "Debug_PrİntStacK".as_bytes(),
                            Opcode::BurghardPrintStack,
                        )),
                        quotes: QuoteStyle::Double,
                        errors: QuotedError::Unterminated.into(),
                    }),
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
                        Token::from(StringToken {
                            literal: b"1".into(),
                            unescaped: StringData::Utf8("1".into()),
                            quotes: QuoteStyle::Double,
                            errors: EnumSet::empty(),
                        }),
                        space!(b" "),
                    ),
                    (
                        Token::from(QuotedToken {
                            inner: Box::new(Token::from(IntegerToken {
                                literal: b"2".into(),
                                value: Integer::from(2),
                                ..Default::default()
                            })),
                            quotes: QuoteStyle::Double,
                            errors: EnumSet::empty(),
                        }),
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
                        (Token::from(WordToken { word: $option.into() }), lf!()),
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
                        (Token::from(WordToken { word: $option.into() }), lf!()),
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
