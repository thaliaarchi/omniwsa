//! Converting between integer syntaxes.

use std::{iter, mem, slice};

pub use rug::Integer;

use crate::tokens::integer::{
    Base, BaseStyle, DigitSep, IntegerSyntax, IntegerToken, Sign, SignStyle,
};

// TODO:
// - Reserve size according to the size required by the new base.
// - Manipulate leading zeros according to base.
// - Maintain digit separators when possible.
// - Make in-place version, which reuses `self.value`.
// - When removing Haskell parentheses, keep the spaces, either in the token if
//   allowed by the dialect or moved out as space tokens.

impl IntegerToken<'_> {
    /// Returns whether this integer literal is already compatible with the
    /// syntax to be converted to and does not need to be changed.
    pub fn compatible_syntax(&self, from: &IntegerSyntax, to: &IntegerSyntax) -> bool {
        if !to.bases.contains(self.base) {
            return false;
        }
        if from.base_style != to.base_style && self.base != Base::Decimal {
            if !matches!(
                (self.base, from.base_style, to.base_style),
                (
                    Base::Hexadecimal | Base::Binary,
                    BaseStyle::C | BaseStyle::Rust,
                    BaseStyle::C | BaseStyle::Rust,
                )
            ) {
                return false;
            }
        }
        if from.sign_style != to.sign_style
            && (self.sign == Sign::Pos && to.sign_style != SignStyle::NegPos
                || from.sign_style == SignStyle::Haskell)
        {
            return false;
        }
        if self.has_digit_seps && to.digit_sep == DigitSep::None {
            return false;
        }
        true
    }

    /// Converts the syntax of this integer token to another style. The value is
    /// unchanged.
    ///
    /// If `self.literal` is not consistent with what was parsed to the other
    /// fields, the result may be inconsistent.
    pub fn convert_syntax(&self, from: &IntegerSyntax, to: &IntegerSyntax) -> Self {
        // Both Haskell parentheses and Palaiologos bases use suffixes and would
        // need a refactor to combine.
        debug_assert!(
            !(from.sign_style == SignStyle::Haskell && from.base_style == BaseStyle::Palaiologos
                || to.sign_style == SignStyle::Haskell && to.base_style == BaseStyle::Palaiologos)
        );

        let mut new_literal = Vec::with_capacity(self.literal.len());
        let mut suffix = &b""[..];

        // Convert the sign.
        let drop_sign = self.sign == Sign::Pos && to.sign_style != SignStyle::NegPos;
        let new_sign = if drop_sign { Sign::None } else { self.sign };
        let (parsed_sign, sign_bytes, s) = match from.sign_style {
            SignStyle::Neg | SignStyle::NegPos => {
                let (sign, s) = Sign::strip(&self.literal);
                let (sign_bytes, _) = slice_subtract(&self.literal, s);
                (sign, sign_bytes, s)
            }
            SignStyle::Haskell => {
                let (sign, s, _) = Sign::strip_haskell(&self.literal);
                let sign_bytes = if to.sign_style == SignStyle::Haskell {
                    let (before, after) = slice_subtract(&self.literal, s);
                    suffix = after;
                    before
                } else {
                    new_sign.as_str().as_bytes()
                };
                (sign, sign_bytes, s)
            }
        };
        debug_assert_eq!(parsed_sign, self.sign);
        if !drop_sign {
            new_literal.extend_from_slice(sign_bytes);
        }

        // Convert the base.
        let new_base = if !to.bases.contains(self.base) && !to.bases.is_empty() {
            if to.bases.contains(Base::Hexadecimal) {
                Base::Hexadecimal
            } else if to.bases.contains(Base::Decimal) {
                Base::Decimal
            } else if to.bases.contains(Base::Binary) {
                Base::Binary
            } else {
                Base::Octal
            }
        } else {
            self.base
        };
        let (parsed_base, s) = match from.base_style {
            BaseStyle::C => {
                let (base, s2) = Base::strip_c(s);
                let (base_bytes, _) = slice_subtract(s, s2);
                match to.base_style {
                    BaseStyle::Rust if self.base == Base::Octal => {
                        new_literal.extend_from_slice(b"0o");
                    }
                    BaseStyle::C | BaseStyle::Rust => new_literal.extend_from_slice(base_bytes),
                    BaseStyle::Palaiologos => {
                        suffix = match base_bytes {
                            [_, b'x'] => b"h",
                            [_, b'X'] => b"H",
                            [_, _] => &base_bytes[1..2],
                            [_] => b"o",
                            _ => b"",
                        };
                    }
                }
                (base, s2)
            }
            BaseStyle::Rust => {
                let (base, s2) = Base::strip_rust(s);
                let (base_bytes, _) = slice_subtract(s, s2);
                match to.base_style {
                    BaseStyle::C if self.base == Base::Octal => new_literal.push(b'0'),
                    BaseStyle::C | BaseStyle::Rust => new_literal.extend_from_slice(base_bytes),
                    BaseStyle::Palaiologos => {
                        suffix = match base_bytes {
                            [_, b'x'] => b"h",
                            [_, b'X'] => b"H",
                            [_, _] => &base_bytes[1..2],
                            _ => b"",
                        };
                    }
                }
                (base, s2)
            }
            BaseStyle::Palaiologos => {
                let (base, s2) = Base::strip_palaiologos(s);
                let (_, base_bytes) = slice_subtract(s, s2);
                if to.base_style == BaseStyle::Palaiologos {
                    suffix = base_bytes;
                } else {
                    let base_bytes: &[u8] = match base_bytes {
                        [b'h'] => b"0x",
                        [b'H'] => b"0X",
                        [b'b'] => b"0b",
                        [b'B'] => b"0B",
                        [b'o'] if to.base_style == BaseStyle::Rust => b"0o",
                        [b'O'] if to.base_style == BaseStyle::Rust => b"0O",
                        [b'o' | b'O'] if to.base_style == BaseStyle::C => b"0",
                        _ => b"",
                    };
                    new_literal.extend_from_slice(base_bytes);
                }
                (base, s2)
            }
        };
        debug_assert_eq!(parsed_base, self.base);

        // Handle the obligatory leading zero when the number starts with a hex
        // digit.
        let mut new_leading_zeros = self.leading_zeros;
        if to.base_style == BaseStyle::C && new_base == Base::Decimal {
            debug_assert!(s.len() >= self.leading_zeros);
            new_leading_zeros = 0;
        } else if from.base_style == BaseStyle::Palaiologos
            && to.base_style != BaseStyle::Palaiologos
            && self.base == Base::Hexadecimal
            && self.leading_zeros == 1
            && most_significant_hex_digit(&self.value) >= 0x0a
        {
            debug_assert!(!s.is_empty());
            new_leading_zeros = 0;
        } else if from.base_style != BaseStyle::Palaiologos
            && to.base_style == BaseStyle::Palaiologos
            && new_base == Base::Hexadecimal
            && self.leading_zeros == 0
            && most_significant_hex_digit(&self.value) >= 0x0a
        {
            new_leading_zeros = 1;
        }

        // Append the leading zeros and converted digits.
        let mut new_has_digit_seps = self.has_digit_seps;
        if self.base == new_base && !(self.has_digit_seps && to.digit_sep == DigitSep::None) {
            let mut s = s;
            if new_leading_zeros > self.leading_zeros {
                new_literal.extend(iter::repeat(b'0').take(new_leading_zeros - self.leading_zeros));
            } else if self.leading_zeros < new_leading_zeros {
                s = &s[1..];
            }
            new_literal.extend_from_slice(s);
        } else {
            let digits = self.value.to_string_radix(new_base as _);
            new_literal.extend(iter::repeat(b'0').take(new_leading_zeros));
            new_literal.extend_from_slice(digits.as_bytes());
            new_has_digit_seps = false;
        }

        // Append the Haskell close parentheses or Palaiologos base suffix.
        new_literal.extend_from_slice(suffix);

        IntegerToken {
            literal: new_literal.into(),
            value: self.value.clone(),
            sign: new_sign,
            base: new_base,
            leading_zeros: new_leading_zeros,
            has_digit_seps: new_has_digit_seps,
            errors: self.errors,
        }
    }
}

