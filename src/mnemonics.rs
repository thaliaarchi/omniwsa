//! Resolution of mnemonics to opcodes.

use std::{
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
};

use bstr::ByteSlice;

/// Instruction or predefined macro opcode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opcode {
    Push,
    Dup,
    Copy,
    Swap,
    Drop,
    Slide,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Store,
    Retrieve,
    Label,
    Call,
    Jmp,
    Jz,
    Jn,
    Ret,
    End,
    Printc,
    Printi,
    Readc,
    Readi,

    /// Burghard `pushs`.
    PushString0,

    /// Burghard `option`.
    DefineOption,
    /// Burghard `ifoption` and Respace `@ifdef`.
    IfOption,
    /// Burghard `elseifoption`.
    ElseIfOption,
    /// Burghard `elseoption` and Respace `@else`.
    ElseOption,
    /// Burghard `endoption` and Respace `@endif`.
    EndOption,

    /// Burghard `include`.
    BurghardInclude,
    /// Respace `@include`.
    RespaceInclude,

    /// Burghard `valueinteger`.
    BurghardValueInteger,
    /// Burghard `valuestring`.
    BurghardValueString,

    /// Burghard `debug_printstack`.
    BurghardPrintStack,
    /// Burghard `debug_printheap`.
    BurghardPrintHeap,

    /// Burghard `jumpp`.
    BurghardJmpP,
    /// Burghard `jumpnp` or `jumppn`.
    BurghardJmpNP,
    /// Burghard `jumpnz`.
    BurghardJmpNZ,
    /// Burghard `jumppz`.
    BurghardJmpPZ,
    /// Burghard `test`.
    BurghardTest,

    /// Palaiologos `rep`.
    PalaiologosRep,

    /// An invalid mnemonic.
    Invalid,
}

/// A conventionally UTF-8 string which compares by folding to lowercase. Only
/// characters that fold to ASCII are folded, as those are all that are needed
/// for mnemonics. In addition to `[A-Z]`, those are 'İ' (U+0130, LATIN CAPITAL
/// LETTER I WITH DOT ABOVE), which maps to 'i', and 'K' (U+212A, KELVIN SIGN),
/// which maps to 'k'. This matches a subset of the case folding behavior of
/// Haskell `toLower` from `Data.Char`, which performs single
/// character-to-character mappings.
#[derive(Clone, Copy)]
pub struct Utf8LowerToAscii<'s>(pub &'s [u8]);

/// A byte string which compares by folding ASCII letters to lowercase.
#[derive(Clone, Copy)]
pub struct AsciiLower<'s>(pub &'s [u8]);

impl Iterator for Utf8LowerToAscii<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.0;
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
            // Don't bother decoding codepoints that don't lower to ASCII.
            (b, 1)
        };
        *s = &s[len..];
        Some(lower)
    }
}

impl Iterator for AsciiLower<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let s = &mut self.0;
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

macro_rules! impl_folding_traits(($Ty:ty) => {
    impl PartialEq for $Ty {
        fn eq(&self, other: &Self) -> bool {
            Iterator::eq(*self, *other)
        }
    }

    impl Eq for $Ty {}

    impl Hash for $Ty {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.for_each(|b| b.hash(state));
        }
    }

    impl Debug for $Ty {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            Debug::fmt(self.0.as_bstr(), f)
        }
    }

    impl Display for $Ty {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            Display::fmt(self.0.as_bstr(), f)
        }
    }
});

impl_folding_traits!(Utf8LowerToAscii<'_>);
impl_folding_traits!(AsciiLower<'_>);

#[cfg(test)]
mod tests {
    use crate::mnemonics::Utf8LowerToAscii;

    #[test]
    fn utf8_folding() {
        assert_eq!(
            Utf8LowerToAscii(b"debug_printStack"),
            Utf8LowerToAscii("Debug_PrİntStacK".as_bytes()),
        );
    }
}
