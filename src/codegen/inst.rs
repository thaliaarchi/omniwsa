//! Whitespace instructions for code generation.

use rug::Integer;

use crate::tokens::integer::Sign;

// TODO:
// - Distinction between Haskell Integer for `push` and Int for `copy` and
//   `slide`. Perhaps issue a warning when the index would wrap.

/// A Whitespace instruction for code generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inst<'a> {
    /// Whitespace `push` (SS).
    Push(IntegerBits<'a>),
    /// Whitespace `dup` (SLS).
    Dup,
    /// Whitespace `copy` (STS).
    Copy(IntegerBits<'a>),
    /// Whitespace `swap` (SLT).
    Swap,
    /// Whitespace `drop` (SLL).
    Drop,
    /// Whitespace `slide` (STL).
    Slide(IntegerBits<'a>),
    /// Whitespace `add` (TSSS).
    Add,
    /// Whitespace `sub` (TSST).
    Sub,
    /// Whitespace `mul` (TSSL).
    Mul,
    /// Whitespace `div` (TSTS).
    Div,
    /// Whitespace `mod` (TSTT).
    Mod,
    /// Whitespace `store` (TTS).
    Store,
    /// Whitespace `retrieve` (TTT).
    Retrieve,
    /// Whitespace `label` (LSS).
    Label(LabelBits<'a>),
    /// Whitespace `call` (LST).
    Call(LabelBits<'a>),
    /// Whitespace `jmp` (LSL).
    Jmp(LabelBits<'a>),
    /// Whitespace `jz` (LTS).
    Jz(LabelBits<'a>),
    /// Whitespace `jn` (LTT).
    Jn(LabelBits<'a>),
    /// Whitespace `ret` (LTL).
    Ret,
    /// Whitespace `end` (LLL).
    End,
    /// Whitespace `printc` (TLSS).
    Printc,
    /// Whitespace `printi` (TLST).
    Printi,
    /// Whitespace `readc` (TLTS).
    Readc,
    /// Whitespace `readi` (TLTT).
    Readi,

    /// Burghard `debug_printstack` (LLSSS).
    BurghardPrintStack,
    /// Burghard `debug_printheap` (LLSST).
    BurghardPrintHeap,
    /// voliva `or` (TSLS).
    VolivaOr,
    /// voliva `not` (TSLT).
    VolivaNot,
    /// voliva `and` (TSLL).
    VolivaAnd,
    /// voliva `dbg` (LLS).
    VolivaBreakpoint,
}

/// A signed integer value for code generation, encoded with explicit leading
/// zeros.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerBits<'a>(pub(super) ArgBits<'a>);

/// An unsigned label value for code generation, encoded with explicit leading
/// zeros.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelBits<'a>(pub(super) ArgBits<'a>);

/// An integer or label for code generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ArgBits<'a> {
    pub(super) value: &'a Integer,
    pub(super) sign: Sign,
    pub(super) leading_zeros: usize,
}

impl<'a> IntegerBits<'a> {
    /// Creates a signed integer, encoded with a number of leading zeros.
    #[inline]
    pub const fn new(value: &'a Integer, leading_zeros: usize) -> Self {
        IntegerBits(ArgBits {
            value,
            sign: if value.is_negative() {
                Sign::Neg
            } else {
                Sign::Pos
            },
            leading_zeros,
        })
    }

    /// Creates a zero integer, encoded with a sign and no bits.
    #[inline]
    pub fn zero(sign: Sign) -> Self {
        static ZERO: Integer = Integer::ZERO;
        IntegerBits(ArgBits {
            value: &ZERO,
            sign,
            leading_zeros: 0,
        })
    }

    /// Gets the value of this integer.
    #[inline]
    pub const fn value(&self) -> &'a Integer {
        self.0.value
    }

    /// Gets the sign of this integer.
    #[inline]
    pub const fn sign(&self) -> Sign {
        self.0.sign
    }

    /// Gets the number of leading zeros this integer has.
    #[inline]
    pub const fn leading_zeros(&self) -> usize {
        self.0.leading_zeros
    }
}

/// Creates a signed integer, encoded with no leading zeros.
impl<'a> From<&'a Integer> for IntegerBits<'a> {
    #[inline]
    fn from(value: &'a Integer) -> Self {
        IntegerBits::new(value, 0)
    }
}

impl<'a> LabelBits<'a> {
    /// Creates an unsigned integer, encoded with no sign and a number of
    /// leading zeros.
    #[inline]
    pub fn new(value: &'a Integer, leading_zeros: usize) -> Self {
        debug_assert!(!value.is_negative());
        LabelBits(ArgBits {
            value,
            sign: Sign::None,
            leading_zeros,
        })
    }

    /// Gets the value of this label.
    #[inline]
    pub const fn value(&self) -> &'a Integer {
        self.0.value
    }

    /// Gets the number of leading zeros this label has.
    #[inline]
    pub const fn leading_zeros(&self) -> usize {
        self.0.leading_zeros
    }
}

/// Creates an unsigned integer, encoded with no sign and no leading zeros.
impl<'a> From<&'a Integer> for LabelBits<'a> {
    #[inline]
    fn from(value: &'a Integer) -> Self {
        LabelBits::new(value, 0)
    }
}
