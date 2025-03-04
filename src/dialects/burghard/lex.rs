//! Lexer for the Burghard Whitespace assembly dialect.

use enumset::EnumSet;

use crate::{
    lex::{Lex, Scanner},
    tokens::{
        comment::{
            BlockCommentError, BlockCommentStyle, BlockCommentToken, LineCommentError,
            LineCommentStyle, LineCommentToken,
        },
        spaces::{EofToken, LineTermStyle, LineTermToken, SpaceToken, Spaces},
        GroupError, GroupStyle, GroupToken, Token, WordError, WordToken,
    },
};

/// A lexer for tokens in the Burghard Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s> {
    scan: Scanner<'s>,
}

impl<'s> Lexer<'s> {
    /// Constructs a new lexer for Burghard-dialect source text.
    pub fn new(src: &'s [u8]) -> Self {
        Lexer {
            scan: Scanner::new(src),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.start_next();

        if scan.eof() {
            return EofToken.into();
        }

        let rest = scan.rest();
        scan.next_char();
        match rest {
            [b';', ..] => line_comment(scan, LineCommentStyle::Semi).into(),
            [b'-', b'-', ..] => {
                scan.bump_ascii();
                line_comment(scan, LineCommentStyle::DashDash).into()
            }
            [b'{', b'-', rest @ ..] if !rest.starts_with(b"-") => {
                scan.bump_ascii();
                block_comment(scan).into()
            }
            [b'-', b'}', ..] => {
                scan.bump_ascii();
                Token::from(BlockCommentToken {
                    text: b""[..].into(),
                    style: BlockCommentStyle::Burghard,
                    errors: BlockCommentError::Unopened.into(),
                })
            }
            [b' ' | b'\t', ..] => {
                scan.bump_while_ascii(|ch| ch == b' ' || ch == b'\t');
                Token::from(SpaceToken::from(scan.text()))
            }
            [b'\n', ..] => Token::from(LineTermToken::from(LineTermStyle::Lf)),
            [b'"', ..] => {
                scan.bump_until_ascii(|ch| ch == b'"' || ch == b'\n');
                let word = &scan.text()[1..];
                let quoted_errors = if !scan.bump_if_ascii(|ch| ch == b'"') {
                    GroupError::Unterminated.into()
                } else {
                    EnumSet::empty()
                };
                let word_errors = if scan.has_invalid_utf8() {
                    WordError::InvalidUtf8.into()
                } else {
                    EnumSet::empty()
                };
                Token::from(GroupToken {
                    delim: GroupStyle::DoubleQuotes,
                    space_before: Spaces::new(),
                    inner: Box::new(Token::from(WordToken {
                        word: word.into(),
                        errors: word_errors,
                    })),
                    space_after: Spaces::new(),
                    errors: quoted_errors,
                })
            }
            _ => {
                while !scan.eof() {
                    match scan.rest() {
                        [b'"' | b';' | b' ' | b'\t' | b'\n', ..] | [b'-', b'-' | b'}', ..] => break,
                        // Line comments take precedence over block comments.
                        [b'{', b'-', rest @ ..] if !rest.starts_with(b"-") => break,
                        _ => {}
                    }
                    scan.next_char();
                }
                Token::from(WordToken {
                    word: scan.text().into(),
                    errors: if scan.has_invalid_utf8() {
                        WordError::InvalidUtf8.into()
                    } else {
                        EnumSet::empty()
                    },
                })
            }
        }
    }
}

/// Consumes a line comment. The cursor must start just after the comment
/// prefix.
fn line_comment<'s>(scan: &mut Scanner<'s>, style: LineCommentStyle) -> LineCommentToken<'s> {
    let text = scan.bump_until_lf();
    let mut errors = EnumSet::new();
    if scan.has_invalid_utf8() {
        errors |= LineCommentError::InvalidUtf8;
    };
    LineCommentToken {
        text,
        style,
        errors,
    }
}

/// Consumes a nested block comment. Line comments take precedence over block
/// comment markers. The cursor must start just after `{-`.
fn block_comment<'s>(scan: &mut Scanner<'s>) -> BlockCommentToken<'s> {
    let mut errors = EnumSet::empty();
    let mut level = 1;
    let text = loop {
        match scan.rest() {
            [b'-', b'}', ..] => {
                let text = scan.text();
                scan.bump_ascii_no_lf(2);
                level -= 1;
                if level == 0 {
                    break &text[2..];
                }
            }
            [b'{', b'-', rest @ ..] if !rest.starts_with(b"-") => {
                scan.bump_ascii_no_lf(2);
                level += 1;
            }
            [b'-', b'-', ..] | [b';', ..] => {
                scan.bump_until_lf();
            }
            [] => {
                errors |= BlockCommentError::Unterminated;
                break &scan.text()[2..];
            }
            _ => {
                scan.next_char();
            }
        }
    };
    if scan.has_invalid_utf8() {
        errors |= BlockCommentError::InvalidUtf8;
    }
    BlockCommentToken {
        text,
        style: BlockCommentStyle::Burghard,
        errors,
    }
}
