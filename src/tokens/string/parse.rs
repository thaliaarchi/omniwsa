use bstr::ByteSlice;
use derive_more::derive::Debug as DebugCustom;
use enumset::EnumSet;

use crate::{
    lex::Scanner,
    tokens::string::{
        CharData, CharError, CharToken, Encoding, QuoteStyle, StringError, StringToken,
    },
};

/// A string literal which has been scanned, but not yet parsed.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct StringScan<'s> {
    #[debug("{:?}", literal.as_bstr())]
    literal: &'s [u8],
    backslashes: usize,
    errors: EnumSet<StringError>,
}

/// A character literal which has been scanned, but not yet parsed.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct CharScan<'s> {
    #[debug("{:?}", literal.as_bstr())]
    literal: &'s [u8],
    errors: EnumSet<CharError>,
}

impl<'s> Scanner<'s> {
    /// Scans a single-line string literal. The scanner must be just after the
    /// open `"`.
    pub fn string_lit_oneline(&mut self) -> StringScan<'s> {
        let mut backslashes = 0;
        let mut errors = EnumSet::empty();
        let literal = loop {
            self.bump_until_ascii(|ch| ch == b'"' || ch == b'\\' || ch == b'\n');
            let b = self.peek_byte();
            if b == Some(b'"') {
                let literal = self.text();
                self.bump_ascii();
                break literal;
            }
            if b == Some(b'\\') {
                backslashes += 1;
                if self.bump_unless_ascii(|b| b == b'\n') {
                    continue;
                }
            }
            errors |= StringError::Unterminated;
            break self.text();
        };
        if self.has_invalid_utf8() {
            errors |= StringError::InvalidUtf8;
        }
        StringScan {
            literal,
            backslashes,
            errors,
        }
    }

    /// Scans a single-line character literal. The scanner must be just after
    /// the open `'`.
    pub fn char_lit_oneline(&mut self) -> CharScan<'s> {
        let start = self.start();
        let mut errors = EnumSet::empty();
        let literal = loop {
            self.bump_until_ascii(|ch| ch == b'\'' || ch == b'\\' || ch == b'\n');
            match self.peek_byte() {
                Some(b'\'') => {
                    let literal = self.text();
                    self.bump_ascii();
                    break literal;
                }
                Some(b'\\') if self.bump_unless_ascii(|b| b == b'\n') => {
                    continue;
                }
                _ => {
                    errors |= CharError::Unterminated;
                    self.backtrack(start);
                    self.bump_if_ascii(|ch| ch == b'\\');
                    self.bump_unless_ascii(|ch| ch == b'\n');
                    break self.text();
                }
            }
        };
        if self.has_invalid_utf8() {
            errors |= CharError::InvalidUtf8;
        }
        CharScan { literal, errors }
    }
}

impl<'s> StringScan<'s> {
    /// Unescapes the string literal, only handling single-byte escape
    /// sequences.
    pub fn unescape_simple<F: Fn(u8) -> Option<u8>>(
        &self,
        unescape: F,
        encoding: Encoding,
    ) -> StringToken<'s> {
        let mut errors = self.errors;
        if encoding == Encoding::Bytes {
            errors.remove(StringError::InvalidUtf8);
        }
        let unescaped = if self.backslashes == 0 {
            self.literal.into()
        } else {
            let mut unescaped = Vec::with_capacity(self.literal.len() - self.backslashes);
            let mut s = self.literal;
            while let Some(i) = s.find_byte(b'\\') {
                unescaped.extend_from_slice(&s[..i]);
                if let Some(&b) = s.get(i + 1) {
                    unescaped.push(unescape(b).unwrap_or_else(|| {
                        errors |= StringError::InvalidEscape;
                        b
                    }));
                } else {
                    break;
                }
                s = &s[i + 2..];
            }
            unescaped.extend_from_slice(s);
            unescaped.into()
        };
        StringToken {
            literal: self.literal.into(),
            unescaped,
            encoding,
            quotes: QuoteStyle::Double,
            errors,
        }
    }
}

