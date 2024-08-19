//! Parser for the Burghard Whitespace assembly dialect.

use std::{borrow::Cow, mem, str};

use enumset::EnumSet;

use crate::{
    dialects::{burghard::lex::Lexer, Burghard},
    lex::TokenStream,
    syntax::{ArgType, Cst, HasError, Inst, Opcode},
    tokens::{
        integer::IntegerToken,
        label::{LabelStyle, LabelToken},
        mnemonics::MnemonicToken,
        spaces::Spaces,
        string::{QuoteStyle, StringData, StringToken},
        words::Words,
        ErrorToken, SplicedToken, Token, VariableStyle, VariableToken,
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
        while matches!(self.toks.curr(), Token::Word(_) | Token::Quoted(_)) {
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
        if matches!(self.toks.curr(), Token::LineComment(_)) {
            space_after.push(self.toks.advance());
        }
        debug_assert!(matches!(
            self.toks.curr(),
            Token::LineTerm(_) | Token::Eof(_) | Token::Error(ErrorToken::InvalidUtf8 { .. }),
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
        while matches!(self.toks.curr(), Token::Space(_) | Token::BlockComment(_)) {
            space.push(self.toks.advance());
        }
        space
    }

    /// Parses the mnemonic and arguments of an instruction.
    fn parse_inst(&mut self, inst: &mut Inst<'s>) {
        let ((mnemonic, _), args) = inst.words.words.split_first_mut().unwrap();
        let mnemonic = mnemonic.ungroup_mut();
        let Token::Word(mnemonic_word) = mnemonic else {
            panic!("unhandled token");
        };
        let opcodes = self
            .dialect
            .mnemonics()
            .get_opcodes(&mnemonic_word.word)
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
                *mnemonic = Token::from(MnemonicToken {
                    mnemonic: mem::take(&mut mnemonic_word.word),
                    opcode,
                });
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
        let quoted = matches!(tok, Token::Quoted(_));
        let inner = tok.ungroup_mut();
        let Token::Word(inner_word) = inner else {
            return true;
        };

        if ty == ArgType::Include || ty == ArgType::Option {
            return true;
        }

        // Parse it as a label.
        if ty == ArgType::Label {
            *inner = Token::from(LabelToken {
                label: mem::take(&mut inner_word.word),
                style: LabelStyle::NoSigil,
                errors: EnumSet::empty(),
            });
            return true;
        }

        // Try to parse it as a variable.
        if inner_word.word.starts_with(b"_") {
            let ident = match &inner_word.word {
                Cow::Borrowed(text) => text[1..].into(),
                Cow::Owned(text) => text[1..].to_vec().into(),
            };
            *inner = Token::from(VariableToken {
                ident,
                style: VariableStyle::UnderscoreSigil,
            });
            return true;
        }

        // Try to parse it as an integer.
        if ty == ArgType::Integer || ty == ArgType::Variable && !quoted {
            let text = match inner_word.word.clone() {
                Cow::Borrowed(text) => str::from_utf8(text).unwrap().into(),
                Cow::Owned(text) => String::from_utf8(text).unwrap().into(),
            };
            let int = IntegerToken::parse_haskell(text, &mut self.digit_buf);
            if ty == ArgType::Integer || !int.has_error() {
                *inner = Token::from(int);
                return ty == ArgType::Integer;
            }
        }

        // Convert it to a string, including quotes if quoted.
        let tok = match tok {
            Token::Spliced(s) => &mut s.spliced,
            _ => tok,
        };
        *tok = match mem::take(tok) {
            Token::Word(w) => Token::from(StringToken {
                literal: w.word.clone(),
                unescaped: StringData::from_utf8(w.word.clone()).unwrap(),
                quotes: QuoteStyle::Bare,
                errors: EnumSet::empty(),
            }),
            Token::Quoted(q) => {
                let Token::Word(w) = *q.inner else {
                    panic!("unhandled token");
                };
                Token::from(StringToken {
                    literal: w.word.clone(),
                    unescaped: StringData::from_utf8(w.word).unwrap(),
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
        .all(|tok| matches!(tok, Token::BlockComment(_)))
        && matches!(lhs, Token::Word(_) | Token::Spliced(_))
        && matches!(rhs, Token::Word(_))
}

/// Splices adjacent tokens, if they are only separated by block comments.
fn splice_tokens<'s>(lhs: &mut Token<'s>, space: &mut Spaces<'s>, rhs: Token<'s>) {
    if matches!(lhs, Token::Word(_)) {
        let spliced = lhs.clone();
        *lhs = Token::from(SplicedToken {
            tokens: vec![mem::take(lhs)],
            spliced: Box::new(spliced),
        });
    }
    match lhs {
        Token::Spliced(s) => {
            let Token::Word(spliced) = &mut *s.spliced else {
                panic!("unhandled token");
            };
            let Token::Word(rhs_word) = &rhs else {
                panic!("unhandled token");
            };
            spliced.word.to_mut().extend_from_slice(&rhs_word.word);
            s.tokens.reserve(space.tokens.len() + 1);
            s.tokens.append(&mut space.tokens);
            s.tokens.push(rhs);
        }
        _ => panic!("unhandled token"),
    }
}
