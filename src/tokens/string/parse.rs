use bstr::{ByteSlice, ByteVec};
use derive_more::Debug as DebugCustom;
use enumset::EnumSet;

use crate::{
    lex::Scanner,
    tokens::string::{
        CharData, CharError, CharToken, Encoding, QuoteStyle, StringError, StringToken,
    },
};

// TODO:
// - When lexing an unterminated char, it should recover by scanning until the
//   next whitespace.
// - For invalid escape error recovery, a fallback unescaping algorithm should
//   be tried which handles the union of common escapes.
// - Unescape callbacks only need ASCII, not a char.
// - Single-character unescape callbacks should be changed to an ASCII-to-char
//   table: a perfect hash table or a 128-bit bitset with a dense array indexed
//   by popcnt.

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
        let text = loop {
            self.bump_until_ascii(|ch| ch == b'"' || ch == b'\\' || ch == b'\n');
            let b = self.peek_byte();
            if b == Some(b'"') {
                let literal = self.text();
                self.bump_ascii();
                break literal;
            }
            if b == Some(b'\\') {
                self.bump_ascii();
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
            literal: text[1..].into(),
            backslashes,
            errors,
        }
    }

    /// Scans a single-line character literal. The scanner must be just after
    /// the open `'`.
    pub fn char_lit_oneline(&mut self) -> CharScan<'s> {
        let start = self.end();
        let mut errors = EnumSet::empty();
        let literal = loop {
            self.bump_until_ascii(|ch| ch == b'\'' || ch == b'\\' || ch == b'\n');
            match self.peek_byte() {
                Some(b'\'') => {
                    let literal = self.text_from_offset(start.offset());
                    self.bump_ascii();
                    break literal;
                }
                Some(b'\\') => {
                    self.bump_ascii();
                    if self.bump_unless_ascii(|b| b == b'\n') {
                        continue;
                    }
                }
                _ => {}
            }
            errors |= CharError::Unterminated;
            self.backtrack(start);
            self.bump_if_ascii(|ch| ch == b'\\');
            self.bump_unless_ascii(|ch| ch == b'\n');
            break self.text_from_offset(start.offset());
        };
        if self.has_invalid_utf8() {
            errors |= CharError::InvalidUtf8;
        }
        CharScan { literal, errors }
    }
}

