//! Generic token scanning.

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

    /// Returns the next byte without consuming it.
    #[inline]
    pub fn peek_byte(&mut self) -> u8 {
        self.src.as_bytes()[self.end.offset]
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

    /// Consumes and returns the next byte.
    #[inline]
    pub fn next_byte(&mut self) -> u8 {
        let ch = self.peek_byte();
        self.end.offset += 1;
        self.end.col += 1;
        if ch == b'\n' {
            self.end.line += 1;
            self.end.col = 1;
        }
        ch
    }

    /// Consumes chars while they match the predicate.
    #[inline]
    pub fn bump_chars_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
        while !self.eof() && predicate(self.peek_char()) {
            self.next_char();
        }
    }

    /// Consumes bytes while they match the predicate.
    #[inline]
    pub fn bump_bytes_while<F: Fn(u8) -> bool>(&mut self, predicate: F) {
        while !self.eof() && predicate(self.peek_byte()) {
            self.next_byte();
        }
    }

    /// Returns the text for the previous token.
    #[inline]
    pub fn text(&self) -> &'s str {
        &self.src[self.start.offset..self.end.offset]
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
