//! Parsing for the Burghard Whitespace assembly dialect.

use std::{borrow::Cow, collections::HashMap, mem, str};

use enumset::EnumSet;

use crate::{
    mnemonics::Utf8LowerToAscii,
    scan::Utf8Scanner,
    syntax::{ArgSep, Cst, Dialect, HasError, Inst, OptionBlock, Space},
    token_stream::{Lex, TokenStream},
    tokens::{
        integer::IntegerToken,
        string::{QuoteStyle, QuotedToken, StringData, StringToken},
        Opcode, Token, TokenError, TokenKind,
    },
};

// TODO:
// - Add shape of arguments to Inst. This should subsume Args, Type,
//   Inst::valid_arity, and Inst::valid_types.
// - Transform strings to lowercase.
// - Assign stricter tokens to `include` and options.
// - Move Cst macros to syntax.
// - Clean up UTF-8 decoding in parse_arg, since tokens are already validated as
//   UTF-8.
// - Split into lex, integer, parse, etc. files.

/// State for parsing the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Burghard {
    mnemonics: HashMap<Utf8LowerToAscii<'static>, (Opcode, Args)>,
}

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
struct Lexer<'s> {
    scan: Utf8Scanner<'s>,
    /// The remaining text at the first UTF-8 error and the length of the
    /// invalid sequence.
    invalid_utf8: Option<(&'s [u8], usize)>,
}

/// A parser for the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
struct Parser<'s, 'd> {
    dialect: &'d Burghard,
    toks: TokenStream<'s, Lexer<'s>>,
    digit_buf: Vec<u8>,
}

/// A builder, which structures options into blocks.
#[derive(Clone, Debug)]
struct OptionNester<'s> {
    root: Vec<Cst<'s>>,
    option_stack: Vec<OptionBlock<'s>>,
}

