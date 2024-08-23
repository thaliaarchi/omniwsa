//! Lexer for the Palaiologos Whitespace assembly dialect.

use std::borrow::Cow;

use bstr::ByteSlice;
use enumset::EnumSet;

use crate::{
    dialects::Palaiologos,
    lex::{ByteScanner, Lex},
    syntax::Opcode,
    tokens::{
        comment::{LineCommentStyle, LineCommentToken},
        label::{LabelError, LabelStyle, LabelToken},
        mnemonics::MnemonicToken,
        spaces::{
            ArgSepStyle, ArgSepToken, EofToken, InstSepStyle, InstSepToken, LineTermStyle,
            LineTermToken, SpaceToken,
        },
        string::{
            CharData, CharError, CharToken, QuoteStyle, StringData, StringError, StringToken,
        },
        ErrorToken, Token,
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
            return Token::from(EofToken);
        }

        match scan.next_byte() {
            b @ (b'A'..=b'Z' | b'a'..=b'z' | b'_') => {
                let rest = &scan.src()[scan.start_offset()..];
                if let Some((mnemonic, opcodes)) = scan_mnemonic(rest, self.dialect) {
                    scan.bump_bytes_no_lf(mnemonic.len() - 1);
                    return Token::from(MnemonicToken {
                        mnemonic: mnemonic.into(),
                        opcode: opcodes[0],
                    });
                }
                // Try to scan a hex literal, even though the first digit is not
                // allowed to be a letter. This does not conflict with any
                // mnemonics.
                if matches!(b, b'A'..=b'F' | b'a'..=b'f') {
                    let pos = scan.end();
                    scan.bump_while(|b| b.is_ascii_hexdigit() || b == b'_');
                    if scan.bump_if(|b| b == b'h' || b == b'H') {
                        return self
                            .dialect
                            .integers()
                            .parse(scan.text().into(), &mut self.digit_buf)
                            .into();
                    }
                    scan.revert(pos);
                }
                // Consume as much as possible, until a valid mnemonic.
                while !scan.eof() && matches!(scan.peek_byte(), b'A'..=b'Z' | b'a'..=b'z' | b'_') {
                    if scan_mnemonic(scan.rest(), self.dialect).is_some() {
                        break;
                    }
                    scan.next_byte();
                }
                Token::from(MnemonicToken {
                    mnemonic: scan.text().into(),
                    opcode: Opcode::Invalid,
                })
            }
            b @ (b'0'..=b'9' | b'-' | b'+') => {
                // Extend the syntax to handle '+', starting with a hex letter,
                // and '_' digit separators, just for errors.
                scan.bump_while(|b| b.is_ascii_hexdigit() || b == b'_');
                scan.bump_if(|b| matches!(b, b'h' | b'H' | b'o' | b'O'));
                if (b == b'-' || b == b'+') && scan.text().len() == 1 {
                    Token::from(ErrorToken::UnrecognizedChar {
                        text: scan.text().into(),
                    })
                } else {
                    self.dialect
                        .integers()
                        .parse(scan.text().into(), &mut self.digit_buf)
                        .into()
                }
            }
            sigil @ (b'@' | b'%') => {
                let style = if sigil == b'@' {
                    LabelStyle::AtSigil
                } else {
                    LabelStyle::PercentSigil
                };
                scan.bump_while(|b| matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_'));
                let text = scan.text();
                let errors = match text.get(1) {
                    None => LabelError::Empty.into(),
                    Some(b'0'..=b'9') => LabelError::StartsWithDigit.into(),
                    _ => EnumSet::empty(),
                };
                Token::from(LabelToken {
                    label: text[1..].into(),
                    style,
                    errors,
                })
            }
            b'\'' => {
                let (unescaped, errors, len) = match *scan.rest() {
                    [b'\\', b'\n', ..] | [b'\\'] => (0, CharError::Unterminated.into(), 1),
                    [b'\\', b, ref rest @ ..] => {
                        let b = match b {
                            b'a' => b'\x07',
                            b'b' => b'\x08',
                            b'f' => b'\x0c',
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            b'v' => b'\x0b',
                            _ => b,
                        };
                        if rest.starts_with(b"'") {
                            (b, EnumSet::empty(), 3)
                        } else {
                            (b, CharError::Unterminated.into(), 2)
                        }
                    }
                    [b'\'', ..] => (0, CharError::Empty.into(), 1),
                    [b'\n', ..] => (0, CharError::Unterminated.into(), 0),
                    [b, b'\'', ..] => (b, EnumSet::empty(), 2),
                    _ => (0, CharError::Unterminated.into(), 0),
                };
                let literal =
                    &scan.rest()[..len - !errors.contains(CharError::Unterminated) as usize];
                scan.bump_bytes(len);
                Token::from(CharToken {
                    literal: literal.into(),
                    unescaped: CharData::Byte(unescaped),
                    quotes: QuoteStyle::Single,
                    errors,
                })
            }
            b'"' => {
                let (unescaped, errors, len) = scan_string(scan.rest());
                let literal =
                    &scan.rest()[..len - !errors.contains(StringError::Unterminated) as usize];
                scan.bump_bytes(len);
                Token::from(StringToken {
                    literal: literal.into(),
                    unescaped: StringData::Bytes(unescaped),
                    quotes: QuoteStyle::Double,
                    errors,
                })
            }
            b';' => {
                scan.bump_while(|b| b != b'\n');
                Token::from(LineCommentToken {
                    text: &scan.text()[1..],
                    style: LineCommentStyle::Semi,
                })
            }
            b',' => ArgSepToken::from(ArgSepStyle::Comma).into(),
            b'/' => InstSepToken::from(InstSepStyle::Slash).into(),
            b'\n' => LineTermToken::from(LineTermStyle::Lf).into(),
            b' ' | b'\t' | b'\r' | b'\x0c' => {
                scan.bump_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\x0c'));
                SpaceToken::from(scan.text()).into()
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
                Token::from(ErrorToken::UnrecognizedChar {
                    text: scan.text().into(),
                })
            }
        }
    }
}

/// Tries to scan a mnemonic at the start of the bytes.
fn scan_mnemonic<'s>(s: &'s [u8], dialect: &Palaiologos) -> Option<(&'s [u8], &'static [Opcode])> {
    let chunk = &s[..Palaiologos::MAX_MNEMONIC_LEN.min(s.len())];
    for len in (1..=chunk.len()).rev() {
        let mnemonic = &chunk[..len];
        if let Some(opcodes) = dialect.mnemonics().get_opcodes(mnemonic) {
            return Some((mnemonic, opcodes));
        }
    }
    None
}

/// Scans a string at the start of the bytes and returns the unquoted and
/// unescaped string, and the number of bytes consumed. The string must start at
/// the byte after the open `"`.
fn scan_string(s: &[u8]) -> (Cow<'_, [u8]>, EnumSet<StringError>, usize) {
    let Some(mut j) = s.find_byteset(b"\"\\\n") else {
        return (s.into(), StringError::Unterminated.into(), s.len());
    };
    if s[j] == b'"' {
        return (s[..j].into(), EnumSet::empty(), j + 1);
    } else if s[j] == b'\n' {
        return (s[..j].into(), StringError::Unterminated.into(), j + 1);
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
            b'\n' => {
                return (unquoted.into(), StringError::Unterminated.into(), j);
            }
            _ => unreachable!(),
        }
        i = j + 1;
        let Some(j2) = s[i..].find_byteset(b"\"\\\n") else {
            break;
        };
        j = j2;
    }
    (unquoted.into(), StringError::Unterminated.into(), s.len())
}
