//! Parsing for the Burghard Whitespace assembly dialect.

use std::{collections::HashMap, str};

use crate::{
    dialects::burghard::{option::OptionNester, parse::Parser},
    mnemonics::Utf8LowerToAscii,
    syntax::Cst,
    tokens::Opcode,
};

// TODO:
// - Add shape of arguments to Inst. This should subsume Args, Type,
//   Inst::valid_arity, and Inst::valid_types.
// - Move Cst macros to syntax.

/// State for parsing the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Burghard {
    mnemonics: HashMap<Utf8LowerToAscii<'static>, (Opcode, Args)>,
}

/// The shape of the arguments for an opcode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Args {
    /// No arguments.
    None,
    /// An integer or variable.
    Integer,
    /// An optional integer or variable.
    IntegerOpt,
    /// A string or variable.
    String,
    /// A variable and an integer or variable.
    VariableAndInteger,
    /// A variable and a string or variable.
    VariableAndString,
    /// A label.
    Label,
    /// A word.
    Word,
}

/// The allowed types of an argument.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Type {
    /// An integer or variable.
    Integer,
    /// A string or variable.
    String,
    /// A variable.
    Variable,
    /// A label.
    Label,
}

macro_rules! mnemonics[($($mnemonic:literal => $opcode:ident $args:ident,)*) => {
    &[$(($mnemonic, Opcode::$opcode, Args::$args),)+]
}];
static MNEMONICS: &[(&'static str, Opcode, Args)] = mnemonics![
    "push" => Push Integer,
    "pushs" => PushString0 String,
    "doub" => Dup None,
    "swap" => Swap None,
    "pop" => Drop None,
    "add" => Add IntegerOpt,
    "sub" => Sub IntegerOpt,
    "mul" => Mul IntegerOpt,
    "div" => Div IntegerOpt,
    "mod" => Mod IntegerOpt,
    "store" => Store IntegerOpt,
    "retrive" => Retrieve IntegerOpt,
    "label" => Label Label,
    "call" => Call Label,
    "jump" => Jmp Label,
    "jumpz" => Jz Label,
    "jumpn" => Jn Label,
    "jumpp" => BurghardJmpP Label,
    "jumpnp" => BurghardJmpNP Label,
    "jumppn" => BurghardJmpNP Label,
    "jumpnz" => BurghardJmpNZ Label,
    "jumppz" => BurghardJmpPZ Label,
    "ret" => Ret None,
    "exit" => End None,
    "outC" => Printc None,
    "outN" => Printi None,
    "inC" => Readc None,
    "inN" => Readi None,
    "debug_printstack" => BurghardPrintStack None,
    "debug_printheap" => BurghardPrintHeap None,
    "test" => BurghardTest Integer,
    "valueinteger" => BurghardValueInteger VariableAndInteger,
    "valuestring" => BurghardValueString VariableAndString,
    "include" => BurghardInclude Word,
    "option" => DefineOption Word,
    "ifoption" => IfOption Word,
    "elseifoption" => ElseIfOption Word,
    "elseoption" => ElseOption None,
    "endoption" => EndOption None,
];

impl Burghard {
    /// Constructs state for the Burghard dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Burghard {
            mnemonics: MNEMONICS
                .iter()
                .map(|&(mnemonic, opcode, args)| {
                    (Utf8LowerToAscii(mnemonic.as_bytes()), (opcode, args))
                })
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Burghard dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Cst<'s> {
        OptionNester::new().nest(&mut Parser::new(src, self))
    }

    /// Gets the opcode and arguments signature for a mnemonic.
    pub(super) fn get_signature(&self, mnemonic: &[u8]) -> Option<(Opcode, Args)> {
        self.mnemonics.get(&Utf8LowerToAscii(mnemonic)).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        dialects::Burghard,
        syntax::{ArgSep, Cst, Dialect, Inst, InstSep, OptionBlock, Space},
        tokens::{
            integer::{Integer, IntegerToken},
            string::{QuoteStyle, QuotedToken, StringData, StringToken},
            Opcode, Token, TokenKind,
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
    macro_rules! space(($space:literal) => {
        Space::from(vec![Token::new($space, TokenKind::Space)])
    });
    macro_rules! lf(() => {
        InstSep::LineTerm {
            space_before: Space::new(),
            line_comment: None,
            line_term: Token::new(b"\n", TokenKind::LineTerm),
        }
    });
    macro_rules! eof(() => {
        InstSep::LineTerm {
            space_before: Space::new(),
            line_comment: None,
            line_term: Token::new(b"", TokenKind::Eof),
        }
    });

    #[test]
    fn spliced() {
        let src = b" {-c1-}hello{-splice-}world{-c2-}\t!";
        let cst = Burghard::new().parse(src);
        let expect = root![Cst::Inst(Inst {
            space_before: Space::from(vec![
                Token::new(b" ", TokenKind::Space),
                block_comment!("c1"),
            ]),
            opcode: Token::new(
                b"hello{-splice-}world",
                TokenKind::Spliced {
                    tokens: vec![
                        Token::new(b"hello", TokenKind::Word),
                        block_comment!("splice"),
                        Token::new(b"world", TokenKind::Word),
                    ],
                    spliced: Box::new(Token::new(b"helloworld", Opcode::Invalid)),
                },
            ),
            args: vec![(
                ArgSep::Space(Space::from(vec![
                    block_comment!("c2"),
                    Token::new(b"\t", TokenKind::Space),
                ])),
                Token::new(b"!", TokenKind::Word),
            )],
            inst_sep: eof!(),
            valid_arity: false,
            valid_types: false,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn mnemonic_utf8_folding() {
        let cst = Burghard::new().parse("\"Debug_PrİntStacK".as_bytes());
        let expect = root![Cst::Inst(Inst {
            space_before: Space::new(),
            opcode: Token::new(
                "\"Debug_PrİntStacK".as_bytes(),
                QuotedToken {
                    inner: Box::new(Token::new(
                        "Debug_PrİntStacK".as_bytes(),
                        Opcode::BurghardPrintStack,
                    )),
                    quotes: QuoteStyle::UnclosedDouble,
                },
            ),
            args: vec![],
            inst_sep: eof!(),
            valid_arity: true,
            valid_types: true,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn bad_args() {
        let cst = Burghard::new().parse(b"valueinteger \"1\" \"2\"");
        let expect = root![Cst::Inst(Inst {
            space_before: Space::new(),
            opcode: Token::new(b"valueinteger", Opcode::BurghardValueInteger),
            args: vec![
                (
                    ArgSep::Space(space!(b" ")),
                    Token::new(
                        b"\"1\"",
                        StringToken {
                            data: StringData::Utf8("1".into()),
                            quotes: QuoteStyle::Double,
                        },
                    ),
                ),
                (
                    ArgSep::Space(space!(b" ")),
                    Token::new(
                        b"\"2\"",
                        QuotedToken {
                            inner: Box::new(Token::new(
                                b"2",
                                IntegerToken {
                                    value: Integer::from(2),
                                    ..Default::default()
                                },
                            )),
                            quotes: QuoteStyle::Double,
                        },
                    ),
                ),
            ],
            inst_sep: eof!(),
            valid_arity: true,
            valid_types: false,
        })];
        assert_eq!(cst, expect);
    }

    #[test]
    fn option_blocks() {
        macro_rules! letter(($letter:literal) => {
            Cst::Inst(Inst {
                space_before: Space::new(),
                opcode: Token::new($letter, Opcode::Invalid),
                args: vec![],
                inst_sep: lf!(),
                valid_arity: true,
                valid_types: true,
            })
        });
        macro_rules! ifoption(($option:literal) => {
            Inst {
                space_before: Space::new(),
                opcode: Token::new(b"ifoption", Opcode::IfOption),
                args: vec![(
                    ArgSep::Space(space!(b" ")),
                    Token::new($option, TokenKind::Word),
                )],
                inst_sep: lf!(),
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! elseifoption(($option:literal) => {
            Inst {
                space_before: Space::new(),
                opcode: Token::new(b"elseifoption", Opcode::ElseIfOption),
                args: vec![(
                    ArgSep::Space(space!(b" ")),
                    Token::new($option, TokenKind::Word),
                )],
                inst_sep: lf!(),
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! elseoption(() => {
            Inst {
                space_before: Space::new(),
                opcode: Token::new(b"elseoption", Opcode::ElseOption),
                args: vec![],
                inst_sep: lf!(),
                valid_arity: true,
                valid_types: true,
            }
        });
        macro_rules! endoption(() => {
            Inst {
                space_before: Space::new(),
                opcode: Token::new(b"endoption", Opcode::EndOption),
                args: vec![],
                inst_sep: lf!(),
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
