//! Parser for the Burghard Whitespace assembly dialect.

use std::{borrow::Cow, mem, str};

use enumset::EnumSet;

use crate::{
    dialects::{burghard::lex::Lexer, Burghard},
    lex::TokenStream,
    syntax::{ArgSep, ArgType, Cst, HasError, Inst, Opcode, Space},
    tokens::{
        integer::IntegerToken,
        string::{QuoteStyle, StringData, StringToken},
        Token, TokenKind,
    },
};

// TODO:
// - Transform strings to lowercase.
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
        let mnemonic = inst.opcode.unwrap_mut();
        debug_assert_eq!(mnemonic.kind, TokenKind::Word);
        let opcodes = self
            .dialect
            .get_opcodes(&mnemonic.text)
            .unwrap_or(&[Opcode::Invalid]);
        debug_assert!(opcodes.len() > 0);

        // Iterate signatures by the largest arity first.
        for (i, &opcode) in opcodes.iter().enumerate().rev() {
            let types = opcode.arg_types();
            let mut valid = true;
            for ((_, arg), &ty) in inst.args.iter_mut().zip(types.iter()) {
                valid &= self.parse_arg(arg, ty);
            }
            if inst.args.len() >= types.len() || i == 0 {
                inst.valid_arity = inst.args.len() == types.len() || opcode == Opcode::Invalid;
                inst.valid_types = valid;
                mnemonic.kind = TokenKind::Opcode(opcode);
                // Process the remaining arguments.
                let rest = inst.args.len().min(types.len());
                for (_, arg) in &mut inst.args[rest..] {
                    self.parse_arg(arg, ArgType::Variable);
                }
                return;
            }
        }
    }

    /// Parses an argument according to its type and returns whether it is
    /// valid.
    fn parse_arg(&mut self, tok: &mut Token<'_>, ty: ArgType) -> bool {
        let quoted = matches!(tok.kind, TokenKind::Quoted(_));
        let inner = tok.unwrap_mut();
        if inner.kind != TokenKind::Word {
            return true;
        }

        if ty == ArgType::Include || ty == ArgType::Option {
            return true;
        }

        // Parse it as a label.
        if ty == ArgType::Label {
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
        if ty == ArgType::Integer || ty == ArgType::Variable && !quoted {
            let text = str::from_utf8(&inner.text).unwrap();
            let int = IntegerToken::parse_haskell(text, &mut self.digit_buf);
            if ty == ArgType::Integer || !int.has_error() {
                inner.kind = TokenKind::from(int);
                return ty == ArgType::Integer;
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
        ty == ArgType::String
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