/// Subtracts `b` from `a`. `b` must be a sub-slice of `a`.
#[inline]
fn slice_subtract<'a, T>(a: &'a [T], b: &'a [T]) -> (&'a [T], &'a [T]) {
    debug_assert!(a.as_ptr() <= b.as_ptr() && b.as_ptr_range().end <= a.as_ptr_range().end);
    // SAFETY: The caller guarantees that `b` is a sub-slice of `a`.
    unsafe {
        let start = b.as_ptr().offset_from(a.as_ptr()) as usize;
        (&a[..start], &a[start + b.len()..])
    }
}

/// Gets the most-significant byte from the integer.
fn most_significant_byte(int: &Integer) -> u8 {
    // Avoid dependency on `gmp_mpfr_sys::gmp::limb_t`.
    #[inline]
    unsafe fn as_bytes<T>(s: &[T]) -> &[u8] {
        slice::from_raw_parts(s.as_ptr() as *const u8, s.len() * mem::size_of::<T>())
    }
    // SAFETY: The pointer and length are valid and the alignment of u8 is less
    // than u64.
    let bytes = unsafe { as_bytes(int.as_limbs()) };
    bytes.iter().find(|&&b| b != 0).copied().unwrap_or(0)
}

/// Gets the most-significant hex digit from the integer.
fn most_significant_hex_digit(int: &Integer) -> u8 {
    let msb = most_significant_byte(int);
    if msb > 0x0f {
        msb >> 4
    } else {
        msb
    }
}