/// The shape of the arguments for an opcode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Args {
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
enum Type {
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
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    fn new(src: &'s [u8]) -> Self {
        let (src, invalid_utf8) = match str::from_utf8(src) {
            Ok(src) => (src, None),
            Err(err) => {
                let (valid, rest) = src.split_at(err.valid_up_to());
                let error_len = err.error_len().unwrap_or(rest.len());
                // SAFETY: This sequence has been validated as UTF-8.
                let valid = unsafe { str::from_utf8_unchecked(valid) };
                (valid, Some((rest, error_len)))
            }
        };
        Lexer {
            scan: Utf8Scanner::new(src),
            invalid_utf8,
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s> {
    /// Scans the next token from the source.
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.reset();

        if scan.eof() {
            if let Some((rest, error_len)) = self.invalid_utf8.take() {
                return Token::new(rest, TokenError::Utf8 { error_len });
            }
            return Token::new(b"", TokenKind::Eof);
        }

        match scan.next_char() {
            ';' => scan.line_comment(),
            '-' if scan.bump_if(|c| c == '-') => scan.line_comment(),
            '{' if scan.bump_if(|c| c == '-') => scan.nested_block_comment(*b"{-", *b"-}"),
            ' ' | '\t' => {
                scan.bump_while(|c| c == ' ' || c == '\t');
                scan.wrap(TokenKind::Space)
            }
            '\n' => scan.wrap(TokenKind::LineTerm),
            '"' => {
                let word_start = scan.offset();
                scan.bump_while(|c| c != '"' && c != '\n');
                let word = &scan.src().as_bytes()[word_start..scan.offset()];
                let quotes = if scan.bump_if(|c| c == '"') {
                    QuoteStyle::Double
                } else {
                    QuoteStyle::UnclosedDouble
                };
                scan.wrap(QuotedToken {
                    inner: Box::new(Token::new(word, TokenKind::Word)),
                    quotes,
                })
            }
            _ => {
                while !scan.eof() {
                    let rest = scan.rest().as_bytes();
                    match rest[0] {
                        b';' | b' ' | b'\t' | b'\n' => break,
                        b'-' | b'{' if rest.get(1) == Some(&b'-') => break,
                        _ => {}
                    }
                    scan.next_char();
                }
                scan.wrap(TokenKind::Word)
            }
        }
    }
}

impl<'s, 'd> Parser<'s, 'd> {
    /// Constructs a new parser for Burghard-dialect source text.
    fn new(src: &'s [u8], dialect: &'d Burghard) -> Self {
        Parser {
            dialect,
            toks: TokenStream::new(Lexer::new(src)),
            digit_buf: Vec::new(),
        }
    }
}

impl<'s> Iterator for Parser<'s, '_> {
    type Item = Cst<'s>;

    /// Parses the next line.
    fn next(&mut self) -> Option<Self::Item> {
        if self.toks.eof() {
            return None;
        }

        let space_before = self.toks.space();
        let mut opcode = match self.toks.curr() {
            TokenKind::Word | TokenKind::Quoted(_) => self.toks.advance(),
            _ => return Some(Cst::Empty(self.toks.line_term_sep(space_before))),
        };

        let mut prev_word = &mut opcode;
        let mut args = Vec::new();
        let space_after = loop {
            let space = self.toks.space();
            let arg = match self.toks.curr() {
                TokenKind::Word | TokenKind::Quoted(_) => self.toks.advance(),
                _ => break space,
            };
            if should_splice_tokens(prev_word, &space, &arg) {
                splice_tokens(prev_word, space, arg);
            } else {
                args.push((ArgSep::Space(space), arg));
                prev_word = &mut args.last_mut().unwrap().1;
            }
        };
        let inst_sep = self.toks.line_term_sep(space_after);

        let mut inst = Inst {
            space_before,
            opcode,
            args,
            inst_sep,
            valid_arity: false,
            valid_types: false,
        };
        self.parse_inst(&mut inst);
        Some(Cst::Inst(inst))
    }
}

impl<'s> Parser<'s, '_> {
    /// Parses the opcode and arguments of an instruction.
    fn parse_inst(&mut self, inst: &mut Inst<'s>) {
        let opcode_word = inst.opcode.unwrap_mut();
        debug_assert_eq!(opcode_word.kind, TokenKind::Word);
        let (opcode, args) = self
            .dialect
            .mnemonics
            .get(&Utf8LowerToAscii(&opcode_word.text))
            .copied()
            .unwrap_or((Opcode::Invalid, Args::None));
        opcode_word.kind = TokenKind::Opcode(opcode);

        inst.valid_arity = true;
        inst.valid_types = match (args, &mut inst.args[..]) {
            (Args::None | Args::IntegerOpt, []) => true,
            (Args::Integer | Args::IntegerOpt, [(_, x)]) => self.parse_arg(x, Type::Integer),
            (Args::String, [(_, x)]) => self.parse_arg(x, Type::String),
            (Args::VariableAndInteger, [(_, x), (_, y)]) => {
                self.parse_arg(x, Type::Variable) & self.parse_arg(y, Type::Integer)
            }
            (Args::VariableAndString, [(_, x), (_, y)]) => {
                self.parse_arg(x, Type::Variable) & self.parse_arg(y, Type::String)
            }
            (Args::Label, [(_, x)]) => self.parse_arg(x, Type::Label),
            (Args::Word, [_]) => true,
            _ => {
                inst.valid_arity = false;
                false
            }
        };
    }

    /// Parses an argument according to its type and returns whether it is
    /// valid.
    fn parse_arg(&mut self, tok: &mut Token<'_>, ty: Type) -> bool {
        let quoted = matches!(tok.kind, TokenKind::Quoted(_));
        let inner = tok.unwrap_mut();
        debug_assert_eq!(inner.kind, TokenKind::Word);

        // Parse it as a label.
        if ty == Type::Label {
            inner.kind = TokenKind::Label {
                sigil: b"",
                label: inner.text.clone(),
                errors: EnumSet::empty(),
            };
            return true;
        }

        // Try to parse it as a variable.
        if inner.text.starts_with(b"_") {
            let ident = match &inner.text {
                Cow::Borrowed(text) => text[1..].into(),
                Cow::Owned(text) => text[1..].to_vec().into(),
            };
            inner.kind = TokenKind::Ident { sigil: b"_", ident };
            return true;
        }

        // Try to parse it as an integer.
        if ty == Type::Integer || ty == Type::Variable && !quoted {
            let text = str::from_utf8(&inner.text).unwrap();
            let int = IntegerToken::parse_haskell(text, &mut self.digit_buf);
            if !int.has_error() {
                inner.kind = TokenKind::from(int);
                return ty == Type::Integer;
            }
        }

        // Convert it to a string, including quotes if quoted.
        let tok = match &mut tok.kind {
            TokenKind::Spliced { spliced, .. } => spliced,
            _ => tok,
        };
        tok.kind = match mem::replace(&mut tok.kind, TokenKind::Word) {
            TokenKind::Word => TokenKind::from(StringToken {
                data: StringData::from_utf8(tok.text.clone()).unwrap(),
                quotes: QuoteStyle::Bare,
            }),
            TokenKind::Quoted(q) => {
                debug_assert_eq!(q.inner.kind, TokenKind::Word);
                TokenKind::from(StringToken {
                    data: StringData::from_utf8(q.inner.text).unwrap(),
                    quotes: q.quotes,
                })
            }
            _ => panic!("unhandled token"),
        };
        ty == Type::String
    }
}

