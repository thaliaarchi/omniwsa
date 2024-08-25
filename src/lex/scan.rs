//! Generic token scanning.

use std::{cmp::Ordering, str};

use enumset::EnumSet;

use crate::tokens::{
    comment::{
        BlockCommentError, BlockCommentStyle, BlockCommentToken, LineCommentStyle, LineCommentToken,
    },
    spaces::EofToken,
    ErrorToken, Token,
};

// TODO:
// - Make Pos::line and Pos::col be NonZeroU32 and rename Pos::col -> column.
// - Remove Utf8Scanner.

/// A scanner for generically reading tokens from conventionally UTF-8 text.
#[derive(Clone, Debug)]
pub struct Scanner<'s> {
    /// Source text being scanned. It is not guaranteed to be UTF-8, but is
    /// processed as UTF-8 when valid.
    src: &'s [u8],
    /// Start position of the current token.
    start: Pos,
    /// End position of the current token.
    end: Pos,
}

/// A scanner for generically reading tokens from UTF-8 text.
#[derive(Clone, Debug)]
pub struct Utf8Scanner<'s> {
    /// Source text being scanned.
    src: &'s str,
    /// Start position of the current token.
    start: Pos,
    /// End position of the current token.
    end: Pos,
    /// The remaining text at the first UTF-8 error and the length of the
    /// invalid sequence.
    invalid_utf8: Option<(&'s [u8], usize)>,
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

/// An error from decoding an invalid UTF-8 code point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf8Error;

impl<'s> Utf8Scanner<'s> {
    /// Constructs a new scanner for the UTF-8 source text.
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
        let pos = Pos {
            offset: 0,
            line: 1,
            col: 1,
        };
        Utf8Scanner {
            src,
            start: pos,
            end: pos,
            invalid_utf8,
        }
    }

    /// Returns whether the scanner is at the end of the source.
    #[inline]
    pub fn eof(&self) -> bool {
        debug_assert!(self.end.offset <= self.src.len());
        self.end.offset >= self.src.len()
    }

    /// Returns an EOF token or an invalid UTF-8 token.
    #[inline]
    pub fn eof_token(&mut self) -> Token<'s> {
        debug_assert!(self.eof());
        match self.invalid_utf8.take() {
            Some((rest, error_len)) => {
                return Token::from(ErrorToken::InvalidUtf8 {
                    text: rest.into(),
                    error_len,
                });
            }
            None => Token::from(EofToken),
        }
    }

    /// Returns the next char without consuming it.
    #[inline]
    pub fn peek_char(&mut self) -> char {
        self.src[self.end.offset..].chars().next().unwrap()
    }

    /// Consumes and returns the next char.
    #[inline]
    pub fn next_char(&mut self) -> char {
        let mut chars = self.src[self.end.offset..].chars();
        let ch = chars.next().unwrap();
        self.end.offset = self.src.len() - chars.as_str().len();
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

    /// Consumes a line comment. The cursor must start after the comment prefix.
    pub fn line_comment(&mut self, style: LineCommentStyle) -> LineCommentToken<'s> {
        let text_start = self.offset();
        self.bump_while(|c| c != '\n');
        let src = self.src.as_bytes();
        LineCommentToken {
            text: &src[text_start..self.offset()],
            style,
        }
    }

    /// Consumes a non-nested block comment. The cursor must start after the
    /// opening sequence.
    pub fn block_comment(
        &mut self,
        close: [u8; 2],
        style: BlockCommentStyle,
    ) -> BlockCommentToken<'s> {
        debug_assert!(!style.can_nest());
        let text_start = self.offset();
        let (text_end, errors) = loop {
            let rest = self.rest().as_bytes();
            if rest.len() < 2 {
                if !self.eof() {
                    self.next_char();
                }
                break (self.end.offset, BlockCommentError::Unterminated.into());
            } else if rest[..2] == close {
                self.end.offset += 2;
                self.end.col += 2;
                break (self.end.offset - 2, EnumSet::empty());
            }
            self.next_char();
        };
        BlockCommentToken {
            text: &self.src.as_bytes()[text_start..text_end],
            style,
            errors,
        }
    }

    /// Consumes a nested block comment. The cursor must start after the opening
    /// sequence.
    pub fn nested_block_comment(
        &mut self,
        open: [u8; 2],
        close: [u8; 2],
        style: BlockCommentStyle,
    ) -> BlockCommentToken<'s> {
        debug_assert!(style.can_nest());
        let mut level = 1;
        let text_start = self.offset();
        let (text_end, errors) = loop {
            let rest = self.rest().as_bytes();
            if rest.len() < 2 {
                if !self.eof() {
                    self.next_char();
                }
                break (self.end.offset, BlockCommentError::Unterminated.into());
            } else if rest[..2] == close {
                self.end.offset += 2;
                self.end.col += 2;
                level -= 1;
                if level == 0 {
                    break (self.end.offset - 2, EnumSet::empty());
                }
            } else if rest[..2] == open {
                self.end.offset += 2;
                self.end.col += 2;
                level += 1;
            } else {
                self.next_char();
            }
        };
        BlockCommentToken {
            text: &self.src.as_bytes()[text_start..text_end],
            style,
            errors,
        }
    }

    /// Consumes a Burghard-style nested block comment. Line comments take
    /// precedence over block comment markers. The cursor must start after the
    /// opening sequence.
    pub fn burghard_nested_block_comment(&mut self) -> BlockCommentToken<'s> {
        let mut level = 1;
        let text_start = self.offset();
        let (text_end, errors) = loop {
            let rest = self.rest().as_bytes();
            if rest.len() < 2 {
                if !self.eof() {
                    self.next_char();
                }
                break (self.end.offset, BlockCommentError::Unterminated.into());
            } else if rest[..2] == b"-}"[..] {
                self.end.offset += 2;
                self.end.col += 2;
                level -= 1;
                if level == 0 {
                    break (self.end.offset - 2, EnumSet::empty());
                }
            } else if rest[..2] == b"{-"[..] && rest.get(2) != Some(&b'-') {
                self.end.offset += 2;
                level += 1;
            } else if rest[..2] == b"--"[..] || rest[0] == b';' {
                while !self.eof() && self.next_char() != '\n' {}
            } else {
                self.next_char();
            }
        };
        BlockCommentToken {
            text: &self.src.as_bytes()[text_start..text_end],
            style: BlockCommentStyle::Haskell,
            errors,
        }
    }

    /// Starts a new token.
    #[inline]
    pub fn reset(&mut self) {
        self.start = self.end;
    }

    /// Reverts to an earlier position in the source.
    #[inline]
    pub fn revert(&mut self, end: Pos) {
        debug_assert!(
            self.start.offset <= end.offset
                && self.start.line <= end.line
                && (self.start.line != end.line || self.start.col <= end.col),
        );
        self.end = end;
    }

    /// Returns the full source text.
    #[inline]
    pub fn src(&self) -> &'s str {
        self.src
    }

    /// Returns the text for the previous token as bytes.
    #[inline]
    pub fn text(&self) -> &'s [u8] {
        self.text_str().as_bytes()
    }

    /// Returns the text for the previous token.
    #[inline]
    pub fn text_str(&self) -> &'s str {
        &self.src[self.start_offset()..self.offset()]
    }

    /// Returns the remaining text.
    #[inline]
    pub fn rest(&self) -> &'s str {
        &self.src[self.offset()..]
    }

    /// Returns the starting offset into the source of the current token.
    #[inline]
    pub fn start_offset(&self) -> usize {
        self.start.offset
    }

    /// Returns the current offset into the source.
    #[inline]
    pub fn offset(&self) -> usize {
        self.end.offset
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

impl<'s> Scanner<'s> {
    /// Constructs a new scanner for the source text.
    pub fn new(src: &'s [u8]) -> Self {
        let pos = Pos {
            offset: 0,
            line: 1,
            col: 1,
        };
        Scanner {
            src,
            start: pos,
            end: pos,
        }
    }

    /// Returns whether the scanner is at the end of the source.
    #[inline]
    pub fn eof(&self) -> bool {
        debug_assert!(self.end.offset <= self.src.len());
        self.end.offset >= self.src.len()
    }

    /// Returns the full source text.
    #[inline]
    pub fn src(&self) -> &'s [u8] {
        self.src
    }

    /// Returns the text of the current token.
    #[inline]
    pub fn text(&self) -> &'s [u8] {
        &self.src[self.start.offset..self.end.offset]
    }

    /// Returns the remaining source text to be scanned.
    #[inline]
    pub fn rest(&self) -> &'s [u8] {
        &self.src[self.end.offset..]
    }

    /// Returns the remaining source text, from the start of the current token.
    #[inline]
    pub fn rest_from_start(&self) -> &'s [u8] {
        &self.src[self.start.offset..]
    }

    /// Returns the start position of the current token.
    #[inline]
    pub fn start(&self) -> Pos {
        self.start
    }

    /// Returns the end position of the current token.
    #[inline]
    pub fn end(&self) -> Pos {
        self.end
    }

    /// Starts scanning a new token.
    #[inline]
    pub fn start_next(&mut self) {
        self.start = self.end;
    }

    /// Backtracks to an earlier position in the current token.
    #[inline]
    pub fn backtrack(&mut self, end: Pos) {
        debug_assert!(self.start <= end && end <= self.end);
        self.end = end;
    }

    /// Returns the next byte without consuming it. It is not guaranteed to be
    /// ASCII.
    pub fn peek_byte(&self) -> Option<u8> {
        self.src.get(self.end.offset).copied()
    }

    /// Consumes and returns the next UTF-8 character from the source or, if
    /// invalid, returns the U+FFFD replacement character.
    pub fn next_char_or_replace(&mut self) -> char {
        self.next_char_or_error().unwrap_or('\u{FFFD}')
    }

    /// Consumes and returns the next UTF-8 character from the source or, if
    /// invalid, returns an error.
    pub fn next_char_or_error(&mut self) -> Result<char, Utf8Error> {
        debug_assert!(!self.eof());
        let (ch, size) = bstr::decode_utf8(self.rest());
        self.end.offset += size;
        self.end.col += 1;
        if ch == Some('\n') {
            self.end.line += 1;
            self.end.col = 1;
        }
        match ch {
            Some(ch) => Ok(ch),
            None => Err(Utf8Error),
        }
    }

    /// Consumes and returns the next UTF-8 character from the source or, if
    /// invalid, returns the bytes of the invalid sequence. The invalid sequence
    /// will be the maximal prefix of a valid sequence and will have a length
    /// between 1 and 3, inclusive.
    pub fn next_char_or_bytes(&mut self) -> Result<char, &'s [u8]> {
        debug_assert!(!self.eof());
        let (ch, size) = bstr::decode_utf8(self.rest());
        debug_assert!((1..=3).contains(&size));
        let res = match ch {
            Some(ch) => Ok(ch),
            None => Err(&self.rest()[..size]),
        };
        self.end.offset += size;
        self.end.col += 1;
        if ch == Some('\n') {
            self.end.line += 1;
            self.end.col = 1;
        }
        res
    }

    /// Consumes the next ASCII character. The caller must guarantee that the
    /// next character is ASCII.
    #[inline]
    pub fn bump_ascii(&mut self) {
        let b = self.src[self.end.offset];
        debug_assert!(b.is_ascii());
        self.end.offset += 1;
        if b == b'\n' {
            self.end.line += 1;
            self.end.col = 1;
        } else {
            self.end.col += 1;
        }
    }

    /// Consumes a number of ASCII characters. The caller must guarantee that
    /// at least this many characters remain and that they are ASCII.
    #[inline]
    pub fn bump_ascii_no_lf(&mut self, count: usize) {
        debug_assert!(count <= self.rest().len(), "bumped too far");
        debug_assert!(
            self.rest()[..count]
                .iter()
                .all(|&b| b.is_ascii() && b != b'\n'),
            "bumped past ASCII or LF",
        );
        self.end.offset += count;
        self.end.col += count;
    }

    /// Consumes the next character if it is ASCII and matches the predicate.
    pub fn bump_if_ascii<F: FnMut(u8) -> bool>(&mut self, mut predicate: F) -> bool {
        debug_assert!(!self.eof());
        let b = self.src[self.end.offset];
        if b.is_ascii() && predicate(b) {
            self.bump_ascii();
            true
        } else {
            false
        }
    }

    /// Consumes characters that are ASCII and match the predicate and returns
    /// the consumed text.
    pub fn bump_while_ascii<F: FnMut(u8) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while self
            .peek_byte()
            .is_some_and(|b| b.is_ascii() && predicate(b))
        {
            self.bump_ascii();
        }
        &self.src[start..self.end.offset]
    }

    /// Consumes characters, stopping before the first character that is ASCII
    /// and matches the predicate, and returns the consumed text.
    pub fn bump_until_ascii<F: FnMut(u8) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while let Some(b) = self.peek_byte() {
            if b.is_ascii() {
                if predicate(b) {
                    break;
                }
                self.bump_ascii();
            } else {
                let (_, size) = bstr::decode_utf8(self.rest());
                self.end.offset += size;
                self.end.col += 1;
            }
        }
        &self.src[start..self.end.offset]
    }

    /// Consumes characters until and not including LF and returns the consumed
    /// text.
    #[inline]
    pub fn bump_until_lf(&mut self) -> &'s [u8] {
        let start = self.end.offset;
        while self.peek_byte().is_some_and(|b| b != b'\n') {
            self.end.offset += bstr::decode_utf8(self.rest()).1;
            self.end.col += 1;
        }
        &self.src[start..self.end.offset]
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        #[cfg(not(debug_assertions))]
        {
            Some(self.offset.cmp(&other.offset))
        }
        #[cfg(debug_assertions)]
        match self.offset.cmp(&other.offset) {
            Ordering::Less
                if self.line < other.line || self.line == other.line && self.col < other.line =>
            {
                Some(Ordering::Less)
            }
            Ordering::Equal if self.line == other.line && self.col == other.col => {
                Some(Ordering::Equal)
            }
            Ordering::Greater
                if self.line > other.line || self.line == other.line && self.col > other.col =>
            {
                Some(Ordering::Greater)
            }
            _ => panic!("compared positions from different sources"),
        }
    }
}
