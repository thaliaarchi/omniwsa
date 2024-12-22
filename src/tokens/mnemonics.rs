//! Resolution of mnemonics to opcodes.

use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    hash::{Hash, Hasher},
};

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;

use crate::syntax::{HasError, Opcode, Pretty};

// TODO:
// - Make mapping from opcode to mnemonics.
// - Make mnemonics configurable on the fly.
//   - Make the double-insertion panic an error.
//   - Add general case folding.
//   - Find a better name for K and I/K folding for the CLI. Perhaps Unicode and
//     UnicodeTr, respectively. The ASCII part is only relevant as an
//     optimization with detection.

/// Instruction mnemonic token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct MnemonicToken<'s> {
    /// The mnemonic text.
    #[debug("{:?}", mnemonic.as_bstr())]
    pub mnemonic: Cow<'s, [u8]>,
    /// The resolved mnemonic.
    pub opcode: Opcode,
}

/// A mapping from instruction mnemonic to overloaded opcodes.
#[derive(Clone, Debug)]
pub struct MnemonicMap {
    map: HashMap<FoldedStr<'static>, &'static [Opcode]>,
}

/// A conventionally UTF-8 string which compares with configurable case folding.
/// When two `FoldedStr` are compared, the most permissive case folding between
/// the two is used.
#[derive(Clone, Copy)]
pub struct FoldedStr<'a> {
    /// The bytes of the string, which are conventionally UTF-8, though not
    /// required to be.
    pub bytes: &'a [u8],
    /// The case folding used when comparing this string.
    pub fold: CaseFold,
}

/// Describes the style of case folding to perform when comparing strings. This
/// only considers case folding involving ASCII letters. The two special cases
/// in Unicode are 'İ' (U+0130: LATIN CAPITAL LETTER I WITH DOT ABOVE) and 'K'
/// (U+212A: KELVIN SIGN). 'İ' folds to 'i' in Turkic languages and to the
/// combining sequence 'i̇' (U+0069: LATIN SMALL LETTER I, U+0307: COMBINING DOT
/// ABOVE) otherwise; however some implementations which cannot change character
/// length may always fold to ASCII 'i'. 'K' always folds to 'k'.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaseFold {
    /// Compares verbatim, without case folding,
    Exact,
    /// Compares ASCII letters case-insensitively.
    Ascii,
    /// Compares ASCII letters and 'K' case-insensitively.
    AsciiK,
    /// Compares ASCII letters, 'İ', and 'K' case-insensitively.
    AsciiIK,
}

impl HasError for MnemonicToken<'_> {
    fn has_error(&self) -> bool {
        self.opcode == Opcode::Invalid
    }
}

impl Pretty for MnemonicToken<'_> {
    fn pretty(&self, buf: &mut Vec<u8>) {
        self.mnemonic.pretty(buf);
    }
}

impl MnemonicMap {
    /// Constructs an empty mnemonic map.
    pub fn new() -> Self {
        MnemonicMap {
            map: HashMap::new(),
        }
    }

    /// Associates a mnemonic with opcode overloads.
    pub fn insert(&mut self, mnemonic: FoldedStr<'static>, opcodes: &'static [Opcode]) {
        if self.map.insert(mnemonic, opcodes).is_some() {
            panic!("conflicting mnemonics");
        }
    }

    /// Gets the opcode overloads for a mnemonic.
    pub fn get_opcodes(&self, mnemonic: &[u8]) -> Option<&'static [Opcode]> {
        self.map.get(&FoldedStr::exact(mnemonic)).copied()
    }

    /// Returns the number of mnemonics in this map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether this map has no elements.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl From<&'static [(FoldedStr<'static>, &'static [Opcode])]> for MnemonicMap {
    fn from(map: &'static [(FoldedStr<'static>, &'static [Opcode])]) -> Self {
        MnemonicMap {
            map: map.iter().copied().collect(),
        }
    }
}

impl<'a> FoldedStr<'a> {
    /// Wraps the byte string so it compares with the given case folding.
    pub const fn new(bytes: &'a [u8], fold: CaseFold) -> Self {
        FoldedStr { bytes, fold }
    }

    /// Detects the minimum case folding features needed for this byte string
    /// and wraps the byte string so it compares with that case folding.
    pub const fn new_detect(bytes: &'a [u8], fold: CaseFold) -> Self {
        FoldedStr::new(bytes, fold.detect(bytes))
    }

    /// Wraps the byte string so it compares verbatim, without case folding,
    pub const fn exact(bytes: &'a [u8]) -> Self {
        FoldedStr::new(bytes, CaseFold::Exact)
    }

