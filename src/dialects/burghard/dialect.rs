//! Parsing for the Burghard Whitespace assembly dialect.

use crate::{
    dialects::{
        burghard::{lex::Lexer, option::OptionNester, parse::Parser},
        define_mnemonics,
        dialect::DialectState,
        Dialect,
    },
    lex::Lex,
    syntax::Cst,
    tokens::{integer::IntegerSyntax, Token},
};

// TODO:
// - Move Cst macros to syntax.

/// Burghard Whitespace assembly dialect.
#[derive(Clone, Copy, Debug)]
pub struct Burghard;

impl Dialect for Burghard {
    define_mnemonics! {
        fold = AsciiIK,
        b"push" => [Push],
        b"pushs" => [PushString0],
        b"doub" => [Dup],
        b"swap" => [Swap],
        b"pop" => [Drop],
        b"add" => [Add], // Overload::BinaryConstRhs
        b"sub" => [Sub], // Overload::BinaryConstRhs
        b"mul" => [Mul], // Overload::BinaryConstRhs
        b"div" => [Div], // Overload::BinaryConstRhs
        b"mod" => [Mod], // Overload::BinaryConstRhs
        b"store" => [Store], // Overload::BinaryConstLhs
        b"retrive" => [Retrieve], // Overload::UnaryConst
        b"label" => [Label],
        b"call" => [Call],
        b"jump" => [Jmp],
        b"jumpz" => [Jz],
        b"jumpn" => [Jn],
        b"jumpp" => [BurghardJmpPos],
        b"jumpnp" => [BurghardJmpNonZero],
        b"jumppn" => [BurghardJmpNonZero],
        b"jumpnz" => [BurghardJmpNonPos],
        b"jumppz" => [BurghardJmpNonNeg],
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
    }

    fn parse<'s>(src: &'s [u8], dialect: &DialectState<Self>) -> Cst<'s> {
        OptionNester::new().nest(&mut Parser::new(src, dialect))
    }

    fn lex<'s>(src: &'s [u8], _dialect: &DialectState<Self>) -> Vec<Token<'s>> {
        let mut lex = Lexer::new(src);
        let mut toks = Vec::new();
        loop {
            let tok = lex.next_token();
            if let Token::Eof(_) = tok {
                break;
            }
            toks.push(tok);
        }
        toks
    }

    fn make_integers() -> IntegerSyntax {
        IntegerSyntax::haskell()
    }
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;

    use crate::{
        dialects::{Burghard, Dialect as _},
        syntax::{ArgLayout, Cst, Dialect, Inst, InstError, Opcode, OptionBlock},
        tokens::{
            comment::{BlockCommentStyle, BlockCommentToken},
            integer::{BaseStyle, Integer, IntegerToken, Sign},
            mnemonics::MnemonicToken,
            spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken, Spaces},
            string::{Encoding, QuoteStyle, StringToken},
            words::Words,
            GroupError, GroupStyle, GroupToken, SplicedToken, Token, WordToken,
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
            style: BlockCommentStyle::Burghard,
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
            opcode: Opcode::Invalid,
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
                                    word: b"hello".into(),
                                    errors: EnumSet::empty(),
                                }),
                                block_comment!(b"splice"),
                                Token::from(WordToken {
                                    word: b"world".into(),
                                    errors: EnumSet::empty(),
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
                            unescaped: b"!".into(),
                            encoding: Encoding::Utf8,
                            quotes: QuoteStyle::Bare,
                            errors: EnumSet::empty(),
                        }),
                        eof!(),
                    ),
                ],
            },
            arg_layout: ArgLayout::Mnemonic,
            overload: None,
            errors: EnumSet::empty(),
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn mnemonic_utf8_folding() {
        let cst = Burghard::new().parse("\"Debug_PrİntStacK".as_bytes());
        let expect = root![Cst::Inst(Inst {
            opcode: Opcode::BurghardPrintStack,
            words: Words {
                space_before: Spaces::new(),
                words: vec![(
                    Token::from(GroupToken {
                        delim: GroupStyle::DoubleQuotes,
                        space_before: Spaces::new(),
                        inner: Box::new(mnemonic!(
                            "Debug_PrİntStacK".as_bytes(),
                            Opcode::BurghardPrintStack,
                        )),
                        space_after: Spaces::new(),
                        errors: GroupError::Unterminated.into(),
                    }),
                    eof!(),
                )],
            },
            arg_layout: ArgLayout::Mnemonic,
            overload: None,
            errors: EnumSet::empty(),
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn bad_args() {
        let cst = Burghard::new().parse(b"valueinteger \"1\" \"2\"");
        let expect = root![Cst::Inst(Inst {
            opcode: Opcode::BurghardValueInteger,
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
                            unescaped: b"1".into(),
                            encoding: Encoding::Utf8,
                            quotes: QuoteStyle::Double,
                            errors: EnumSet::empty(),
                        }),
                        space!(b" "),
                    ),
                    (
                        Token::from(GroupToken {
                            delim: GroupStyle::DoubleQuotes,
                            space_before: Spaces::new(),
                            inner: Box::new(Token::from(IntegerToken {
                                literal: b"2".into(),
                                value: Integer::from(2),
                                sign: Sign::None,
                                base_style: BaseStyle::Decimal,
                                leading_zeros: 0,
                                has_digit_seps: false,
                                errors: EnumSet::empty(),
                            })),
                            space_after: Spaces::new(),
                            errors: EnumSet::empty(),
                        }),
                        eof!(),
                    ),
                ],
            },
            arg_layout: ArgLayout::Mnemonic,
            overload: None,
            errors: InstError::InvalidTypes.into(),
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn option_blocks() {
        macro_rules! letter(($letter:literal) => {
            Cst::Inst(Inst {
                opcode: Opcode::Invalid,
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!($letter, Opcode::Invalid), lf!())],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
            })
        });
        macro_rules! ifoption(($option:literal) => {
            Inst {
                opcode: Opcode::IfOption,
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![
                        (mnemonic!(b"ifoption", Opcode::IfOption), space!(b" ")),
                        (
                            Token::from(WordToken {
                                word: $option.into(),
                                errors: EnumSet::empty(),
                            }),
                            lf!(),
                        ),
                    ],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
            }
        });
        macro_rules! elseifoption(($option:literal) => {
            Inst {
                opcode: Opcode::ElseIfOption,
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![
                        (mnemonic!(b"elseifoption", Opcode::ElseIfOption), space!(b" ")),
                        (
                            Token::from(WordToken {
                                word: $option.into(),
                                errors: EnumSet::empty(),
                            }),
                            lf!(),
                        ),
                    ],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
            }
        });
        macro_rules! elseoption(() => {
            Inst {
                opcode: Opcode::ElseOption,
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!(b"elseoption", Opcode::ElseOption), lf!())],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
            }
        });
        macro_rules! endoption(() => {
            Inst {
                opcode: Opcode::EndOption,
                words: Words {
                    space_before: Spaces::new(),
                    words: vec![(mnemonic!(b"endoption", Opcode::EndOption), lf!())],
                },
                arg_layout: ArgLayout::Mnemonic,
                overload: None,
                errors: EnumSet::empty(),
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