/// Returns whether these tokens should be spliced by block comments.
fn should_splice_tokens<'s>(lhs: &Token<'s>, space: &Space<'s>, rhs: &Token<'s>) -> bool {
    space
        .tokens
        .iter()
        .all(|tok| matches!(tok.kind, TokenKind::BlockComment { .. }))
        && matches!(lhs.kind, TokenKind::Word | TokenKind::Spliced { .. })
        && matches!(rhs.kind, TokenKind::Word)
}

/// Splices adjacent tokens, if they are only separated by block comments.
fn splice_tokens<'s>(lhs: &mut Token<'s>, mut space: Space<'s>, rhs: Token<'s>) {
    if lhs.kind == TokenKind::Word {
        lhs.kind = TokenKind::Spliced {
            tokens: vec![lhs.clone()],
            spliced: Box::new(lhs.clone()),
        };
    }
    match &mut lhs.kind {
        TokenKind::Spliced { tokens, spliced } => {
            let text = lhs.text.to_mut();
            for tok in &space.tokens {
                text.extend_from_slice(&tok.text);
            }
            text.extend_from_slice(&rhs.text);
            spliced.text.to_mut().extend_from_slice(&rhs.text);
            tokens.reserve(space.tokens.len() + 1);
            tokens.append(&mut space.tokens);
            tokens.push(rhs);
        }
        _ => panic!("unhandled token"),
    }
}

impl<'s> OptionNester<'s> {
    /// Constructs a builder, which structures options into blocks.
    fn new() -> Self {
        OptionNester {
            root: Vec::new(),
            option_stack: Vec::new(),
        }
    }

    /// Nests instructions into structured option blocks.
    fn nest(&mut self, lines: &mut Parser<'s, '_>) -> Cst<'s> {
        while let Some(line) = lines.next() {
            if let Cst::Inst(inst) = line {
                match inst.opcode() {
                    Opcode::IfOption => {
                        self.option_stack.push(OptionBlock {
                            options: vec![(inst, Vec::new())],
                            end: None,
                        });
                    }
                    Opcode::ElseIfOption | Opcode::ElseOption => {
                        match self.option_stack.last_mut() {
                            Some(block) => {
                                block.options.push((inst, Vec::new()));
                            }
                            None => {
                                self.option_stack.push(OptionBlock {
                                    options: vec![(inst, Vec::new())],
                                    end: None,
                                });
                            }
                        }
                    }
                    Opcode::EndOption => match self.option_stack.pop() {
                        Some(mut block) => {
                            block.end = Some(inst);
                            self.curr_block().push(Cst::OptionBlock(block));
                        }
                        None => {
                            self.root.push(Cst::OptionBlock(OptionBlock {
                                options: Vec::new(),
                                end: Some(inst),
                            }));
                        }
                    },
                    _ => self.curr_block().push(Cst::Inst(inst)),
                }
            } else {
                self.curr_block().push(line);
            }
        }
        let mut parent = &mut self.root;
        for block in self.option_stack.drain(..) {
            parent.push(Cst::OptionBlock(block));
            let Cst::OptionBlock(last) = parent.last_mut().unwrap() else {
                unreachable!();
            };
            parent = &mut last.options.last_mut().unwrap().1;
        }
        let nodes = mem::take(&mut self.root);
        Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block { nodes }),
        }
    }

    /// Returns the current block for instructions to be inserted into.
    fn curr_block(&mut self) -> &mut Vec<Cst<'s>> {
        match self.option_stack.last_mut() {
            Some(last) => &mut last.options.last_mut().unwrap().1,
            None => &mut self.root,
        }
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