    /// Wraps the byte string so it compares ASCII letters case-insensitively.
    pub const fn ascii(bytes: &'a [u8]) -> Self {
        FoldedStr::new(bytes, CaseFold::Ascii)
    }

    /// Wraps the byte string so it compares ASCII letters and 'K'
    /// case-insensitively.
    pub const fn ascii_k(bytes: &'a [u8]) -> Self {
        FoldedStr::new(bytes, CaseFold::AsciiK)
    }

    /// Wraps the byte string so it compares ASCII letters, 'İ', and 'K'
    /// case-insensitively.
    pub const fn ascii_ik(bytes: &'a [u8]) -> Self {
        FoldedStr::new(bytes, CaseFold::AsciiIK)
    }

    /// Returns whether this string starts with the given prefix, folding both.
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        self.fold.starts_with(self.bytes, prefix)
    }

    /// Iterates over case-folded bytes.
    pub const fn iter(&self) -> CaseFoldIter<'a, CaseFold> {
        CaseFoldIter {
            bytes: self.bytes,
            fold: self.fold,
        }
    }
}

impl Debug for FoldedStr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("FoldedStr::")?;
        f.write_str(match self.fold {
            CaseFold::Exact => "exact",
            CaseFold::Ascii => "ascii",
            CaseFold::AsciiK => "ascii_k",
            CaseFold::AsciiIK => "ascii_ik",
        })?;
        write!(f, "({:?})", self.bytes.as_bstr())
    }
}

impl Default for FoldedStr<'_> {
    fn default() -> Self {
        FoldedStr {
            bytes: b"",
            fold: CaseFold::Exact,
        }
    }
}

impl PartialEq<[u8]> for FoldedStr<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.fold.compare(self.bytes, other)
    }
}

impl PartialEq for FoldedStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.fold.max(other.fold).compare(self.bytes, other.bytes)
    }
}

impl Eq for FoldedStr<'_> {}

impl Hash for FoldedStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash with the most permissive case folding, so different folding
        // styles can be mixed.
        CaseFoldIter::<CaseFoldAsciiIK>::new(self.bytes).for_each(|b| b.hash(state))
    }
}

impl<'a> IntoIterator for FoldedStr<'a> {
    type Item = u8;
    type IntoIter = CaseFoldIter<'a, CaseFold>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &FoldedStr<'a> {
    type Item = u8;
    type IntoIter = CaseFoldIter<'a, CaseFold>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl CaseFold {
    /// Compares two byte strings for equality using the specified style of case
    /// folding.
    pub fn compare(self, a: &[u8], b: &[u8]) -> bool {
        match self {
            CaseFold::Exact => a == b,
            CaseFold::Ascii => Iterator::eq(
                CaseFoldIter::<CaseFoldAscii>::new(a),
                CaseFoldIter::<CaseFoldAscii>::new(b),
            ),
            CaseFold::AsciiK => Iterator::eq(
                CaseFoldIter::<CaseFoldAsciiK>::new(a),
                CaseFoldIter::<CaseFoldAsciiK>::new(b),
            ),
            CaseFold::AsciiIK => Iterator::eq(
                CaseFoldIter::<CaseFoldAsciiIK>::new(a),
                CaseFoldIter::<CaseFoldAsciiIK>::new(b),
            ),
        }
    }

    /// Returns whether this string starts with the given prefix, folding both.
    pub fn starts_with(self, s: &[u8], prefix: &[u8]) -> bool {
        if self == CaseFold::Exact {
            return s.starts_with(prefix);
        }
        let mut s = FoldedStr::new(s, self).iter();
        let mut prefix = FoldedStr::new(prefix, self).iter();
        while let Some(b) = prefix.next() {
            if s.next() != Some(b) {
                return false;
            }
        }
        true
    }

    /// Detects the minimum case folding features needed for this byte string.
    const fn detect(self, bytes: &[u8]) -> CaseFold {
        let s = bytes;
        match self {
            CaseFold::Exact => return CaseFold::Exact,
            CaseFold::Ascii => {
                let mut i = 0;
                while i < s.len() {
                    if s[i].is_ascii_alphabetic() {
                        return CaseFold::Ascii;
                    }
                    i += 1;
                }
                return CaseFold::Exact;
            }
            _ => {}
        }

        let mut has_ascii = false;
        let mut has_i = false;
        let mut has_k = false;
        let mut i = 0;
        while i < s.len() {
            const I: &[u8] = "İ".as_bytes();
            const K: &[u8] = "K".as_bytes();
            match s[i] {
                b'i' | b'I' => has_i = true,
                b'k' | b'K' => has_k = true,
                b'a'..=b'z' | b'A'..=b'Z' => has_ascii = true,
                _ if i + 2 < s.len() && s[i] == I[0] && s[i + 1] == I[1] && s[i + 2] == I[2] => {
                    has_i = true;
                    i += 2;
                }
                _ if i + 2 < s.len() && s[i] == K[0] && s[i + 1] == K[1] && s[i + 2] == K[2] => {
                    has_k = true;
                    i += 2;
                }
                ..=b'\x7f' => {}
                _ => panic!("unsupported character"),
            }
            i += 1;
        }
        has_ascii |= has_i | has_k;
        match self {
            CaseFold::AsciiIK if has_i => CaseFold::AsciiIK,
            CaseFold::AsciiIK | CaseFold::AsciiK if has_k => CaseFold::AsciiK,
            CaseFold::AsciiIK | CaseFold::AsciiK | CaseFold::Ascii if has_ascii => CaseFold::Ascii,
            _ => CaseFold::Exact,
        }
    }
}

/// An iterator over case-folded bytes.
#[derive(Clone, DebugCustom)]
pub struct CaseFoldIter<'a, F> {
    #[debug("{:?}", bytes.as_bstr())]
    bytes: &'a [u8],
    fold: F,
}

