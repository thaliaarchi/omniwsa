//! Generic token scanning.

use std::{cmp::Ordering, num::NonZeroU32};

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
    /// Whether the text of the current token contains invalid UTF-8.
    has_invalid_utf8: bool,
}

/// Source position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    /// Byte offset, starting at 0.
    offset: usize,
    /// Line number, starting at 1.
    line: NonZeroU32,
    /// Column number, starting at 1.
    column: NonZeroU32,
}

/// An error from decoding an invalid UTF-8 code point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf8Error;

impl<'s> Scanner<'s> {
    /// Constructs a new scanner for the source text.
    pub fn new(src: &'s [u8]) -> Self {
        let pos = Pos {
            offset: 0,
            line: NonZeroU32::new(1).unwrap(),
            column: NonZeroU32::new(1).unwrap(),
        };
        Scanner {
            src,
            start: pos,
            end: pos,
            has_invalid_utf8: false,
        }
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

    /// Returns the text of the current token, starting at the given offset.
    #[inline]
    pub fn text_from_offset(&self, offset: usize) -> &'s [u8] {
        &self.src[offset..self.end.offset]
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

    /// Returns the current offset into the source.
    #[inline]
    pub fn offset(&self) -> usize {
        self.end.offset
    }

    /// Whether the text of the current token contains invalid UTF-8.
    #[inline]
    pub fn has_invalid_utf8(&self) -> bool {
        self.has_invalid_utf8
    }

    /// Starts scanning a new token.
    #[inline]
    pub fn start_next(&mut self) {
        self.start = self.end;
        self.has_invalid_utf8 = false;
    }

    /// Backtracks to an earlier position in the current token.
    #[inline]
    pub fn backtrack(&mut self, end: Pos) {
        debug_assert!(self.start <= end, "backtracked before start");
        debug_assert!(end <= self.end, "backtracked after end");
        self.end = end;
    }

    /// Returns whether the scanner is at the end of the source.
    #[inline]
    pub fn eof(&self) -> bool {
        debug_assert!(self.end.offset <= self.src.len());
        self.end.offset >= self.src.len()
    }

    /// Returns the next byte without consuming it. It is not guaranteed to be
    /// ASCII.
    #[inline]
    pub fn peek_byte(&self) -> Option<u8> {
        self.src.get(self.end.offset).copied()
    }

    /// Returns the nth byte without consuming it. It is not guaranteed to be
    /// ASCII.
    #[inline]
    pub fn peek_byte_at(&self, n: usize) -> Option<u8> {
        self.src.get(self.end.offset + n).copied()
    }

    /// Consumes and returns the next UTF-8 character from the source or, if
    /// invalid, returns the U+FFFD replacement character.
    pub fn next_char(&mut self) -> char {
        self.try_next_char().unwrap_or('\u{FFFD}')
    }

    /// Consumes and returns the next UTF-8 character from the source or, if
    /// invalid, returns an error.
    pub fn try_next_char(&mut self) -> Result<char, Utf8Error> {
        debug_assert!(!self.eof());
        let (ch, size) = bstr::decode_utf8(self.rest());
        self.end.move_char(ch, size);
        match ch {
            Some(ch) => Ok(ch),
            None => {
                self.has_invalid_utf8 = true;
                Err(Utf8Error)
            }
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
            None => {
                self.has_invalid_utf8 = true;
                Err(&self.rest()[..size])
            }
        };
        self.end.move_char(ch, size);
        res
    }

    /// Consumes the next character.
    pub fn bump_char(&mut self) {
        let (ch, size) = bstr::decode_utf8(self.rest());
        self.end.move_char(ch, size);
        self.has_invalid_utf8 |= ch.is_none();
    }

    /// Consumes the next ASCII character. The caller must guarantee that the
    /// next character is ASCII.
    #[inline]
    pub fn bump_ascii(&mut self) {
        self.end.move_ascii(self.src[self.end.offset])
    }

    /// Consumes the next character. The caller must guarantee that the next
    /// character is not LF.
    pub fn bump_char_no_lf(&mut self) {
        let (ch, size) = bstr::decode_utf8(self.rest());
        debug_assert!(ch != Some('\n'));
        self.end.offset += size;
        self.end.column = self.end.column.saturating_add(1);
        self.has_invalid_utf8 |= ch.is_none();
    }

    /// Consumes a number of ASCII characters. The caller must guarantee that
    /// at least this many characters remain and that they are ASCII and not LF.
    #[inline]
    pub fn bump_ascii_no_lf(&mut self, count: usize) {
        debug_assert!(count <= self.rest().len(), "bumped too far");
        debug_assert!(
            self.rest()[..count]
                .iter()
                .all(|&b| b.is_ascii() && b != b'\n'),
            "bumped past ASCII or LF",
        );
        debug_assert!(u32::try_from(count).is_ok(), "count too large for u32");
        self.end.offset += count;
        self.end.column = self.end.column.saturating_add(count as u32);
    }

    /// Consumes the next character if it is valid UTF-8 and matches the
    /// predicate.
    pub fn bump_if_char<F: FnOnce(char) -> bool>(&mut self, predicate: F) -> bool {
        let (ch, size) = bstr::decode_utf8(self.rest());
        if ch.is_some_and(predicate) {
            self.end.move_char(ch, size);
            true
        } else {
            false
        }
    }

    /// Consumes the next character if it is ASCII and matches the predicate.
    pub fn bump_if_ascii<F: FnOnce(u8) -> bool>(&mut self, predicate: F) -> bool {
        if self
            .peek_byte()
            .is_some_and(|b| b.is_ascii() && predicate(b))
        {
            self.bump_ascii();
            true
        } else {
            false
        }
    }

    /// Consumes the next character if it is not valid UTF-8 or does not match
    /// the predicate.
    pub fn bump_unless_char<F: FnOnce(char) -> bool>(&mut self, predicate: F) -> bool {
        let (ch, size) = bstr::decode_utf8(self.rest());
        match ch {
            Some(ch) if predicate(ch) => false,
            None if size == 0 => false,
            _ => {
                self.end.move_char(ch, size);
                true
            }
        }
    }

    /// Consumes the next character if it is not ASCII or does not match the
    /// predicate.
    pub fn bump_unless_ascii<F: FnOnce(u8) -> bool>(&mut self, predicate: F) -> bool {
        let Some(b) = self.peek_byte() else {
            return false;
        };
        if b.is_ascii() {
            if predicate(b) {
                return false;
            }
            self.bump_ascii();
        } else {
            self.bump_char_no_lf();
        }
        true
    }

    /// Consumes characters that are valid UTF-8 and match the predicate and
    /// returns the consumed text.
    pub fn bump_while_char<F: FnMut(char) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while self.bump_if_char(&mut predicate) {}
        &self.src[start..self.end.offset]
    }

