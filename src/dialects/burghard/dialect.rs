//! Parsing for the Burghard Whitespace assembly dialect.

use crate::{
    dialects::burghard::{option::OptionNester, parse::Parser},
    syntax::{Cst, Opcode},
    tokens::mnemonics::{CaseFold, FoldedStr, MnemonicMap},
};

// TODO:
// - Move Cst macros to syntax.

/// State for parsing the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Burghard {
    mnemonics: MnemonicMap,
}

macro_rules! mnemonics[($($mnemonic:literal => [$($opcode:ident),+],)+) => {
    &[$((FoldedStr::new_detect($mnemonic, CaseFold::AsciiIK), &[$(Opcode::$opcode),+])),+]
}];
static MNEMONICS: &[(FoldedStr<'_>, &[Opcode])] = mnemonics![
    b"push" => [Push],
    b"pushs" => [PushString0],
    b"doub" => [Dup],
    b"swap" => [Swap],
    b"pop" => [Drop],
    b"add" => [Add, AddConstRhs],
    b"sub" => [Sub, SubConstRhs],
    b"mul" => [Mul, MulConstRhs],
    b"div" => [Div, DivConstRhs],
    b"mod" => [Mod, ModConstRhs],
    b"store" => [Store, StoreConstLhs],
    b"retrive" => [Retrieve, RetrieveConst],
    b"label" => [Label],
    b"call" => [Call],
    b"jump" => [Jmp],
    b"jumpz" => [Jz],
    b"jumpn" => [Jn],
    b"jumpp" => [BurghardJmpP],
    b"jumpnp" => [BurghardJmpNP],
    b"jumppn" => [BurghardJmpNP],
    b"jumpnz" => [BurghardJmpNZ],
    b"jumppz" => [BurghardJmpPZ],
    b"ret" => [Ret],
    b"exit" => [End],
    b"outC" => [Printc],
    b"outN" => [Printi],
    b"inC" => [Readc],
    b"inN" => [Readi],
    b"debug_printstack" => [BurghardPrintStack],
    b"debug_printheap" => [BurghardPrintHeap],
    b"test" => [BurghardTest],
    b"valueinteger" => [BurghardValueInteger],
    b"valuestring" => [BurghardValueString],
    b"include" => [BurghardInclude],
    b"option" => [DefineOption],
    b"ifoption" => [IfOption],
    b"elseifoption" => [ElseIfOption],
    b"elseoption" => [ElseOption],
    b"endoption" => [EndOption],
];

impl Burghard {
    /// Constructs state for the Burghard dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Burghard {
            mnemonics: MnemonicMap::from(MNEMONICS),
        }
    }

    /// Parses a Whitespace assembly program in the Burghard dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        OptionNester::new().nest(&mut Parser::new(src, self))
    }

    /// Returns the mnemonic map for this dialect.
    pub(super) fn mnemonics(&self) -> &MnemonicMap {
        &self.mnemonics
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
