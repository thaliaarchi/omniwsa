//! Compact sequence of Whitespace tokens.

use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    hint::unreachable_unchecked,
};

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;

// TODO:
// - Store comment lexemes inline in `data`. They would quickly exhaust all of
//   the 31 short lexemes.
// - Make the lexeme table a proper index table with hashbrown.
// - Create iterator and improve Debug impl using it.
// - Consider unchecked indexing.
// - Reconstruct spans with deltas like incremental compilers. Perhaps whenever
//   a new lexeme is added, its line/col delta could be computed and stored.

/// A compact sequence of the Whitespace tokens in a single source file.
///
/// It is optimized for few kinds of tokens and few unique lexemes per token.
/// See the [design draft](../../docs/drafts/compact_ws_cst.md) for a
/// description of its representation.
#[derive(Clone, PartialEq, Eq)]
pub struct TokenSource {
    /// Compact encoding of tokens.
    data: Vec<u8>,
    /// Interned lexical representations of tokens.
    lexemes: Vec<Vec<u8>>,
    /// Mapping from lexeme text to index in `lexemes`.
    lexeme_table: HashMap<Vec<u8>, u32>,
    /// Extension token kinds. A maximum of 256 extension tokens can be
    /// registered.
    extensions: Vec<ExtensionToken>,
}

/// A token in Whitespace.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// Standard Whitespace space.
    S = 0,
    /// Standard Whitespace tab.
    T = 1,
    /// Standard Whitespace line feed.
    L = 2,
    /// Non-standard extension token, identified by the ID returned from
    /// `TokenSource::add_extension`.
    Extension(ExtensionTokenId) = 3,
    /// Non-semantic comment.
    Comment = 4,
    /// Invalid token.
    InvalidToken = 5,
    /// Invalid UTF-8 sequence.
    InvalidUtf8 = 6,
}

/// The identifier of a [`Token`] in a [`TokenSource`].
///
/// It is implemented as the byte offset of the token in `TokenSource::data`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TokenId(usize);

/// A standard token in Whitespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StandardToken {
    /// Space.
    S,
    /// Tab.
    T,
    /// Line feed.
    L,
}

/// The definition of a non-standard token added by an extension to Whitespace.
#[derive(Clone, DebugCustom, PartialEq, Eq, Hash)]
pub struct ExtensionToken {
    /// The human-readable name of this token, used to identify it.
    name: String,
    /// The canonical lexeme for encoding this token.
    #[debug("{:?}", canon_lexeme.as_bstr())]
    canon_lexeme: Vec<u8>,
}

/// The identifier of an [`ExtensionToken`] in a [`TokenSource`].
///
/// It is implemented as the index in `TokenSource::extensions`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExtensionTokenId(u8);

impl TokenSource {
    const KIND_MASK: u8 = 0b00000111;
    const KIND_BITS: usize = 3;
    const LEXEME_MASK: u8 = 0b11111000;
    const LEXEME_BITS: usize = 5;
    const MAX_LEXEME: u8 = (1 << Self::LEXEME_BITS) - 1;

    const TAG_S: u8 = 0;
    const TAG_T: u8 = 1;
    const TAG_L: u8 = 2;
    const TAG_EXTENSION: u8 = 3;
    const TAG_COMMENT: u8 = 4;
    const TAG_INVALID_TOKEN: u8 = 5;
    const TAG_INVALID_UTF_8: u8 = 6;
    const TAG_RESERVED: u8 = 7;

    /// Constructs a new, empty `TokenSource`.
    pub fn new() -> Self {
        TokenSource {
            data: Vec::new(),
            lexemes: Vec::new(),
            lexeme_table: HashMap::new(),
            extensions: Vec::new(),
        }
    }

