//! Lexer for the Palaiologos Whitespace assembly dialect.

use std::borrow::Cow;

use bstr::ByteSlice;
use enumset::EnumSet;

use crate::{
    dialects::Palaiologos,
    scan::ByteScanner,
    token_stream::Lex,
    tokens::{
        integer::IntegerToken,
        string::{CharData, CharToken, QuoteStyle, StringData, StringToken},
        LabelError, LineCommentError, Opcode, Token, TokenError, TokenKind,
    },
};

// TODO:
// - How to represent empty and overlong char literals?

/// A lexer for tokens in the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s, 'd> {
    dialect: &'d Palaiologos,
    scan: ByteScanner<'s>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for Palaiologos-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d Palaiologos) -> Self {
        Lexer {
            dialect,
            scan: ByteScanner::new(src),
            digit_buf: Vec::new(),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s, '_> {
    /// Scans the next token from the source.
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.reset();

        if scan.eof() {
            return Token::new(b"", TokenKind::Eof);
        }

        match scan.next_byte() {
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let rest = &scan.src()[scan.start_offset()..];
                if let Some((mnemonic, opcode)) = scan_mnemonic(rest, self.dialect) {
                    scan.bump_bytes_no_lf(mnemonic.len() - 1);
                    Token::new(mnemonic, TokenKind::Opcode(opcode))
                } else {
                    // Consume as much as possible for an error, until a valid
                    // mnemonic.
                    while !scan.eof()
                        && matches!(scan.peek_byte(), b'A'..=b'Z' | b'a'..=b'z' | b'_')
                    {
                        if scan_mnemonic(scan.rest(), self.dialect).is_some() {
                            break;
                        }
                        scan.next_byte();
                    }
                    scan.wrap(Opcode::Invalid)
                }
            }
            b @ (b'0'..=b'9' | b'-') => {
                if b == b'-' && !scan.bump_if(|b| matches!(b, b'0'..=b'9')) {
                    scan.wrap(TokenError::UnrecognizedChar)
                } else {
                    scan.bump_while(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f'));
                    // Extend the syntax to handle octal, just for errors.
                    scan.bump_if(|b| matches!(b, b'h' | b'H' | b'o' | b'O'));
                    let int = IntegerToken::parse_palaiologos(scan.text(), &mut self.digit_buf);
                    scan.wrap(int)
                }
            }
            b'@' | b'%' => {
                scan.bump_while(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'));
                let text = scan.text();
                let errors = match text.get(1) {
                    None => LabelError::Empty.into(),
                    Some(b'0'..=b'9') => LabelError::StartsWithDigit.into(),
                    _ => EnumSet::empty(),
                };
                scan.wrap(TokenKind::Label {
                    sigil: &text[..1],
                    label: text[1..].into(),
                    errors,
                })
            }
            b'\'' => {
                let (data, quotes, len) = match *scan.rest() {
                    [b'\\', b, b'\'', ..] => (CharData::Byte(b), QuoteStyle::Single, 3),
                    // A buggy escape, that is parsed as `'\''`.
                    [b'\\', b'\'', ..] => (CharData::Byte(b'\''), QuoteStyle::Single, 2),
                    [b, b'\'', ..] => (CharData::Byte(b), QuoteStyle::Single, 2),
                    [b'\'', ..] => (CharData::Error, QuoteStyle::Single, 1),
                    [b'\\', ..] => (CharData::Error, QuoteStyle::UnclosedSingle, 1),
                    _ => (CharData::Error, QuoteStyle::UnclosedSingle, 0),
                };
                scan.bump_bytes(len);
                scan.wrap(CharToken { data, quotes })
            }
            b'"' => {
                let (unquoted, quotes, len) = scan_string(scan.rest());
                scan.bump_bytes(len);
                scan.wrap(StringToken {
                    data: StringData::Bytes(unquoted),
                    quotes,
                })
            }
            b';' => {
                scan.bump_while(|b| b != b'\n');
                let errors = if scan.bump_if(|b| b == b'\n') {
                    EnumSet::empty()
                } else {
                    LineCommentError::MissingLf.into()
                };
                let text = scan.text();
                scan.wrap(TokenKind::LineComment {
                    prefix: &text[..1],
                    text: &text[1..],
                    errors,
                })
            }
            b',' => scan.wrap(TokenKind::ArgSep),
            // Handle instruction separator and LF repetitions in the parser.
            b'/' => scan.wrap(TokenKind::InstSep),
            b'\n' => scan.wrap(TokenKind::LineTerm),
            b' ' | b'\t' | b'\r' | b'\x0c' => {
                scan.bump_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\x0c'));
                scan.wrap(TokenKind::Space)
            }
            _ => {
                scan.bump_while(|b| {
                    !matches!(b,
                        b'A'..=b'Z'
                        | b'a'..=b'z'
                        | b'_'
                        | b'0'..=b'9'
                        | b'-'
                        | b'@'
                        | b'%'
                        | b'\''
                        | b'"'
                        | b';'
                        | b','
                        | b'/'
                        | b'\n'
                        | b' '
                        | b'\t'
                        | b'\r'
                        | b'\x0c'
                    )
                });
                scan.wrap(TokenKind::Error(TokenError::UnrecognizedChar))
            }
        }
    }
}

/// Tries to scan a mnemonic at the start of the bytes.
fn scan_mnemonic<'s>(s: &'s [u8], dialect: &Palaiologos) -> Option<(&'s [u8], Opcode)> {
    let chunk = &s[..Palaiologos::MAX_MNEMONIC_LEN.min(s.len())];
    let mut chunk_lower = [0; Palaiologos::MAX_MNEMONIC_LEN];
    chunk_lower[..chunk.len()].copy_from_slice(chunk);
    chunk_lower.iter_mut().for_each(|b| *b |= 0x20);

    for len in (1..chunk.len()).rev() {
        let mnemonic = &chunk[..len];
        if let Some(opcode) = dialect.get_opcode(mnemonic) {
            return Some((mnemonic, opcode));
        }
    }
    None
}

/// Scans a string at the start of the bytes and returns the unquoted and
/// unescaped string, and the number of bytes consumed. The string must start at
/// the byte after the open `"`.
fn scan_string(s: &[u8]) -> (Cow<'_, [u8]>, QuoteStyle, usize) {
    let Some(mut j) = s.find_byteset(b"\"\\") else {
        return (s.into(), QuoteStyle::UnclosedDouble, s.len());
    };
    if s[j] == b'"' {
        return (s[..j].into(), QuoteStyle::Double, j + 1);
    }
    let mut unquoted = Vec::new();
    let mut i = 0;
    loop {
        unquoted.extend_from_slice(&s[i..j]);
        match s[j] {
            b'"' => {
                return (unquoted.into(), QuoteStyle::Double, j + 1);
            }
            b'\\' => {
                j += 1;
                if j >= s.len() {
                    break;
                }
                unquoted.push(s[j]);
            }
            _ => unreachable!(),
        }
        i = j + 1;
        let Some(j2) = s[i..].find_byteset(b"\"\\") else {
            break;
        };
        j = j2;
    }
    (unquoted.into(), QuoteStyle::UnclosedDouble, s.len())
}
