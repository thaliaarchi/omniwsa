//! Converting between integer syntaxes.

use std::{iter, mem, slice};

pub use rug::Integer;

use crate::tokens::integer::{BaseStyle, DigitSep, IntegerSyntax, IntegerToken, Sign, SignStyle};

// TODO:
// - Reserve size according to the size required for the digits in the new base.
// - Manipulate leading zeros according to base.
// - Maintain digit separators when possible.
// - Make in-place version, which reuses `self.value`.
// - When removing Haskell parentheses, keep the spaces, either in the token if
//   allowed by the dialect or moved out as space tokens.
// - Generalize suffix handling in convert_syntax to support both Haskell parens
//   and Palaiologos bases.

impl IntegerToken<'_> {
    /// Returns whether this integer literal is already compatible with the
    /// syntax to be converted to and does not need to be changed.
    pub fn compatible_syntax(&self, from: &IntegerSyntax, to: &IntegerSyntax) -> bool {
        if !to.base_styles.contains(self.base_style) {
            return false;
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
            !(from.sign_style == SignStyle::Haskell
                && !BaseStyle::suffix_family().contains(self.base_style)
                || to.sign_style == SignStyle::Haskell
                    && !to.base_styles.is_disjoint(BaseStyle::suffix_family()))
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
        let new_base_style = self.base_style.to_nearest(to.base_styles);
        debug_assert!(s.starts_with(self.base_style.prefix().as_bytes()));
        debug_assert!(s.ends_with(self.base_style.suffix().as_bytes()));
        let s = &s[self.base_style.prefix().len()..s.len() - self.base_style.suffix().len()];

        debug_assert!(s[..self.leading_zeros].iter().all(|&b| b == b'0'));

        let mut new_leading_zeros = self.leading_zeros;
        if new_base_style != self.base_style {
            if to.base_styles.contains(BaseStyle::OctPrefix_0)
                && new_base_style == BaseStyle::Decimal
            {
                new_leading_zeros = 0;
            } else {
                // Handle the obligatory leading zero for hex suffixes when the
                // number starts with a hex digit.
                let from_hex_suffix = self.base_style == BaseStyle::HexSuffix_h
                    || self.base_style == BaseStyle::HexSuffix_H;
                let to_hex_suffix = new_base_style == BaseStyle::HexSuffix_h
                    || new_base_style == BaseStyle::HexSuffix_H;
                if from_hex_suffix
                    && !to_hex_suffix
                    && self.leading_zeros == 1
                    && most_significant_hex_digit(&self.value) >= 0xa
                {
                    new_leading_zeros = 0;
                } else if !from_hex_suffix
                    && to_hex_suffix
                    && self.leading_zeros == 0
                    && most_significant_hex_digit(&self.value) >= 0xa
                {
                    new_leading_zeros = 1;
                }
            }
        }

        // Append the leading zeros and converted digits.
        let mut new_has_digit_seps = self.has_digit_seps;
        let new_base = new_base_style.base();
        if self.base_style.base() == new_base
            && !(self.has_digit_seps && to.digit_sep == DigitSep::None)
        {
            let mut s = s;
            if new_leading_zeros > self.leading_zeros {
                new_literal.extend(iter::repeat(b'0').take(new_leading_zeros - self.leading_zeros));
            } else if new_leading_zeros < self.leading_zeros {
                s = &s[self.leading_zeros - new_leading_zeros..];
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
            base_style: new_base_style,
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