#[derive(Clone, Copy, Debug, Default)]
struct CaseFoldAscii;
#[derive(Clone, Copy, Debug, Default)]
struct CaseFoldAsciiK;
#[derive(Clone, Copy, Debug, Default)]
struct CaseFoldAsciiIK;

impl<'a, F: Default> CaseFoldIter<'a, F> {
    fn new(bytes: &'a [u8]) -> Self {
        CaseFoldIter {
            bytes,
            fold: Default::default(),
        }
    }
}

impl<'a> CaseFoldIter<'a, CaseFold> {
    /// Returns the remaining bytes which have not been iterated.
    pub fn as_str(&self) -> FoldedStr<'a> {
        FoldedStr::new(self.bytes, self.fold)
    }
}

impl<'a, F> CaseFoldIter<'a, F> {
    /// Returns the remaining bytes which have not been iterated.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

impl Iterator for CaseFoldIter<'_, CaseFold> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.bytes;
        let Some(&b) = s.first() else {
            return None;
        };
        let (mut lower, mut len) = (b, 1);
        if self.fold >= CaseFold::Ascii {
            if b <= b'\x7f' {
                if (b'A'..=b'Z').contains(&b) {
                    lower |= 0x20;
                }
            } else if self.fold >= CaseFold::AsciiK {
                if s.starts_with("K".as_bytes()) {
                    (lower, len) = (b'k', "K".len());
                } else if self.fold == CaseFold::AsciiIK && s.starts_with("İ".as_bytes()) {
                    (lower, len) = (b'i', "İ".len());
                }
            }
        }
        *s = &s[len..];
        Some(lower)
    }
}

impl Iterator for CaseFoldIter<'_, CaseFoldAscii> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.bytes;
        let Some(&b) = s.first() else {
            return None;
        };
        *s = &s[1..];
        Some(if (b'A'..=b'Z').contains(&b) {
            b | 0x20
        } else {
            b
        })
    }
}

impl Iterator for CaseFoldIter<'_, CaseFoldAsciiK> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.bytes;
        let Some(&b) = s.first() else {
            return None;
        };
        let (lower, len) = if b <= b'\x7f' {
            if (b'A'..=b'Z').contains(&b) {
                (b | 0x20, 1)
            } else {
                (b, 1)
            }
        } else if s.starts_with("K".as_bytes()) {
            (b'k', "K".len())
        } else {
            // Don't bother decoding codepoints that don't lowercase to ASCII.
            (b, 1)
        };
        *s = &s[len..];
        Some(lower)
    }
}

impl Iterator for CaseFoldIter<'_, CaseFoldAsciiIK> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.bytes;
        let Some(&b) = s.first() else {
            return None;
        };
        let (lower, len) = if b <= b'\x7f' {
            if (b'A'..=b'Z').contains(&b) {
                (b | 0x20, 1)
            } else {
                (b, 1)
            }
        } else if s.starts_with("İ".as_bytes()) {
            (b'i', "İ".len())
        } else if s.starts_with("K".as_bytes()) {
            (b'k', "K".len())
        } else {
            // Don't bother decoding codepoints that don't lowercase to ASCII.
            (b, 1)
        };
        *s = &s[len..];
        Some(lower)
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::mnemonics::CaseFold;

    #[test]
    fn utf8_folding() {
        assert!(CaseFold::AsciiIK.compare(b"debug_printStack", "Debug_PrİntStacK".as_bytes()));
    }
}