impl<'s> StringScan<'s> {
    /// Unescapes the string literal, only handling single-char escape
    /// sequences.
    pub fn unescape_simple<F: Fn(char) -> Option<char>>(
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
                let (ch, size) = bstr::decode_utf8(&s[i + 1..]);
                if let Some(unescaped_ch) = ch.and_then(|ch| unescape(ch)) {
                    unescaped.push_char(unescaped_ch);
                } else {
                    // Drop the `\` for an invalid escape sequence like C
                    // `printf`.
                    errors |= StringError::InvalidEscape;
                    unescaped.extend_from_slice(&s[i + 1..i + 1 + size]);
                }
                s = &s[i + 1 + size..];
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
    /// single-char escape sequences.
    pub fn unescape_simple<F: Fn(char) -> Option<char>>(
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
            [b'\\', bs @ ..] => (bs, true),
            bs => (bs, false),
        };
        let (ch, size) = bstr::decode_utf8(bs);
        if size != bs.len() {
            errors |= CharError::MultipleChars;
        }
        let data = match ch {
            Some(mut ch) => {
                // TODO: Use if-let chain once stabilized.
                if escaped {
                    if let Some(unescaped) = unescape(ch) {
                        ch = unescaped;
                    } else {
                        errors |= CharError::InvalidEscape;
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
            None => {
                if escaped {
                    errors |= CharError::InvalidEscape;
                }
                match encoding {
                    Encoding::Utf8 => {
                        CharData::Unicode(if bs.is_empty() { '\0' } else { '\u{fffd}' })
                    }
                    Encoding::Bytes => {
                        if size > 1 {
                            errors |= CharError::MultipleChars;
                        }
                        CharData::Byte(if bs.is_empty() { b'\0' } else { bs[0] })
                    }
                }
            }
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
    fn scan_string() {
        let mut scan = Scanner::new(b"\"abc\\n123\\\"456\"rest");
        assert!(scan.bump_if_ascii(|b| b == b'"'));
        let s = scan.string_lit_oneline();
        assert_eq!(scan.text().as_bstr(), b"\"abc\\n123\\\"456\"".as_bstr());
        assert_eq!(scan.rest().as_bstr(), b"rest".as_bstr());
        assert_eq!(
            s,
            StringScan {
                literal: b"abc\\n123\\\"456",
                backslashes: 2,
                errors: EnumSet::empty()
            },
        );
    }

    #[test]
    fn scan_char() {
        macro_rules! test(($text:literal + $rest:literal => $literal:literal $(, $($errors:tt)+)?) => {
            let mut scan = Scanner::new(concat!($text, $rest).as_bytes());
            assert!(scan.bump_if_ascii(|b| b == b'\''));
            let c = scan.char_lit_oneline();
            assert_eq!(scan.text().as_bstr(), $text.as_bytes().as_bstr());
            assert_eq!(scan.rest().as_bstr(), $rest.as_bytes().as_bstr());
            assert_eq!(
                c,
                CharScan {
                    literal: $literal,
                    errors: enum_set!($($($errors)+)?),
                },
            );
        });

        use CharError::*;
        test!("'x'" + "rest" => b"x");
        test!("'\\n'" + "rest" => b"\\n");
        test!("'\\''" + "rest" => b"\\'");
        test!("'\\\\'" + "rest" => b"\\\\");
        test!("'xyz'" + "rest" => b"xyz");
        test!("'x\\ny\\'z\\\\'" + "rest" => b"x\\ny\\'z\\\\");
        test!("'x" + "yz" => b"x", Unterminated);
        test!("'x" + "yz\nrest" => b"x", Unterminated);
    }

    #[test]
    fn unescape_string() {
        #[track_caller]
        fn test(
            literal: &[u8],
            encoding: Encoding,
            expect_unescaped: &[u8],
            expect_errors: EnumSet<StringError>,
        ) {
            let mut errors = EnumSet::empty();
            if str::from_utf8(literal).is_err() {
                errors |= StringError::InvalidUtf8;
            }
            let scanned = StringScan {
                literal,
                backslashes: literal.iter().filter(|&&b| b == b'\\').count(),
                errors,
            };
            let expect = StringToken {
                literal: literal.into(),
                unescaped: expect_unescaped.into(),
                encoding,
                quotes: QuoteStyle::Double,
                errors: expect_errors,
            };
            let tok = scanned.unescape_simple(unescape, encoding);
            assert_eq!(tok, expect);
        }
        macro_rules! test(($escaped:expr, $encoding:expr => $unescaped:expr $(, $($errors:tt)+)?) => {
            test($escaped, $encoding, $unescaped, enum_set!($($($errors)+)?))
        });

        use {Encoding::*, StringError::*};
        test!(b"", Bytes => b"");
        test!(b"", Utf8 => b"");
        test!(b"abc", Bytes => b"abc");
        test!(b"abc", Utf8 => b"abc");
        test!(b"\\", Bytes => b"", InvalidEscape);
        test!(b"\\", Utf8 => b"", InvalidEscape);
        test!(b"\\\\", Bytes => b"\\");
        test!(b"\\\\", Utf8 => b"\\");
        test!(b"\\n", Bytes => b"\n");
        test!(b"\\n", Utf8 => b"\n");
        test!(b"\\a", Bytes => b"a", InvalidEscape);
        test!(b"\\a", Utf8 => b"a", InvalidEscape);
        test!("\\ß".as_bytes(), Bytes => "ß".as_bytes(), InvalidEscape);
        test!("\\ß".as_bytes(), Utf8 => "ß".as_bytes(), InvalidEscape);
        test!(b"\\\xff", Bytes => b"\xff", InvalidEscape);
        test!(b"\\\xff", Utf8 => b"\xff", InvalidEscape | InvalidUtf8);
        test!(b"\\\xf0\x9f\x9a", Bytes => b"\xf0\x9f\x9a", InvalidEscape);
        test!(b"\\\xf0\x9f\x9a", Utf8 => b"\xf0\x9f\x9a", InvalidEscape | InvalidUtf8);
        test!(b"\\\xed\xa0\x80", Bytes => b"\xed\xa0\x80", InvalidEscape);
        test!(b"\\\xed\xa0\x80", Utf8 => b"\xed\xa0\x80", InvalidEscape | InvalidUtf8);
    }

    #[test]
    fn unescape_char() {
        #[track_caller]
        fn test(
            literal: &[u8],
            encoding: Encoding,
            expect_unescaped: CharData,
            expect_errors: EnumSet<CharError>,
        ) {
            let mut errors = EnumSet::empty();
            if str::from_utf8(literal).is_err() {
                errors |= CharError::InvalidUtf8;
            }
            let scanned = CharScan { literal, errors };
            let expect = CharToken {
                literal: literal.into(),
                unescaped: expect_unescaped,
                quotes: QuoteStyle::Single,
                errors: expect_errors,
            };
            let tok = scanned.unescape_simple(unescape, encoding);
            assert_eq!(tok, expect);
        }
        macro_rules! test(($escaped:expr, $encoding:expr => $unescaped:expr $(, $($errors:tt)+)?) => {
            test($escaped, $encoding, $unescaped, enum_set!($($($errors)+)?))
        });

        // b"\xff" is a byte that cannot appear anywhere in UTF-8.
        // b"\xf0\x9f\x9a" is the encoding for 🚇 (U+1F687) without its final
        // byte b'\x87'. b"\xed\xa0\x80" is an unpaired surrogate half (U+D800).

        use {CharData::*, CharError::*, Encoding::*};
        test!(b"", Bytes => Byte(b'\0'), Empty);
        test!(b"", Utf8 => Unicode('\0'), Empty);
        test!(b"a", Bytes => Byte(b'a'));
        test!(b"a", Utf8 => Unicode('a'));
        test!(b"ab", Bytes => Byte(b'a'), MultipleChars);
        test!(b"ab", Utf8 => Unicode('a'), MultipleChars);
        test!(b"a\\", Bytes => Byte(b'a'), MultipleChars);
        test!(b"a\\", Utf8 => Unicode('a'), MultipleChars);
        test!("ß".as_bytes(), Bytes => Unicode('ß'), UnexpectedUnicode);
        test!("ß".as_bytes(), Utf8 => Unicode('ß'));
        test!(b"\xff", Bytes => Byte(b'\xff'));
        test!(b"\xff", Utf8 => Unicode('\u{fffd}'), InvalidUtf8);
        test!(b"\xf0\x9f\x9a", Bytes => Byte(b'\xf0'), MultipleChars);
        test!(b"\xf0\x9f\x9a", Utf8 => Unicode('\u{fffd}'), InvalidUtf8);
        test!(b"\xed\xa0\x80", Bytes => Byte(b'\xed'), MultipleChars);
        test!(b"\xed\xa0\x80", Utf8 => Unicode('\u{fffd}'), MultipleChars | InvalidUtf8);
        test!(b"\\", Bytes => Byte(b'\0'), InvalidEscape);
        test!(b"\\", Utf8 => Unicode('\0'), InvalidEscape);
        test!(b"\\\\", Bytes => Byte(b'\\'));
        test!(b"\\\\", Utf8 => Unicode('\\'));
        test!(b"\\n", Bytes => Byte(b'\n'));
        test!(b"\\n", Utf8 => Unicode('\n'));
        test!(b"\\a", Bytes => Byte(b'a'), InvalidEscape);
        test!(b"\\a", Utf8 => Unicode('a'), InvalidEscape);
        test!("\\ß".as_bytes(), Bytes => Unicode('ß'), InvalidEscape | UnexpectedUnicode);
        test!("\\ß".as_bytes(), Utf8 => Unicode('ß'), InvalidEscape);
        test!(b"\\\xff", Bytes => Byte(b'\xff'), InvalidEscape);
        test!(b"\\\xff", Utf8 => Unicode('\u{fffd}'), InvalidEscape | InvalidUtf8);
        test!(b"\\\xf0\x9f\x9a", Bytes => Byte(b'\xf0'), InvalidEscape | MultipleChars);
        test!(b"\\\xf0\x9f\x9a", Utf8 => Unicode('\u{fffd}'), InvalidEscape | InvalidUtf8);
        test!(b"\\\xed\xa0\x80", Bytes => Byte(b'\xed'), InvalidEscape | MultipleChars);
        test!(b"\\\xed\xa0\x80", Utf8 => Unicode('\u{fffd}'), InvalidEscape | MultipleChars | InvalidUtf8);
    }

    fn unescape(b: char) -> Option<char> {
        match b {
            '\'' => Some('\''),
            '\\' => Some('\\'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            _ => None,
        }
    }
}
