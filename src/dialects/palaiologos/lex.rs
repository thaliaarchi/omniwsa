//! Lexer for the Palaiologos Whitespace assembly dialect.

use std::borrow::Cow;

use bstr::ByteSlice;
use enumset::EnumSet;

use crate::{
    dialects::Palaiologos,
    lex::{ByteScanner, Lex},
    syntax::Opcode,
    tokens::{
        comment::{LineCommentError, LineCommentStyle, LineCommentToken},
        integer::IntegerToken,
        label::{LabelError, LabelToken},
        spaces::{
            ArgSepStyle, ArgSepToken, EofToken, InstSepStyle, InstSepToken, LineTermStyle,
            LineTermToken, SpaceToken,
        },
        string::{
            CharData, CharError, CharToken, QuoteStyle, StringData, StringError, StringToken,
        },
        ErrorToken, Token, TokenKind,
    },
};

// TODO:
// - Scan overlong char literals, instead of always marking unterminated.

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
            return Token::new(b"", EofToken);
        }

        match scan.next_byte() {
            b'A'..=b'Z' | b'a'..=b'z' | b'_' => {
                let rest = &scan.src()[scan.start_offset()..];
                if let Some((mnemonic, opcodes)) = scan_mnemonic(rest, self.dialect) {
                    scan.bump_bytes_no_lf(mnemonic.len() - 1);
                    Token::new(mnemonic, opcodes[0])
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
                    scan.wrap(ErrorToken::UnrecognizedChar)
                } else {
                    scan.bump_while(|b| matches!(b, b'0'..=b'9' | b'A'..=b'F' | b'a'..=b'f'));
                    // Extend the syntax to handle octal, just for errors.
                    scan.bump_if(|b| matches!(b, b'h' | b'H' | b'o' | b'O'));
                    let int =
                        IntegerToken::parse_palaiologos(scan.text().into(), &mut self.digit_buf);
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
                scan.wrap(LabelToken {
                    sigil: &text[..1],
                    label: text[1..].into(),
                    errors,
                })
            }
            b'\'' => {
                let (unescaped, errors, len) = match *scan.rest() {
                    [b'\\', b, b'\'', ..] => (CharData::Byte(b), EnumSet::empty(), 3),
                    // A buggy escape, that is parsed as `'\''`.
                    [b'\\', b'\'', ..] => (CharData::Byte(b'\''), EnumSet::empty(), 2),
                    [b, b'\'', ..] => (CharData::Byte(b), EnumSet::empty(), 2),
                    [b'\'', ..] => (CharData::Byte(0), CharError::Empty.into(), 1),
                    [b'\\', b, ..] => (CharData::Byte(b), CharError::Unterminated.into(), 2),
                    [b'\\', ..] => (CharData::Byte(0), CharError::Unterminated.into(), 1),
                    _ => (CharData::Byte(0), CharError::Unterminated.into(), 0),
                };
                let literal =
                    &scan.rest()[..len - !errors.contains(CharError::Unterminated) as usize];
                scan.bump_bytes(len);
                scan.wrap(CharToken {
                    literal: literal.into(),
                    unescaped,
                    quotes: QuoteStyle::Single,
                    errors,
                })
            }
            b'"' => {
                let (unescaped, errors, len) = scan_string(scan.rest());
                let literal =
                    &scan.rest()[..len - !errors.contains(StringError::Unterminated) as usize];
                scan.bump_bytes(len);
                scan.wrap(StringToken {
                    literal: literal.into(),
                    unescaped: StringData::Bytes(unescaped),
                    quotes: QuoteStyle::Double,
                    errors,
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
                scan.wrap(LineCommentToken {
                    text: &text[1..],
                    style: LineCommentStyle::Semi,
                    errors,
                })
            }
            b',' => scan.wrap(ArgSepToken::from(ArgSepStyle::Comma)),
            // Handle instruction separator and LF repetitions in the parser.
            b'/' => scan.wrap(InstSepToken::from(InstSepStyle::Slash)),
            b'\n' => scan.wrap(LineTermToken::from(LineTermStyle::Lf)),
            b' ' | b'\t' | b'\r' | b'\x0c' => {
                scan.bump_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\x0c'));
                scan.wrap(SpaceToken::from(scan.text()))
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
                scan.wrap(TokenKind::Error(ErrorToken::UnrecognizedChar))
            }
        }
    }
}

/// Tries to scan a mnemonic at the start of the bytes.
fn scan_mnemonic<'s>(s: &'s [u8], dialect: &Palaiologos) -> Option<(&'s [u8], &'static [Opcode])> {
    let chunk = &s[..Palaiologos::MAX_MNEMONIC_LEN.min(s.len())];
    let mut chunk_lower = [0; Palaiologos::MAX_MNEMONIC_LEN];
    chunk_lower[..chunk.len()].copy_from_slice(chunk);
    chunk_lower.iter_mut().for_each(|b| *b |= 0x20);

    for len in (1..chunk.len()).rev() {
        let mnemonic = &chunk[..len];
        if let Some(opcodes) = dialect.get_opcodes(mnemonic) {
            return Some((mnemonic, opcodes));
        }
    }
    None
}

/// Scans a string at the start of the bytes and returns the unquoted and
/// unescaped string, and the number of bytes consumed. The string must start at
/// the byte after the open `"`.
fn scan_string(s: &[u8]) -> (Cow<'_, [u8]>, EnumSet<StringError>, usize) {
    let Some(mut j) = s.find_byteset(b"\"\\") else {
        return (s.into(), StringError::Unterminated.into(), s.len());
    };
    if s[j] == b'"' {
        return (s[..j].into(), EnumSet::empty(), j + 1);
    }
    let mut unquoted = Vec::new();
    let mut i = 0;
    loop {
        unquoted.extend_from_slice(&s[i..j]);
        match s[j] {
            b'"' => {
                return (unquoted.into(), EnumSet::empty(), j + 1);
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
    (unquoted.into(), StringError::Unterminated.into(), s.len())
}
