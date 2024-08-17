//! Parser for the Burghard Whitespace assembly dialect.

use std::{borrow::Cow, mem, str};

use enumset::EnumSet;

use crate::{
    dialects::{burghard::lex::Lexer, Burghard},
    lex::TokenStream,
    syntax::{ArgType, Cst, HasError, Inst, Opcode},
    tokens::{
        integer::IntegerToken,
        label::LabelToken,
        spaces::Spaces,
        string::{QuoteStyle, StringData, StringToken},
        words::Words,
        ErrorToken, SplicedToken, Token, TokenKind, VariableToken, WordToken,
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

        let mut words = Words::new(self.space());
        while matches!(self.toks.curr(), TokenKind::Word(_) | TokenKind::Quoted(_)) {
            let word = self.toks.advance();
            let space = self.space();
            match words.words.last_mut() {
                Some((prev_word, prev_space))
                    if should_splice_tokens(prev_word, prev_space, &word) =>
                {
                    splice_tokens(prev_word, prev_space, word);
                    *prev_space = space;
                }
                _ => words.push(word, space),
            }
        }

        let space_after = words.trailing_spaces_mut();
        if matches!(self.toks.curr(), TokenKind::LineComment(_)) {
            space_after.push(self.toks.advance());
        }
        debug_assert!(matches!(
            self.toks.curr(),
            TokenKind::LineTerm(_)
                | TokenKind::Eof(_)
                | TokenKind::Error(ErrorToken::InvalidUtf8 { .. }),
        ));
        space_after.push(self.toks.advance());

        if words.is_empty() {
            return Some(Cst::Empty(words.space_before));
        }
        let mut inst = Inst {
            words,
            valid_arity: false,
            valid_types: false,
        };
        self.parse_inst(&mut inst);
        Some(Cst::Inst(inst))
    }
}

impl<'s> Parser<'s, '_> {
    /// Consumes space and block comment tokens.
    fn space(&mut self) -> Spaces<'s> {
        let mut space = Spaces::new();
        while matches!(
            self.toks.curr(),
            TokenKind::Space(_) | TokenKind::BlockComment(_)
        ) {
            space.push(self.toks.advance());
        }
        space
    }

    /// Parses the mnemonic and arguments of an instruction.
    fn parse_inst(&mut self, inst: &mut Inst<'s>) {
        let ((mnemonic, _), args) = inst.words.words.split_first_mut().unwrap();
        let mnemonic = mnemonic.unwrap_mut();
        debug_assert!(matches!(mnemonic.kind, TokenKind::Word(_)));
        let opcodes = self
            .dialect
            .get_opcodes(&mnemonic.text)
            .unwrap_or(&[Opcode::Invalid]);
        debug_assert!(opcodes.len() > 0);

        // Iterate signatures by the largest arity first.
        for (i, &opcode) in opcodes.iter().enumerate().rev() {
            let types = opcode.arg_types();
            let mut valid = true;
            for ((arg, _), &ty) in args.iter_mut().zip(types.iter()) {
                valid &= self.parse_arg(arg, ty);
            }
            if args.len() >= types.len() || i == 0 {
                inst.valid_arity = args.len() == types.len() || opcode == Opcode::Invalid;
                inst.valid_types = valid;
                mnemonic.kind = TokenKind::Opcode(opcode);
                // Process the remaining arguments.
                let rest = args.len().min(types.len());
                for (arg, _) in &mut args[rest..] {
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
        if !matches!(inner.kind, TokenKind::Word(_)) {
            return true;
        }

        if ty == ArgType::Include || ty == ArgType::Option {
            return true;
        }

        // Parse it as a label.
        if ty == ArgType::Label {
            inner.kind = TokenKind::from(LabelToken {
                sigil: b"",
                label: inner.text.clone(),
                errors: EnumSet::empty(),
            });
            return true;
        }

        // Try to parse it as a variable.
        if inner.text.starts_with(b"_") {
            let ident = match &inner.text {
                Cow::Borrowed(text) => text[1..].into(),
                Cow::Owned(text) => text[1..].to_vec().into(),
            };
            inner.kind = TokenKind::from(VariableToken { sigil: b"_", ident });
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
            TokenKind::Spliced(s) => &mut s.spliced,
            _ => tok,
        };
        tok.kind = match mem::replace(&mut tok.kind, WordToken.into()) {
            TokenKind::Word(_) => TokenKind::from(StringToken {
                literal: tok.text.clone(),
                unescaped: StringData::from_utf8(tok.text.clone()).unwrap(),
                quotes: QuoteStyle::Bare,
                errors: EnumSet::empty(),
            }),
            TokenKind::Quoted(q) => {
                debug_assert!(matches!(q.inner.kind, TokenKind::Word(_)));
                TokenKind::from(StringToken {
                    literal: q.inner.text.clone(),
                    unescaped: StringData::from_utf8(q.inner.text).unwrap(),
                    quotes: q.quotes,
                    errors: EnumSet::empty(),
                })
            }
            _ => panic!("unhandled token"),
        };
        ty == ArgType::String
    }
}

/// Returns whether these tokens should be spliced by block comments.
fn should_splice_tokens<'s>(lhs: &Token<'s>, space: &Spaces<'s>, rhs: &Token<'s>) -> bool {
    space
        .tokens
        .iter()
        .all(|tok| matches!(tok.kind, TokenKind::BlockComment(_)))
        && matches!(lhs.kind, TokenKind::Word(_) | TokenKind::Spliced(_))
        && matches!(rhs.kind, TokenKind::Word(_))
}

/// Splices adjacent tokens, if they are only separated by block comments.
fn splice_tokens<'s>(lhs: &mut Token<'s>, space: &mut Spaces<'s>, rhs: Token<'s>) {
    if matches!(lhs.kind, TokenKind::Word(_)) {
        lhs.kind = TokenKind::from(SplicedToken {
            tokens: vec![lhs.clone()],
            spliced: Box::new(lhs.clone()),
        });
    }
    match &mut lhs.kind {
        TokenKind::Spliced(s) => {
            let text = lhs.text.to_mut();
            for tok in &space.tokens {
                text.extend_from_slice(&tok.text);
            }
            text.extend_from_slice(&rhs.text);
            s.spliced.text.to_mut().extend_from_slice(&rhs.text);
            s.tokens.reserve(space.tokens.len() + 1);
            s.tokens.append(&mut space.tokens);
            s.tokens.push(rhs);
        }
        _ => panic!("unhandled token"),
    }
}
