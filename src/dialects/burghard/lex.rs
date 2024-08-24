//! Lexer for the Burghard Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    lex::{Lex, Utf8Scanner},
    tokens::{
        comment::{BlockCommentError, BlockCommentStyle, BlockCommentToken, LineCommentStyle},
        spaces::{LineTermStyle, LineTermToken, SpaceToken},
        string::{QuoteStyle, QuotedError, QuotedToken},
        Token, WordToken,
    },
};

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Utf8Scanner<'s>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        Lexer {
            scan: Utf8Scanner::new(src),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.reset();

        if scan.eof() {
            return scan.eof_token();
        }

        let rest = scan.rest().as_bytes();
        scan.next_char();
        match rest {
            [b';', ..] => scan.line_comment(LineCommentStyle::Semi).into(),
            [b'-', b'-', ..] => {
                scan.next_char();
                scan.line_comment(LineCommentStyle::DashDash).into()
            }
            [b'{', b'-', rest @ ..] if !rest.starts_with(b"-") => {
                scan.next_char();
                scan.burghard_nested_block_comment().into()
            }
            [b'-', b'}', ..] => {
                scan.next_char();
                Token::from(BlockCommentToken {
                    text: b""[..].into(),
                    style: BlockCommentStyle::Haskell,
                    errors: BlockCommentError::Unopened.into(),
                })
            }
            [b' ' | b'\t', ..] => {
                scan.bump_while(|c| c == ' ' || c == '\t');
                Token::from(SpaceToken::from(scan.text()))
            }
            [b'\n', ..] => Token::from(LineTermToken::from(LineTermStyle::Lf)),
            [b'"', ..] => {
                let word_start = scan.offset();
                scan.bump_while(|c| c != '"' && c != '\n');
                let word = &scan.src().as_bytes()[word_start..scan.offset()];
                let errors = if scan.bump_if(|c| c == '"') {
                    EnumSet::empty()
                } else {
                    QuotedError::Unterminated.into()
                };
                Token::from(QuotedToken {
                    inner: Box::new(Token::from(WordToken { word: word.into() })),
                    quotes: QuoteStyle::Double,
                    errors,
                })
            }
            _ => {
                while !scan.eof() {
                    match scan.rest().as_bytes() {
                        [b'"' | b';' | b' ' | b'\t' | b'\n', ..] | [b'-', b'-' | b'}', ..] => break,
                        // Line comments take precedence over block comments.
                        [b'{', b'-', rest @ ..] if !rest.starts_with(b"-") => break,
                        _ => {}
                    }
                    scan.next_char();
                }
                Token::from(WordToken {
                    word: scan.text().into(),
                })
            }
        }
    }
}