    /// Pushes a token and its lexeme to the end of the sequence.
    pub fn push(&mut self, tok: Token, lexeme: &[u8]) -> TokenId {
        let id = TokenId(self.data.len().try_into().unwrap());
        self.data.reserve(6); // Maximum encoded token size

        let lexeme_id = if let Some(&id) = self.lexeme_table.get(lexeme) {
            id
        } else {
            let id = u32::try_from(self.lexemes.len()).unwrap();
            self.lexemes.push(lexeme.to_owned());
            self.lexeme_table.insert(lexeme.to_owned(), id);
            id
        };
        let is_long_lexeme = lexeme_id >= Self::MAX_LEXEME as u32;
        let short_lexeme = if is_long_lexeme { 0 } else { lexeme_id as u8 };

        let kind = tok.discriminant();
        self.data.push(short_lexeme << Self::KIND_BITS | kind);
        if let Token::Extension(id) = tok {
            self.data.push(id.0);
        }
        if is_long_lexeme {
            self.data.extend_from_slice(&lexeme_id.to_le_bytes());
        }
        id
    }

    /// Gets the identified token and its lexeme.
    pub fn get(&self, id: TokenId) -> (Token, &[u8]) {
        let data = &self.data[id.as_usize()..];
        let head = data[0];
        let kind = head & Self::KIND_MASK;
        let short_lexeme = (head & Self::LEXEME_MASK) >> Self::KIND_BITS;
        let mut long_lexeme_start = 1;

        let tok = if kind == Self::TAG_EXTENSION {
            long_lexeme_start += 1;
            Token::Extension(ExtensionTokenId(data[1]))
        } else {
            match kind {
                Self::TAG_S => Token::S,
                Self::TAG_T => Token::T,
                Self::TAG_L => Token::L,
                Self::TAG_COMMENT => Token::Comment,
                Self::TAG_INVALID_TOKEN => Token::InvalidToken,
                Self::TAG_INVALID_UTF_8 => Token::InvalidUtf8,
                _ => unsafe { unreachable_unchecked() },
            }
        };

        let lexeme_id = if short_lexeme == Self::MAX_LEXEME {
            let bytes = data[long_lexeme_start..long_lexeme_start + 4]
                .try_into()
                .unwrap();
            u32::from_le_bytes(bytes)
        } else {
            short_lexeme as u32
        };
        let lexeme = &*self.lexemes[usize::try_from(lexeme_id).unwrap()];

        (tok, lexeme)
    }

    /// Registers a kind of extension token.
    pub fn add_extension(&mut self, tok: ExtensionToken) -> ExtensionTokenId {
        let id = ExtensionTokenId(self.extensions.len().try_into().unwrap());
        self.extensions.push(tok);
        id
    }

    /// Gets a registered extension token.
    pub fn get_extension(&self, id: ExtensionTokenId) -> &ExtensionToken {
        &self.extensions[id.as_usize()]
    }
}

impl Debug for TokenSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct DebugByte(u8);
        impl Debug for DebugByte {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                let kind = match self.0 & TokenSource::KIND_MASK {
                    TokenSource::TAG_S => "S",
                    TokenSource::TAG_T => "T",
                    TokenSource::TAG_L => "L",
                    TokenSource::TAG_EXTENSION => "Extension",
                    TokenSource::TAG_COMMENT => "Comment",
                    TokenSource::TAG_INVALID_TOKEN => "InvalidToken",
                    TokenSource::TAG_INVALID_UTF_8 => "InvalidUtf8",
                    TokenSource::TAG_RESERVED => "Reserved",
                    _ => unreachable!(),
                };
                let lexeme = (self.0 & TokenSource::LEXEME_MASK) >> TokenSource::KIND_BITS;
                write!(f, "{} ({kind}, {lexeme})", self.0)
            }
        }

        struct DebugData<'a>(&'a Vec<u8>);
        impl Debug for DebugData<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_list()
                    .entries(self.0.iter().map(|&b| DebugByte(b)))
                    .finish()
            }
        }

        struct DebugLexemes<'a>(&'a Vec<Vec<u8>>);
        impl Debug for DebugLexemes<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_map()
                    .entries(
                        self.0
                            .iter()
                            .enumerate()
                            .map(|(i, lexeme)| (i, lexeme.as_bstr())),
                    )
                    .finish()
            }
        }

        f.debug_struct("TokenSource")
            .field("data", &DebugData(&self.data))
            .field("lexemes", &DebugLexemes(&self.lexemes))
            .field("extensions", &self.extensions)
            .finish()
    }
}