    /// Consumes characters that are ASCII and match the predicate and returns
    /// the consumed text.
    pub fn bump_while_ascii<F: FnMut(u8) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while self.bump_if_ascii(&mut predicate) {}
        &self.src[start..self.end.offset]
    }

    /// Consumes characters, stopping before the first character that is valid
    /// UTF-8 and matches the predicate, and returns the consumed text.
    pub fn bump_until_char<F: FnMut(char) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while self.bump_unless_char(&mut predicate) {}
        &self.src[start..self.end.offset]
    }

    /// Consumes characters, stopping before the first character that is ASCII
    /// and matches the predicate, and returns the consumed text.
    pub fn bump_until_ascii<F: FnMut(u8) -> bool>(&mut self, mut predicate: F) -> &'s [u8] {
        let start = self.end.offset;
        while self.bump_unless_ascii(&mut predicate) {}
        &self.src[start..self.end.offset]
    }

    /// Consumes characters until and not including LF and returns the consumed
    /// text.
    #[inline]
    pub fn bump_until_lf(&mut self) -> &'s [u8] {
        let start = self.end.offset;
        while self.peek_byte().is_some_and(|b| b != b'\n') {
            self.bump_char_no_lf();
        }
        &self.src[start..self.end.offset]
    }
}

impl Pos {
    /// Returns the byte offset, starting at 0.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the line number, starting at 1.
    pub fn line(&self) -> usize {
        self.line.get() as usize
    }

    /// Returns the column number, starting at 1.
    pub fn column(&self) -> usize {
        self.column.get() as usize
    }

    /// Moves the position by the width of the char.
    #[inline(always)]
    fn move_char(&mut self, ch: Option<char>, size: usize) {
        debug_assert_ne!(size, 0);
        self.offset += size;
        self.column = self.column.saturating_add(1);
        if ch == Some('\n') {
            self.line = self.line.saturating_add(1);
            self.column = NonZeroU32::new(1).unwrap();
        }
    }

    /// Moves the position by 1. The caller must guarantee that the byte is
    /// ASCII.
    #[inline(always)]
    fn move_ascii(&mut self, ch: u8) {
        debug_assert!(ch.is_ascii());
        self.offset += 1;
        self.column = self.column.saturating_add(1);
        if ch == b'\n' {
            self.line = self.line.saturating_add(1);
            self.column = NonZeroU32::new(1).unwrap();
        }
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
                if self.line < other.line
                    || self.line == other.line && self.column < other.column =>
            {
                Some(Ordering::Less)
            }
            Ordering::Equal if self.line == other.line && self.column == other.column => {
                Some(Ordering::Equal)
            }
            Ordering::Greater
                if self.line > other.line
                    || self.line == other.line && self.column > other.column =>
            {
                Some(Ordering::Greater)
            }
            _ => panic!("compared positions from different sources: {self:?} and {other:?}"),
        }
    }
}
