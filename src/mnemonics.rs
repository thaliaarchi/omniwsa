use std::fmt::{self, Debug, Display, Formatter};

/// Instruction or predefined macro mnemonic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mnemonic {
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

    /// An invalid mnemonic.
    Error,
}

/// A string validated to be in canonical form, that is, that it consists only
/// of the characters `[A-Za-z_]`
#[derive(Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CanonStr(str);

impl CanonStr {
    /// Validates that a string is in canonical form and wrap it. Panics if it
    /// is not canonical.
    pub const fn new(s: &str) -> &Self {
        match CanonStr::validate(s) {
            Some(canon) => canon,
            None => panic!("string not canonical"),
        }
    }

    /// Validates that a string is in canonical form and wrap it.
    pub const fn validate(s: &str) -> Option<&Self> {
        if Self::is_canon(s) {
            Some(unsafe { &*(s as *const str as *const Self) })
        } else {
            None
        }
    }

    /// Returns whether a string is in canonical form.
    pub const fn is_canon(s: &str) -> bool {
        let s = s.as_bytes();
        let mut i = 0;
        while i < s.len() {
            if !matches!(s[i], b'A'..=b'Z' | b'a'..=b'z' | b'_') {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Returns a reference to the canonical string.
    #[inline]
    pub const fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a reference to the canonical string as bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Compare two strings for equality, ignoring case. In addition to ASCII
    /// case folding, 'İ' (U+0130, LATIN CAPITAL LETTER I WITH DOT ABOVE) maps
    /// to 'i' and 'K' (U+212A, KELVIN SIGN) maps to 'k'. This matches the case
    /// folding behavior of Haskell `toLower` from `Data.Char`, which performs
    /// single character-to-character mappings.
    pub fn eq_lower(&self, other: &str) -> bool {
        let (canon, other) = (self.as_bytes(), other.as_bytes());
        let (mut i, mut j) = (0, 0);
        while i < canon.len() && j < other.len() {
            // Map b1 and b2 to lowercase and check that they match, but exclude
            // "lowercase" underscore (DEL). Then test special cases.
            let (b1, b2) = (canon[i] | 0x20, other[j]);
            i += 1;
            if (b1 == b2 | 0x20) & (b2 != b'_' | 0x20) {
                j += 1;
            } else if b1 == b'i' && other[j..].starts_with("İ".as_bytes()) {
                j += "İ".len();
            } else if b1 == b'k' && other[j..].starts_with("K".as_bytes()) {
                j += "K".len();
            } else {
                return false;
            }
        }
        true
    }
}

impl Debug for &CanonStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl Display for &CanonStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use crate::mnemonics::CanonStr;

    #[test]
    fn utf8_folding() {
        assert!(CanonStr::new("debug_printStack").eq_lower("Debug_PrİntStacK"));
    }
}
