//! Resolution of mnemonics to opcodes.

use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bstr::ByteSlice;
use derive_more::Debug as DebugCustom;

use crate::syntax::{HasError, Opcode, Pretty};

// TODO:
// - Make cross-dialect mnemonic map type, where the mnemonic key has an enum
//   with variants for different case folding strategies. They hash with the
//   most permissive case folding and compare against a string with the exact
//   case folding.
// - Make mapping from opcode to mnemonics.

/// Instruction mnemonic token.
#[derive(Clone, DebugCustom, PartialEq, Eq)]
pub struct MnemonicToken<'s> {
    /// The mnemonic text.
    #[debug("{:?}", mnemonic.as_bstr())]
    pub mnemonic: Cow<'s, [u8]>,
    /// The resolved mnemonic.
    pub opcode: Opcode,
}

/// A conventionally UTF-8 string which compares with configurable case folding.
/// When two `FoldedStr` are compared, the most permissive case folding between
/// the two is used.
#[derive(Clone, Copy, DebugCustom)]
pub struct FoldedStr<'a> {
    /// The bytes of the string, which are conventionally UTF-8, though not
    /// required to be.
    #[debug("{:?}", s.as_bstr())]
    pub s: &'a [u8],
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

impl<'a> FoldedStr<'a> {
    /// Wraps the byte string so it compares with the given case folding.
    pub const fn new(s: &'a [u8], fold: CaseFold) -> Self {
        FoldedStr { s, fold }
    }

    /// Wraps the byte string so it compares verbatim, without case folding,
    pub const fn exact(s: &'a [u8]) -> Self {
        FoldedStr::new(s, CaseFold::Exact)
    }

    /// Wraps the byte string so it compares ASCII letters case-insensitively.
    pub const fn ascii(s: &'a [u8]) -> Self {
        FoldedStr::new(s, CaseFold::Ascii)
    }

    /// Wraps the byte string so it compares ASCII letters and 'K'
    /// case-insensitively.
    pub const fn ascii_k(s: &'a [u8]) -> Self {
        FoldedStr::new(s, CaseFold::AsciiK)
    }

    /// Wraps the byte string so it compares ASCII letters, 'İ', and 'K'
    /// case-insensitively.
    pub const fn ascii_ik(s: &'a [u8]) -> Self {
        FoldedStr::new(s, CaseFold::AsciiIK)
    }
}

impl PartialEq<[u8]> for FoldedStr<'_> {
    fn eq(&self, other: &[u8]) -> bool {
        self.fold.compare(self.s, other)
    }
}

impl PartialEq for FoldedStr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.fold.max(other.fold).compare(self.s, other.s)
    }
}

impl Eq for FoldedStr<'_> {}

impl Hash for FoldedStr<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash with the most permissive case folding, so different folding
        // styles can be mixed.
        CaseFoldIter::<CaseFoldAsciiIK>::new(self.s).for_each(|b| b.hash(state))
    }
}

impl CaseFold {
    /// Compares two byte strings for equality using the specified style of case
    /// folding.
    pub fn compare(&self, a: &[u8], b: &[u8]) -> bool {
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
}

/// An iterator over case-folded bytes.
struct CaseFoldIter<'a, F> {
    s: &'a [u8],
    fold: PhantomData<F>,
}
struct CaseFoldAscii;
struct CaseFoldAsciiK;
struct CaseFoldAsciiIK;

impl<'a, F> CaseFoldIter<'a, F> {
    fn new(s: &'a [u8]) -> Self {
        CaseFoldIter {
            s,
            fold: PhantomData,
        }
    }
}

impl Iterator for CaseFoldIter<'_, CaseFoldAscii> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.s;
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
        let s = &mut self.s;
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
        let s = &mut self.s;
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