impl Token {
    /// Gets its enum discriminant.
    #[inline]
    fn discriminant(&self) -> u8 {
        // SAFETY: We can read the discriminant at its pointer, because it is
        // `repr(u8)`.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl From<StandardToken> for Token {
    #[inline]
    fn from(tok: StandardToken) -> Self {
        match tok {
            StandardToken::S => Token::S,
            StandardToken::T => Token::T,
            StandardToken::L => Token::L,
        }
    }
}

impl From<ExtensionTokenId> for Token {
    #[inline]
    fn from(id: ExtensionTokenId) -> Self {
        Token::Extension(id)
    }
}

impl TokenId {
    #[inline]
    fn as_usize(self) -> usize {
        self.0
    }
}

impl ExtensionTokenId {
    #[inline]
    fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        let tokens: &[(Token, &[u8])] = &[
            // push 5
            (Token::Comment, b"Push"),
            (Token::S, b" "),
            (Token::Comment, b"5"),
            (Token::S, b" "),
            (Token::Comment, b"to"),
            (Token::S, b" "),
            (Token::Comment, b"the"),
            (Token::T, b"\t"),
            (Token::Comment, b"stack."),
            (Token::S, b" "),
            (Token::T, b"\t"),
            (Token::L, b"\n"),
            // printi
            (Token::Comment, b"Print"),
            (Token::T, b"\t"),
            (Token::Comment, b"the"),
            (Token::L, b"\n"),
            (Token::Comment, b"number."),
            (Token::S, b" "),
            (Token::T, b"\t"),
        ];

        let mut source = TokenSource::new();
        let mut token_ids = Vec::new();
        for &(tok, lexeme) in tokens {
            token_ids.push(source.push(tok, lexeme));
        }

        let lexemes: &[&[u8]] = &[
            b"Push", b" ", b"5", b"to", b"the", b"\t", b"stack.", b"\n", b"Print", b"number.",
        ];
        let expect = TokenSource {
            data: vec![
                TokenSource::TAG_COMMENT | 0 << TokenSource::KIND_BITS, // Push
                TokenSource::TAG_S | 1 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 2 << TokenSource::KIND_BITS, // 5
                TokenSource::TAG_S | 1 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 3 << TokenSource::KIND_BITS, // to
                TokenSource::TAG_S | 1 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 4 << TokenSource::KIND_BITS, // the
                TokenSource::TAG_T | 5 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 6 << TokenSource::KIND_BITS, // stack.
                TokenSource::TAG_S | 1 << TokenSource::KIND_BITS,
                TokenSource::TAG_T | 5 << TokenSource::KIND_BITS,
                TokenSource::TAG_L | 7 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 8 << TokenSource::KIND_BITS, // Print
                TokenSource::TAG_T | 5 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 4 << TokenSource::KIND_BITS, // the
                TokenSource::TAG_L | 7 << TokenSource::KIND_BITS,
                TokenSource::TAG_COMMENT | 9 << TokenSource::KIND_BITS, // number.
                TokenSource::TAG_S | 1 << TokenSource::KIND_BITS,
                TokenSource::TAG_T | 5 << TokenSource::KIND_BITS,
            ],
            lexemes: lexemes.iter().map(|&lexeme| lexeme.to_owned()).collect(),
            lexeme_table: lexemes
                .iter()
                .enumerate()
                .map(|(i, &lexeme)| (lexeme.to_owned(), i as u32))
                .collect(),
            extensions: vec![],
        };

        assert_eq!(source, expect);

        assert_eq!(token_ids.len(), tokens.len());
        for (&id, &expect) in token_ids.iter().zip(tokens) {
            assert_eq!(source.get(id), expect);
        }
    }
}
