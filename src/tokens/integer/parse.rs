//! Integer literal parsing.

use std::borrow::Cow;

use crate::tokens::integer::{
    Base, BaseStyle, DigitSep, IntegerError, IntegerSyntax, IntegerToken, Sign, SignStyle,
};

impl IntegerSyntax {
    /// Parses an integer with the described syntax, using a scratch buffer of
    /// digits to reuse allocations.
    pub fn parse<'s>(&self, literal: Cow<'s, [u8]>, digits: &mut Vec<u8>) -> IntegerToken<'s> {
        let mut int = IntegerToken::default();
        let (sign, s) = match self.sign_style {
            SignStyle::Neg | SignStyle::NegPos => {
                let (sign, s) = Sign::strip(&literal);
                if int.sign == Sign::Pos && self.sign_style == SignStyle::NegPos {
                    int.errors |= IntegerError::InvalidSign;
                }
                (sign, s)
            }
            SignStyle::Haskell => {
                let (sign, s, errors) = Sign::strip_haskell(&literal);
                int.errors |= errors;
                (sign, s)
            }
        };
        int.sign = sign;
        let (base, s) = match self.base_style {
            BaseStyle::C => Base::strip_c(s),
            BaseStyle::Rust => Base::strip_rust(s),
            BaseStyle::Palaiologos => {
                let (base, s) = Base::strip_palaiologos(s);
                if base == Base::Hexadecimal
                    && s.first()
                        .is_some_and(|b| matches!(b, b'a'..=b'f' | b'A'..=b'F'))
                {
                    int.errors |= IntegerError::StartsWithHex;
                }
                (base, s)
            }
        };
        int.base = base;
        if !self.bases.contains(base) {
            int.errors |= IntegerError::InvalidBase;
        }
        int.parse_digits(s, digits);
        int.literal = literal;
        if int.has_digit_seps && self.digit_sep == DigitSep::None {
            int.errors |= IntegerError::InvalidDigitSep;
        }
        if self.min_value.as_ref().is_some_and(|min| min > &int.value)
            || self.max_value.as_ref().is_some_and(|max| max < &int.value)
        {
            int.errors |= IntegerError::Range;
        }
        int
    }
}

impl IntegerToken<'_> {
    /// Parses the byte string as digits in the given base with optional `_`
    /// digit separators.
    fn parse_digits(&mut self, s: &[u8], digits: &mut Vec<u8>) {
        digits.clear();

        self.leading_zeros = s.iter().take_while(|&&ch| ch == b'0').count();
        let s = &s[self.leading_zeros..];

        if !s.is_empty() {
            digits.reserve(s.len());
            match self.base {
                Base::Decimal => {
                    for &b in s {
                        let digit = b.wrapping_sub(b'0');
                        if digit >= 10 {
                            if digit == b'_' - b'0' {
                                self.has_digit_seps = true;
                                continue;
                            }
                            self.errors |= IntegerError::InvalidDigit;
                            break;
                        }
                        digits.push(digit);
                    }
                }
                Base::Hexadecimal => {
                    for &b in s {
                        let digit = match b {
                            b'0'..=b'9' => b - b'0',
                            b'a'..=b'f' => b - b'a' + 10,
                            b'A'..=b'F' => b - b'A' + 10,
                            _ => {
                                if b == b'_' {
                                    self.has_digit_seps = true;
                                    continue;
                                }
                                self.errors |= IntegerError::InvalidDigit;
                                break;
                            }
                        };
                        digits.push(digit);
                    }
                }
                Base::Octal => {
                    for &b in s {
                        let digit = b.wrapping_sub(b'0');
                        if digit >= 8 {
                            if digit == b'_' - b'0' {
                                self.has_digit_seps = true;
                                continue;
                            }
                            self.errors |= IntegerError::InvalidDigit;
                            break;
                        }
                        digits.push(digit);
                    }
                }
                Base::Binary => {
                    for &b in s {
                        let digit = b.wrapping_sub(b'0');
                        if digit >= 2 {
                            if digit == b'_' - b'0' {
                                self.has_digit_seps = true;
                                continue;
                            }
                            self.errors |= IntegerError::InvalidDigit;
                            break;
                        }
                        digits.push(digit);
                    }
                }
            }
            // SAFETY: Digits are constructed to be in range for the base.
            unsafe {
                self.value.assign_bytes_radix_unchecked(
                    digits,
                    self.base as i32,
                    self.sign == Sign::Neg,
                );
            }
        } else if self.leading_zeros == 0 {
            self.errors |= IntegerError::NoDigits;
        }
    }
}

impl Sign {
    /// Strips an optional sign from an integer literal.
    pub(super) fn strip(s: &[u8]) -> (Self, &[u8]) {
        match s.split_first() {
            Some((b'-', s)) => (Sign::Neg, s),
            Some((b'+', s)) => (Sign::Pos, s),
            _ => (Sign::None, s),
        }
    }
}
impl Base {
    /// Strips a base prefix from an integer literal with C-like syntax,
    /// specifically a prefix of `0x`/`0X` for hexadecimal, `0b`/`0B` for
    /// binary, `0` for octal, and otherwise for decimal.
    #[inline]
    pub(super) fn strip_c(s: &[u8]) -> (Self, &[u8]) {
        match s {
            [b'0', b'x' | b'X', s @ ..] => (Base::Hexadecimal, s),
            [b'0', b'b' | b'B', s @ ..] => (Base::Binary, s),
            [b'0', s @ ..] => (Base::Octal, s),
            s => (Base::Decimal, s),
        }
    }

    /// Strips a base prefix from an integer literal with Rust-like syntax,
    /// specifically a prefix of `0x`/`0X` for hexadecimal, `0b`/`0B` for
    /// binary, `0o`/`0O` for octal, and otherwise for decimal.
    #[inline]
    pub(super) fn strip_rust(s: &[u8]) -> (Self, &[u8]) {
        match s {
            [b'0', b'x' | b'X', s @ ..] => (Base::Hexadecimal, s),
            [b'0', b'b' | b'B', s @ ..] => (Base::Binary, s),
            [b'0', b'o' | b'O', s @ ..] => (Base::Octal, s),
            s => (Base::Decimal, s),
        }
    }

    /// Strips a base suffix from an integer literal with Palaiologos-like
    /// syntax, specifically a suffix of `h`/`H` for hexadecimal, `b`/`B` for
    /// binary, `o`/`O` for octal, and otherwise for decimal.
    #[inline]
    pub(super) fn strip_palaiologos(s: &[u8]) -> (Self, &[u8]) {
        match s.split_last() {
            Some((b'h' | b'H', s)) => (Base::Hexadecimal, s),
            Some((b'b' | b'B', s)) => (Base::Binary, s),
            Some((b'o' | b'O', s)) => (Base::Octal, s),
            _ => (Base::Decimal, s),
        }
    }
}