impl<'s> CharScan<'s> {
    /// Unescapes the character literal with UTF-8 encoding, only handling
    /// single-byte ASCII escape sequences.
    pub fn unescape_simple<F: Fn(u8) -> Option<u8>>(
        &self,
        unescape: F,
        encoding: Encoding,
    ) -> CharToken<'s> {
        let mut errors = self.errors;
        if encoding == Encoding::Bytes {
            errors.remove(CharError::InvalidUtf8);
        }
        if self.literal.is_empty() {
            errors |= CharError::Empty;
        }
        let (bs, escaped) = match self.literal {
            [b'\\', bs @ ..] => {
                errors |= CharError::InvalidEscape;
                (bs, true)
            }
            bs => (bs, false),
        };
        let (ch, size) = bstr::decode_utf8(bs);
        if size != bs.len() {
            errors |= CharError::MoreThanOneChar;
        }
        let data = match ch {
            Some(mut ch) => {
                // TODO: Use if-let chain once stabilized.
                if escaped && ch.is_ascii() {
                    if let Some(unescaped) = unescape(ch as u8) {
                        debug_assert!(unescaped.is_ascii());
                        ch = unescaped as char;
                        errors -= CharError::InvalidEscape;
                    }
                }
                match encoding {
                    Encoding::Utf8 => CharData::Unicode(ch),
                    Encoding::Bytes if ch.is_ascii() => CharData::Byte(ch as u8),
                    Encoding::Bytes => {
                        errors |= CharError::UnexpectedUnicode;
                        CharData::Unicode(ch)
                    }
                }
            }
            None => match encoding {
                Encoding::Utf8 => CharData::Unicode(if bs.is_empty() { '\0' } else { '\u{fffd}' }),
                Encoding::Bytes => {
                    if size > 1 {
                        errors |= CharError::MoreThanOneChar;
                    }
                    CharData::Byte(if bs.is_empty() { 0 } else { bs[0] })
                }
            },
        };
        CharToken {
            literal: self.literal.into(),
            unescaped: data,
            quotes: QuoteStyle::Single,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use enumset::enum_set;

    use super::*;

    #[test]
    fn unescape_char() {
        #[track_caller]
        fn test(
            escaped: &[u8],
            encoding: Encoding,
            expect_unescaped: CharData,
            expect_errors: EnumSet<CharError>,
        ) {
            let mut errors = EnumSet::empty();
            if str::from_utf8(escaped).is_err() {
                errors |= CharError::InvalidUtf8;
            }
            let scanned = CharScan {
                literal: escaped,
                errors,
            };
            let expect = CharToken {
                literal: escaped.into(),
                unescaped: expect_unescaped,
                quotes: QuoteStyle::Single,
                errors: expect_errors,
            };
            let tok = scanned.unescape_simple(unescape_byte, encoding);
            assert_eq!(tok, expect);
        }
        macro_rules! test(($escaped:expr, $encoding:expr => $unescaped:expr $(, $($errors:tt)+)?) => {
            test($escaped, $encoding, $unescaped, enum_set!($($($errors)+)?))
        });

        // b"\xff" is a byte that cannot appear anywhere in UTF-8.
        // b"\xf0\x9f\x9a" is the encoding for 🚇 (U+1F687) without its final
        // byte b'\x87'. b"\xed\xa0\x80" is an unpaired surrogate half (U+D800).

        test!(b"", Encoding::Bytes => CharData::Byte(0), CharError::Empty);
        test!(b"", Encoding::Utf8 => CharData::Unicode('\0'), CharError::Empty);
        test!(b"a", Encoding::Bytes => CharData::Byte(b'a'));
        test!(b"a", Encoding::Utf8 => CharData::Unicode('a'));
        test!(b"ab", Encoding::Bytes => CharData::Byte(b'a'), CharError::MoreThanOneChar);
        test!(b"ab", Encoding::Utf8 => CharData::Unicode('a'), CharError::MoreThanOneChar);
        test!(b"a\\", Encoding::Bytes => CharData::Byte(b'a'), CharError::MoreThanOneChar);
        test!(b"a\\", Encoding::Utf8 => CharData::Unicode('a'), CharError::MoreThanOneChar);
        test!("ß".as_bytes(), Encoding::Bytes => CharData::Unicode('ß'), CharError::UnexpectedUnicode);
        test!("ß".as_bytes(), Encoding::Utf8 => CharData::Unicode('ß'));
        test!(b"\xff", Encoding::Bytes => CharData::Byte(b'\xff'));
        test!(b"\xff", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::InvalidUtf8);
        test!(b"\xf0\x9f\x9a", Encoding::Bytes => CharData::Byte(b'\xf0'), CharError::MoreThanOneChar);
        test!(b"\xf0\x9f\x9a", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::InvalidUtf8);
        test!(b"\xed\xa0\x80", Encoding::Bytes => CharData::Byte(b'\xed'), CharError::MoreThanOneChar);
        test!(b"\xed\xa0\x80", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::MoreThanOneChar | CharError::InvalidUtf8);
        test!(b"\\", Encoding::Bytes => CharData::Byte(0), CharError::InvalidEscape);
        test!(b"\\", Encoding::Utf8 => CharData::Unicode('\0'), CharError::InvalidEscape);
        test!(b"\\\\", Encoding::Bytes => CharData::Byte(b'\\'));
        test!(b"\\\\", Encoding::Utf8 => CharData::Unicode('\\'));
        test!(b"\\n", Encoding::Bytes => CharData::Byte(b'\n'));
        test!(b"\\n", Encoding::Utf8 => CharData::Unicode('\n'));
        test!(b"\\a", Encoding::Bytes => CharData::Byte(b'a'), CharError::InvalidEscape);
        test!(b"\\a", Encoding::Utf8 => CharData::Unicode('a'), CharError::InvalidEscape);
        test!("\\ß".as_bytes(), Encoding::Bytes => CharData::Unicode('ß'), CharError::InvalidEscape | CharError::UnexpectedUnicode);
        test!("\\ß".as_bytes(), Encoding::Utf8 => CharData::Unicode('ß'), CharError::InvalidEscape);
        test!(b"\\\xff", Encoding::Bytes => CharData::Byte(b'\xff'), CharError::InvalidEscape);
        test!(b"\\\xff", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::InvalidEscape | CharError::InvalidUtf8);
        test!(b"\\\xf0\x9f\x9a", Encoding::Bytes => CharData::Byte(b'\xf0'), CharError::InvalidEscape | CharError::MoreThanOneChar);
        test!(b"\\\xf0\x9f\x9a", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::InvalidEscape | CharError::InvalidUtf8);
        test!(b"\\\xed\xa0\x80", Encoding::Bytes => CharData::Byte(b'\xed'), CharError::InvalidEscape | CharError::MoreThanOneChar);
        test!(b"\\\xed\xa0\x80", Encoding::Utf8 => CharData::Unicode('\u{fffd}'), CharError::InvalidEscape | CharError::MoreThanOneChar | CharError::InvalidUtf8);
    }

    fn unescape_byte(b: u8) -> Option<u8> {
        match b {
            b'\'' => Some(b'\''),
            b'\\' => Some(b'\\'),
            b'n' => Some(b'\n'),
            b'r' => Some(b'\r'),
            b't' => Some(b'\t'),
            _ => None,
        }
    }
}
