//! Parsing for the Burghard Whitespace assembly dialect.

use std::{mem, str};

use crate::{
    scan::Utf8Scanner,
    syntax::{ArgSep, Cst, Dialect, Inst, InstSep, Space},
    token::{Token, TokenError, TokenKind},
};

// TODO:
// - Lex tokens in contexts: e.g., mnemonic, number, string, and ident.
// - Resolve mnemonics.
// - Structure option blocks.

impl<'s> Cst<'s> {
    /// Parses a program in the Burghard Whitespace assembly dialect.
    pub fn parse_burghard(src: &'s [u8]) -> Self {
        Parser::new(src).parse()
    }
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
struct Parser<'s> {
    lex: Lexer<'s>,
    tok: Token<'s>,
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

    /// Scans the next token from the source.
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.reset();

        if scan.eof() {
            if let Some((rest, error_len)) = self.invalid_utf8.take() {
                return Token::new(
                    rest,
                    TokenKind::Error(TokenError::InvalidUtf8 { error_len }),
                );
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
                let terminated = scan.bump_if(|c| c == '"');
                scan.wrap(TokenKind::Quoted {
                    inner: Box::new(Token::new(word, TokenKind::Word)),
                    terminated,
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

impl<'s> Parser<'s> {
    /// Constructs a new parser for Burghard-dialect source text.
    fn new(src: &'s [u8]) -> Self {
        let mut lex = Lexer::new(src);
        let tok = lex.next_token();
        Parser { lex, tok }
    }

    /// Parses the entire source.
    fn parse(&mut self) -> Cst<'s> {
        let mut nodes = Vec::new();
        while let Some(inst) = self.next_inst() {
            nodes.push(inst);
        }
        Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block { nodes }),
        }
    }

    /// Parses the next instruction.
    fn next_inst(&mut self) -> Option<Cst<'s>> {
        if self.eof() {
            return None;
        }

        let space_before = self.space();
        let mut mnemonic = match self.curr() {
            TokenKind::Word | TokenKind::Quoted { .. } => self.advance(),
            _ => return Some(Cst::Empty(self.line_term_sep(space_before))),
        };

        let mut prev_word = &mut mnemonic;
        let mut args = Vec::new();
        let space_after = loop {
            let space = self.space();
            let arg = match self.curr() {
                TokenKind::Word | TokenKind::Quoted { .. } => self.advance(),
                _ => break space,
            };
            if should_splice_tokens(prev_word, &space, &arg) {
                splice_tokens(prev_word, space, arg);
            } else {
                args.push((ArgSep::Space(space), arg));
                prev_word = &mut args.last_mut().unwrap().1;
            }
        };

        Some(Cst::Inst(Inst {
            space_before,
            mnemonic,
            args,
            inst_sep: self.line_term_sep(space_after),
        }))
    }

    /// Returns the kind of the current token.
    fn curr(&self) -> &TokenKind<'s> {
        &self.tok.kind
    }

    /// Returns the current token and advances to the next token.
    fn advance(&mut self) -> Token<'s> {
        mem::replace(&mut self.tok, self.lex.next_token())
    }

    /// Returns whether the parser is at EOF.
    fn eof(&self) -> bool {
        matches!(self.curr(), TokenKind::Eof)
    }

    /// Consumes space and block comment tokens.
    fn space(&mut self) -> Space<'s> {
        let mut space = Space::new();
        while matches!(
            self.curr(),
            TokenKind::Space | TokenKind::BlockComment { .. }
        ) {
            space.push(self.advance());
        }
        space
    }

    /// Consumes a line comment token.
    fn line_comment(&mut self) -> Option<Token<'s>> {
        match self.curr() {
            TokenKind::LineComment { .. } => Some(self.advance()),
            _ => None,
        }
    }

    /// Consumes a line terminator, EOF, or invalid UTF-8 error token.
    fn line_term(&mut self) -> Option<Token<'s>> {
        match self.curr() {
            TokenKind::LineTerm
            | TokenKind::Eof
            | TokenKind::Error(TokenError::InvalidUtf8 { .. }) => Some(self.advance()),
            _ => None,
        }
    }

    /// Consumes an optional line comment, followed by a line terminator (or EOF
    /// or invalid UTF-8 error). Panics if not at such a token.
    fn line_term_sep(&mut self, space_before: Space<'s>) -> InstSep<'s> {
        InstSep::LineTerm {
            space_before,
            line_comment: self.line_comment(),
            line_term: self.line_term().expect("line terminator"),
        }
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

#[cfg(test)]
mod tests {
    use crate::{
        syntax::{ArgSep, Cst, Dialect, Inst, InstSep, Space},
        token::{Token, TokenKind},
    };

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

    #[test]
    fn spliced() {
        let src = b" {-c1-}hello{-splice-}world{-c2-}\t!";
        let cst = Cst::parse_burghard(src);
        let expect = Cst::Dialect {
            dialect: Dialect::Burghard,
            inner: Box::new(Cst::Block {
                nodes: vec![Cst::Inst(Inst {
                    space_before: Space::from(vec![
                        Token::new(b" ", TokenKind::Space),
                        block_comment!("c1"),
                    ]),
                    mnemonic: Token::new(
                        b"hello{-splice-}world",
                        TokenKind::Spliced {
                            tokens: vec![
                                Token::new(b"hello", TokenKind::Word),
                                block_comment!("splice"),
                                Token::new(b"world", TokenKind::Word),
                            ],
                            spliced: Box::new(Token::new(b"helloworld", TokenKind::Word)),
                        },
                    ),
                    args: vec![(
                        ArgSep::Space(Space::from(vec![
                            block_comment!("c2"),
                            Token::new(b"\t", TokenKind::Space),
                        ])),
                        Token::new(b"!", TokenKind::Word),
                    )],
                    inst_sep: InstSep::LineTerm {
                        space_before: Space::new(),
                        line_comment: None,
                        line_term: Token::new(b"", TokenKind::Eof),
                    },
                })],
            }),
        };
        assert_eq!(cst, expect);
    }
}
