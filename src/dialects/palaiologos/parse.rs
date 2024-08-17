//! Parser for the Palaiologos Whitespace assembly dialect.

use crate::{
    dialects::{palaiologos::lex::Lexer, Palaiologos},
    lex::TokenStream,
    syntax::{Cst, Dialect, Inst},
    tokens::{
        spaces::{ArgSepError, Spaces},
        words::Words,
        ErrorToken, Token,
    },
};

// TODO:
// - Parse instruction separators and LF more precisely.
// - Parse label definitions to a separate Inst.

/// A parser for the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Parser<'s, 'd> {
    toks: TokenStream<'s, Lexer<'s, 'd>>,
}

impl<'s, 'd> Parser<'s, 'd> {
    /// Constructs a new parser for Palaiologos-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d Palaiologos) -> Self {
        Parser {
            toks: TokenStream::new(Lexer::new(src, dialect)),
        }
    }

    /// Parses the CST.
    pub fn parse(&mut self) -> Cst<'s> {
        let mut nodes = Vec::new();
        while !self.toks.eof() {
            let mut space_before = Spaces::new();
            loop {
                match self.toks.curr() {
                    Token::Space(_) | Token::LineTerm(_) | Token::InstSep(_) => {
                        space_before.push(self.toks.advance())
                    }
                    Token::Eof(_) => {
                        space_before.push(self.toks.advance());
                        break;
                    }
                    Token::ArgSep(_) => {
                        let Token::ArgSep(mut tok) = self.toks.advance() else {
                            unreachable!();
                        };
                        tok.errors |= ArgSepError::NotBetweenArguments;
                        space_before.push(Token::from(tok));
                    }
                    _ => break,
                }
            }
            if self.toks.eof() {
                nodes.push(Cst::Empty(space_before));
                break;
            }

            let mut words = Words::new(space_before);
            while !matches!(
                self.toks.curr(),
                Token::LineTerm(_)
                    | Token::Eof(_)
                    | Token::InstSep(_)
                    | Token::Error(ErrorToken::InvalidUtf8 { .. })
            ) {
                words.push_token(self.toks.advance());
            }
            words.push_space(self.toks.advance());
            nodes.push(Cst::Inst(Inst {
                words,
                valid_arity: false,
                valid_types: false,
            }));
        }
        Cst::Dialect {
            dialect: Dialect::Palaiologos,
            inner: Box::new(Cst::Block { nodes }),
        }
    }
}
