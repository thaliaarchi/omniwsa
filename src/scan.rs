//! Generic token scanning.

use crate::token::{Token, TokenKind};

/// A scanner for generically reading tokens from UTF-8 text.
#[derive(Clone, Debug)]
pub struct Utf8Scanner<'s> {
    /// Source text being scanned.
    src: &'s str,
    /// Start position of the current token.
    start: Pos,
    /// End position of the current token.
    end: Pos,
}

/// Source position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    /// Byte offset, starting at 0.
    pub offset: usize,
    /// Line number, starting at 1.
    pub line: usize,
    /// Column number, starting at 1.
    pub col: usize,
}

impl<'s> Utf8Scanner<'s> {
    /// Constructs a new scanner for the source text.
    #[inline]
    pub fn new(src: &'s str) -> Self {
        let pos = Pos {
            offset: 0,
            line: 1,
            col: 1,
        };
        Utf8Scanner {
            src,
            start: pos,
            end: pos,
        }
    }

    /// Returns whether the lexer is at the end of the source.
    #[inline]
    pub fn eof(&self) -> bool {
        debug_assert!(self.end.offset <= self.src.len());
        self.end.offset >= self.src.len()
    }

    /// Returns the next char without consuming it.
    #[inline]
    pub fn peek_char(&mut self) -> char {
        self.src[self.end.offset..].chars().next().unwrap()
    }

    /// Consumes and returns the next char.
    #[inline]
    pub fn next_char(&mut self) -> char {
        let ch = self.peek_char();
        self.end.offset += 1;
        self.end.col += 1;
        if ch == '\n' {
            self.end.line += 1;
            self.end.col = 1;
        }
        ch
    }

    /// Consumes the next char if it matches the predicate.
    #[inline]
    pub fn bump_if<F: Fn(char) -> bool>(&mut self, predicate: F) -> bool {
        if !self.eof() && predicate(self.peek_char()) {
            self.next_char();
            true
        } else {
            false
        }
    }

    /// Consumes chars while they match the predicate.
    #[inline]
    pub fn bump_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
        while !self.eof() && predicate(self.peek_char()) {
            self.next_char();
        }
    }

    /// Consumes until the end of the line and returns a line comment token. The
    /// cursor must start after the comment prefix.
    pub fn line_comment(&mut self) -> Token<'s> {
        let text_start = self.end.offset;
        self.bump_while(|c| c != '\n');
        let src = self.src.as_bytes();
        self.wrap(TokenKind::LineComment {
            prefix: &src[self.start.offset..text_start],
            text: &src[text_start..self.end.offset],
        })
    }

    /// Consumes a non-nested block comment. The cursor must start after the
    /// opening sequence.
    pub fn block_comment(&mut self, close: [u8; 2]) -> Token<'s> {
        let text_start = self.end.offset;
        let terminated = loop {
            let Some(&chunk) = self.rest().as_bytes().first_chunk::<2>() else {
                self.end.offset = self.src.len();
                break false;
            };
            if chunk == close {
                break true;
            }
            self.next_char();
        };
        let text_end = self.end.offset;
        let src = self.src.as_bytes();
        self.wrap(TokenKind::BlockComment {
            open: &src[self.start.offset..text_start],
            text: &src[text_start..text_end],
            close: &src[text_end..self.end.offset],
            nested: false,
            terminated,
        })
    }

    /// Wraps a `TokenKind` with the text of the current token.
    #[inline]
    pub fn wrap(&self, kind: TokenKind<'s>) -> Token<'s> {
        Token {
            text: self.text().as_bytes(),
            kind,
        }
    }

    /// Starts a new token.
    #[inline]
    pub fn reset(&mut self) {
        self.start = self.end;
    }

    /// Returns the full source text.
    #[inline]
    pub fn src(&self) -> &'s str {
        self.src
    }

    /// Returns the text for the previous token.
    #[inline]
    pub fn text(&self) -> &'s str {
        &self.src[self.start.offset..self.end.offset]
    }

    /// Returns the remaining text.
    #[inline]
    pub fn rest(&self) -> &'s str {
        &self.src[self.end.offset..]
    }

    /// Returns the start position of the previous token.
    #[inline]
    pub fn start(&self) -> Pos {
        self.start
    }

    /// Returns the end position of the previous token.
    #[inline]
    pub fn end(&self) -> Pos {
        self.end
    }
}
