//! Integer literal parsing.

use std::borrow::Cow;

use enumset::EnumSet;

use crate::tokens::integer::{
    Base, BaseStyle, DigitSep, Integer, IntegerError, IntegerSyntax, IntegerToken, Sign, SignStyle,
};

// TODO:
// - Move integer scanning here.
// - Extend Palaiologos syntax with `x`/`X` suffix. It conflicts with `xchg`, so
//   it should peek before bumping.
// - When an integer that ends with `b`/`B` is valid as binary, interpret it as
//   a suffix. Otherwise, unless it is supported, always treat it as decimal.

impl IntegerSyntax {
    /// Parses an integer with the described syntax, using a scratch buffer of
    /// digits to reuse allocations.
    pub fn parse<'s>(&self, literal: Cow<'s, [u8]>, digits: &mut Vec<u8>) -> IntegerToken<'s> {
        let mut int = IntegerToken {
            literal: b""[..].into(),
            value: Integer::new(),
            sign: Sign::None,
            base_style: BaseStyle::Decimal,
            leading_zeros: 0,
            has_digit_seps: false,
            errors: EnumSet::empty(),
        };
        let (sign, s) = match self.sign_style {
            SignStyle::Neg | SignStyle::NegPos => {
                let (sign, s) = Sign::strip(&literal);
                if sign == Sign::Pos && self.sign_style == SignStyle::NegPos {
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
        let (base_style, s) = if (self.base_styles - BaseStyle::Decimal)
            .is_subset(BaseStyle::prefix_family())
        {
            BaseStyle::strip_prefix(s, self.base_styles.contains(BaseStyle::OctPrefix_0))
        } else if (self.base_styles - BaseStyle::Decimal).is_subset(BaseStyle::suffix_family()) {
            let (base_style, s) = BaseStyle::strip_suffix(s);
            if base_style.base() == Base::Hexadecimal
                && s.first()
                    .is_some_and(|b| matches!(b, b'a'..=b'f' | b'A'..=b'F'))
            {
                int.errors |= IntegerError::StartsWithHex;
            }
            (base_style, s)
        } else {
            panic!("both prefix and suffix base styles");
        };
        int.base_style = base_style;
        if !self.base_styles.contains(base_style) {
            int.errors |= IntegerError::InvalidBase;
        }
        int.parse_digits(s, digits);
        int.literal = literal;
        if digits.is_empty() && int.leading_zeros == 0 {
            if !int.errors.contains(IntegerError::InvalidDigit) {
                int.errors |= IntegerError::NoDigits;
            } else if int.base_style == BaseStyle::OctPrefix_0 {
                int.base_style = BaseStyle::Decimal;
            }
        }
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
        if s.is_empty() {
            return;
        }

        digits.reserve(s.len());
        let base = self.base_style.base();
        match base {
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
            self.value
                .assign_bytes_radix_unchecked(digits, base as i32, self.sign == Sign::Neg);
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

impl BaseStyle {
    /// Strips a base prefix from an integer literal with C-like syntax,
    /// specifically a prefix of `0x`/`0X` for hexadecimal, `0b`/`0B` for
    /// binary, `0o`/`0O` and, if enabled, `0` for octal, and otherwise for
    /// decimal.
    #[inline]
    pub(super) fn strip_prefix(s: &[u8], octal_0: bool) -> (Self, &[u8]) {
        match s {
            [b'0', b'b', s @ ..] => (BaseStyle::BinPrefix_0b, s),
            [b'0', b'B', s @ ..] => (BaseStyle::BinPrefix_0B, s),
            [b'0', b'o', s @ ..] => (BaseStyle::OctPrefix_0o, s),
            [b'0', b'O', s @ ..] => (BaseStyle::OctPrefix_0O, s),
            [b'0', b'x', s @ ..] => (BaseStyle::HexPrefix_0x, s),
            [b'0', b'X', s @ ..] => (BaseStyle::HexPrefix_0X, s),
            [b'0', s @ ..] if octal_0 => (BaseStyle::OctPrefix_0, s),
            _ => (BaseStyle::Decimal, s),
        }
    }

    /// Strips a base suffix from an integer literal with Palaiologos-like
    /// syntax, specifically a suffix of `h`/`H` for hexadecimal, `b`/`B` for
    /// binary, `o`/`O` for octal, and otherwise for decimal.
    #[inline]
    pub(super) fn strip_suffix(s: &[u8]) -> (Self, &[u8]) {
        match s.split_last() {
            Some((b'b', s)) => (BaseStyle::BinSuffix_b, s),
            Some((b'B', s)) => (BaseStyle::BinSuffix_B, s),
            Some((b'o', s)) => (BaseStyle::OctSuffix_o, s),
            Some((b'O', s)) => (BaseStyle::OctSuffix_O, s),
            Some((b'h', s)) => (BaseStyle::HexSuffix_h, s),
            Some((b'H', s)) => (BaseStyle::HexSuffix_H, s),
            _ => (BaseStyle::Decimal, s),
        }
    }
}
