//! Parser for the Burghard Whitespace assembly dialect.

use std::{borrow::Cow, mem, str};

use enumset::EnumSet;

use crate::{
    dialects::{
        burghard::{
            dialect::{Args, Type},
            lex::Lexer,
        },
        Burghard,
    },
    syntax::{ArgSep, Cst, HasError, Inst, Space},
    token_stream::TokenStream,
    tokens::{
        integer::IntegerToken,
        string::{QuoteStyle, StringData, StringToken},
        Opcode, Token, TokenKind,
    },
};

// TODO:
// - Transform strings to lowercase.
// - Assign stricter tokens to `include` and options.
// - Clean up UTF-8 decoding in parse_arg, since tokens are already validated as
//   UTF-8.

/// A parser for the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Parser<'s, 'd> {
    dialect: &'d Burghard,
    toks: TokenStream<'s, Lexer<'s>>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Parser<'s, 'd> {
    /// Constructs a new parser for Burghard-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d Burghard) -> Self {
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
            .get_signature(&opcode_word.text)
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
