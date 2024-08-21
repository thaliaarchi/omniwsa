//! Integer literal parsing.

use std::{borrow::Cow, str};

use crate::tokens::integer::{
    IntegerBase, IntegerDigitSep, IntegerError, IntegerSign, IntegerStyle, IntegerSyntax,
    IntegerToken,
};

impl IntegerSyntax {
    /// Parses an integer with the described syntax, using a scratch buffer of
    /// digits to reuse allocations.
    pub fn parse<'s>(&self, literal: Cow<'s, [u8]>, digits: &mut Vec<u8>) -> IntegerToken<'s> {
        let mut int = IntegerToken::default();
        let s = match self.style {
            IntegerStyle::Haskell => {
                debug_assert!(!self.explicit_pos);
                let s = str::from_utf8(&literal).unwrap();
                let (sign, s, sign_errors) = Self::strip_haskell_sign(s);
                let (base, s) = Self::strip_base_rust(s.as_bytes());
                int.sign = sign;
                int.base = base;
                int.errors |= sign_errors;
                s
            }
            IntegerStyle::Palaiologos => {
                let (sign, s) = Self::strip_sign(&literal);
                let (base, s) = Self::strip_base_palaiologos(s);
                if base == IntegerBase::Hexadecimal
                    && s.first()
                        .is_some_and(|b| matches!(b, b'a'..=b'f' | b'A'..=b'F'))
                {
                    int.errors |= IntegerError::StartsWithHex;
                }
                int.sign = sign;
                int.base = base;
                s
            }
        };
        int.parse_digits(s, digits);
        int.literal = literal;
        if int.sign == IntegerSign::Pos && !self.explicit_pos {
            int.errors |= IntegerError::InvalidSign;
        }
        if !self.bases.contains(int.base) {
            int.errors |= IntegerError::InvalidBase;
        }
        if int.has_digit_seps && self.digit_sep == IntegerDigitSep::None {
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
                IntegerBase::Decimal => {
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
                IntegerBase::Hexadecimal => {
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
                IntegerBase::Octal => {
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
                IntegerBase::Binary => {
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
                    self.sign == IntegerSign::Neg,
                );
            }
        } else if self.leading_zeros == 0 {
            self.errors |= IntegerError::NoDigits;
        }
    }
}

impl IntegerSyntax {
    /// Strips an optional sign from an integer literal.
    fn strip_sign(s: &[u8]) -> (IntegerSign, &[u8]) {
        match s.split_first() {
            Some((b'-', s)) => (IntegerSign::Neg, s),
            Some((b'+', s)) => (IntegerSign::Pos, s),
            _ => (IntegerSign::None, s),
        }
    }

    /// Strips a base prefix from an integer literal with C-like syntax,
    /// specifically a prefix of `0x`/`0X` for hexadecimal, `0b`/`0B` for
    /// binary, `0` for octal, and otherwise for decimal.
    #[allow(dead_code)]
    fn strip_base_c(s: &[u8]) -> (IntegerBase, &[u8]) {
        match s {
            [b'0', b'x' | b'X', s @ ..] => (IntegerBase::Hexadecimal, s),
            [b'0', b'b' | b'B', s @ ..] => (IntegerBase::Binary, s),
            [b'0', s @ ..] => (IntegerBase::Octal, s),
            s => (IntegerBase::Decimal, s),
        }
    }

    /// Strips a base prefix from an integer literal with Rust-like syntax,
    /// specifically a prefix of `0x`/`0X` for hexadecimal, `0o`/`0O` for octal,
    /// `0b`/`0B` for binary, and otherwise for decimal.
    fn strip_base_rust(s: &[u8]) -> (IntegerBase, &[u8]) {
        match s {
            [b'0', b'x' | b'X', s @ ..] => (IntegerBase::Hexadecimal, s),
            [b'0', b'o' | b'O', s @ ..] => (IntegerBase::Octal, s),
            [b'0', b'b' | b'B', s @ ..] => (IntegerBase::Binary, s),
            s => (IntegerBase::Decimal, s),
        }
    }

    /// Strips a base suffix from an integer literal with Palaiologos-like
    /// syntax, specifically a suffix of `h`/`H` for hexadecimal, `b`/`B` for
    /// binary, `o`/`O` for octal, and otherwise for decimal.
    fn strip_base_palaiologos(s: &[u8]) -> (IntegerBase, &[u8]) {
        match s.split_last() {
            Some((b'h' | b'H', s)) => (IntegerBase::Hexadecimal, s),
            Some((b'b' | b'B', s)) => (IntegerBase::Binary, s),
            Some((b'o' | b'O', s)) => (IntegerBase::Octal, s),
            _ => (IntegerBase::Decimal, s),
        }
    }
}
