//! Lexer for the Palaiologos Whitespace assembly dialect.

use bstr::ByteSlice;
use enumset::EnumSet;

use crate::{
    dialects::Palaiologos,
    lex::{Lex, Scanner},
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
// - Write CST tests for error recovery.

/// A lexer for tokens in the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Lexer<'s, 'd> {
    dialect: &'d Palaiologos,
    scan: Scanner<'s>,
    digit_buf: Vec<u8>,
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for Palaiologos-dialect source text.
    pub fn new(src: &'s [u8], dialect: &'d Palaiologos) -> Self {
        Lexer {
            dialect,
            scan: Scanner::new(src),
            digit_buf: Vec::new(),
        }
    }
}

impl<'s> Lex<'s> for Lexer<'s, '_> {
    fn next_token(&mut self) -> Token<'s> {
        let scan = &mut self.scan;
        scan.start_next();

        if scan.eof() {
            return Token::from(EofToken);
        }

        match scan.next_char_or_replace() {
            ch @ ('A'..='Z' | 'a'..='z' | '_') => {
                if let Some((mnemonic, opcodes)) =
                    scan_mnemonic(scan.rest_from_start(), self.dialect)
                {
                    scan.bump_ascii_no_lf(mnemonic.len() - 1);
                    return Token::from(MnemonicToken {
                        mnemonic: mnemonic.into(),
                        opcode: opcodes[0],
                    });
                }
                // Try to scan a hex literal, even though the first digit is not
                // allowed to be a letter. This does not conflict with any
                // mnemonics.
                if matches!(ch, 'A'..='F' | 'a'..='f') {
                    let start = scan.end();
                    scan.bump_while_ascii(|ch| ch.is_ascii_hexdigit() || ch == b'_');
                    if scan.bump_if_ascii(|ch| ch == b'h' || ch == b'H') {
                        return self
                            .dialect
                            .integers()
                            .parse(scan.text().into(), &mut self.digit_buf)
                            .into();
                    }
                    scan.backtrack(start);
                }
                // Consume as much as possible, until a valid mnemonic.
                while scan
                    .peek_byte()
                    .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
                {
                    if scan_mnemonic(scan.rest(), self.dialect).is_some() {
                        break;
                    }
                    scan.bump_ascii_no_lf(1);
                }
                Token::from(MnemonicToken {
                    mnemonic: scan.text().into(),
                    opcode: Opcode::Invalid,
                })
            }
            ch @ ('0'..='9' | '-' | '+') => {
                // Extend the syntax to handle '+', starting with a hex letter,
                // and '_' digit separators, just for errors.
                scan.bump_while_ascii(|ch| ch.is_ascii_hexdigit() || ch == b'_');
                scan.bump_if_ascii(|ch| matches!(ch, b'h' | b'H' | b'o' | b'O'));
                if (ch == '-' || ch == '+') && scan.text().len() == 1 {
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
            sigil @ ('@' | '%') => {
                let style = if sigil == '@' {
                    LabelStyle::AtSigil
                } else {
                    LabelStyle::PercentSigil
                };
                scan.bump_while_ascii(|ch| ch.is_ascii_alphanumeric() || ch == b'_');
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
            '\'' => scan_char_literal(scan).into(),
            '"' => scan_string_literal(scan).into(),
            ';' => {
                scan.bump_until_lf();
                Token::from(LineCommentToken {
                    text: &scan.text()[1..],
                    style: LineCommentStyle::Semi,
                })
            }
            ',' => ArgSepToken::from(ArgSepStyle::Comma).into(),
            '/' => InstSepToken::from(InstSepStyle::Slash).into(),
            '\n' => LineTermToken::from(LineTermStyle::Lf).into(),
            ' ' | '\t' | '\r' | '\x0c' => {
                scan.bump_while_ascii(|ch| matches!(ch, b' ' | b'\t' | b'\r' | b'\x0c'));
                SpaceToken::from(scan.text()).into()
            }
            _ => {
                scan.bump_until_ascii(|ch| {
                    matches!(ch,
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

/// Scans a char literal. The scanner must be just after the open `'`.
fn scan_char_literal<'s>(scan: &mut Scanner<'s>) -> CharToken<'s> {
    let start = scan.start();
    let mut errors = EnumSet::empty();
    let literal = loop {
        scan.bump_until_ascii(|ch| ch == b'\'' || ch == b'\\' || ch == b'\n');
        match scan.peek_byte() {
            Some(b'\'') => {
                let literal = scan.text();
                scan.bump_ascii();
                break literal;
            }
            Some(b'\\') if scan.bump_if_ascii(|b| b != b'\n') => {
                continue;
            }
            _ => {
                errors |= CharError::Unterminated;
                scan.backtrack(start);
                scan.bump_if_ascii(|ch| ch == b'\\');
                scan.bump_if_ascii(|ch| ch != b'\n');
                break scan.text();
            }
        }
    };
    let data = match literal {
        [b'\\', b] => CharData::Byte(unescape_byte(*b)),
        [b] => CharData::Byte(*b),
        [b'\\', bs @ ..] | bs => {
            if let Some(&b) = bs.first() {
                let (ch, size) = bstr::decode_utf8(bs);
                match ch {
                    Some(ch) if !ch.is_ascii() => {
                        errors |= if size == bs.len() {
                            CharError::UnexpectedUnicode
                        } else {
                            CharError::MoreThanOneChar
                        };
                        CharData::Unicode(ch)
                    }
                    _ if literal[0] == b'\\' => CharData::Byte(unescape_byte(b)),
                    _ => CharData::Byte(b),
                }
            } else {
                errors |= CharError::Empty;
                CharData::Byte(0)
            }
        }
    };
    CharToken {
        literal: literal.into(),
        unescaped: data,
        quotes: QuoteStyle::Single,
        errors,
    }
}

/// Scans a string literal. The scanner must be just after the open `"`.
fn scan_string_literal<'s>(scan: &mut Scanner<'s>) -> StringToken<'s> {
    let mut backslashes = 0;
    let mut errors = EnumSet::empty();
    let literal = loop {
        scan.bump_until_ascii(|ch| ch == b'"' || ch == b'\\' || ch == b'\n');
        let b = scan.peek_byte();
        if b == Some(b'"') {
            let literal = scan.text();
            scan.bump_ascii();
            break literal;
        }
        if b == Some(b'\\') {
            backslashes += 1;
            if scan.bump_if_ascii(|b| b != b'\n') {
                continue;
            }
        }
        errors |= StringError::Unterminated;
        break scan.text();
    };
    let unescaped = if backslashes == 0 {
        literal.into()
    } else {
        let mut unescaped = Vec::with_capacity(literal.len() - backslashes);
        let mut s = literal;
        while let Some(i) = s.find_byte(b'\\') {
            unescaped.extend_from_slice(&s[..i]);
            if let Some(&b) = s.get(i + 1) {
                unescaped.push(unescape_byte(b));
            } else {
                break;
            }
            s = &s[i + 2..];
        }
        unescaped.extend_from_slice(s);
        unescaped.into()
    };
    StringToken {
        literal: literal.into(),
        unescaped: StringData::Bytes(unescaped),
        quotes: QuoteStyle::Double,
        errors,
    }
}

/// Resolves a backslash-escaped byte to its represented value.
fn unescape_byte(b: u8) -> u8 {
    match b {
        b'a' => b'\x07',
        b'b' => b'\x08',
        b'f' => b'\x0c',
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        b'v' => b'\x0b',
        _ => b,
    }
}
