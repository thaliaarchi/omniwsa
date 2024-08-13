//! Parsing for the Palaiologos Whitespace assembly dialect.

use std::{borrow::Cow, collections::HashMap};

use bstr::ByteSlice;

use crate::{
    mnemonics::AsciiLower,
    scan::ByteScanner,
    token::{CharData, Opcode, QuoteStyle, StringData, Token, TokenError, TokenKind},
};

// TODO:
// - Create another table mapping opcode to argument types and the canonical
//   mnemonic.
// - How to represent empty and overlong char literals?

/// State for parsing the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
pub struct Palaiologos {
    mnemonics: HashMap<AsciiLower<'static>, Opcode>,
}

/// A lexer for tokens in the Palaiologos Whitespace assembly dialect.
#[derive(Clone, Debug)]
struct Lexer<'s, 'd> {
    dialect: &'d Palaiologos,
    scan: ByteScanner<'s>,
}

macro_rules! mnemonics[($($mnemonic:literal => $opcode:ident,)*) => {
    &[$(($mnemonic, Opcode::$opcode),)+]
}];
static MNEMONICS: &[(&'static str, Opcode)] = mnemonics![
    "psh" => Push,
    "push" => Push,
    "dup" => Dup,
    "copy" => Copy,
    "take" => Copy,
    "pull" => Copy,
    "xchg" => Swap,
    "swp" => Swap,
    "swap" => Swap,
    "drop" => Drop,
    "dsc" => Drop,
    "slide" => Slide,
    "add" => Add,
    "sub" => Sub,
    "mul" => Mul,
    "div" => Div,
    "mod" => Mod,
    "sto" => Store,
    "rcl" => Retrieve,
    "call" => Call,
    "gosub" => Call,
    "jsr" => Call,
    "jmp" => Jmp,
    "j" => Jmp,
    "b" => Jmp,
    "jz" => Jz,
    "bz" => Jz,
    "jltz" => Jn,
    "bltz" => Jn,
    "ret" => Ret,
    "end" => End,
    "putc" => Printc,
    "putn" => Printi,
    "getc" => Readc,
    "getn" => Readi,
    "rep" => PalaiologosRep,
];
const MAX_MNEMONIC_LEN: usize = 5;

impl Palaiologos {
    /// Constructs state for the Palaiologos dialect. Only one needs to be
    /// constructed for parsing any number of programs.
    pub fn new() -> Self {
        Palaiologos {
            mnemonics: MNEMONICS
                .iter()
                .map(|&(mnemonic, opcode)| (AsciiLower(mnemonic.as_bytes()), opcode))
                .collect(),
        }
    }

    /// Parses a Whitespace assembly program in the Palaiologos dialect.
    pub fn parse<'s>(&self, src: &'s [u8]) -> Vec<Token<'s>> {
        let dialect = Palaiologos::new();
        let mut lex = Lexer::new(src, &dialect);
        let mut tokens = Vec::new();
        loop {
            let tok = lex.next_token();
            if tok.kind == TokenKind::Eof {
                return tokens;
            }
            tokens.push(tok);
        }
    }
}

impl<'s, 'd> Lexer<'s, 'd> {
    /// Constructs a new lexer for Palaiologos-dialect source text.
    fn new(src: &'s [u8], dialect: &'d Palaiologos) -> Self {
        Lexer {
            dialect,
            scan: ByteScanner::new(src),
        }
    }

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
                    scan.wrap(TokenKind::Opcode(Opcode::Invalid))
                }
            }
            b'0'..=b'9' | b'-' => self.todo(),
            b'@' => self.todo(),
            b'%' => self.todo(),
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
                scan.wrap(TokenKind::Char { data, quotes })
            }
            b'"' => {
                let (unquoted, quotes, len) = scan_string(scan.rest());
                scan.bump_bytes(len);
                scan.wrap(TokenKind::String {
                    data: StringData::Bytes(unquoted),
                    quotes,
                })
            }
            b';' => self.todo(),
            b',' => scan.wrap(TokenKind::ArgSep),
            // Handle repetitions in the parser.
            b'/' | b'\n' => scan.wrap(TokenKind::InstSep),
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

    fn todo(&self) -> Token<'s> {
        self.scan.wrap(TokenKind::Word)
    }
}

/// Tries to scan a mnemonic at the start of the bytes.
fn scan_mnemonic<'s>(s: &'s [u8], dialect: &Palaiologos) -> Option<(&'s [u8], Opcode)> {
    let chunk = &s[..MAX_MNEMONIC_LEN.min(s.len())];
    let mut chunk_lower = [0; MAX_MNEMONIC_LEN];
    chunk_lower[..chunk.len()].copy_from_slice(chunk);
    chunk_lower.iter_mut().for_each(|b| *b |= 0x20);

    for len in (1..chunk.len()).rev() {
        let mnemonic = &chunk[..len];
        if let Some(&opcode) = dialect.mnemonics.get(&AsciiLower(mnemonic)) {
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
