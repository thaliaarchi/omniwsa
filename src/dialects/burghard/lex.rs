//! Lexer for the Burghard Whitespace assembly dialect.

use std::str;

use crate::{
    scan::Utf8Scanner,
    token_stream::Lex,
    tokens::{
        string::{QuoteStyle, QuotedToken},
        Token, TokenError, TokenKind,
    },
};

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Utf8Scanner<'s>,
    /// The remaining text at the first UTF-8 error and the length of the
    /// invalid sequence.
    invalid_utf8: Option<(&'s [u8], usize)>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
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
