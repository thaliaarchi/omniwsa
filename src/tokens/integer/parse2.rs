//! Parsing for integer literals with configurable syntax.

use enumset::{EnumSet, EnumSetType};
use rug::Integer;

use crate::{lex::Scanner, tokens::integer::BaseStyle};

// TODO:
// - Parse parens as `GroupToken`.
// - Implement remainder of parsing, particularly digits.
// - Handle digit separator position errors.
// - Handle `0` octal prefix and `0b`/`0B` prefix as digits for `h`/`H` suffix.
//   These have consequences for digit separator position errors and
//   `suffix_decimal_first`.
// - Make set of allowed whitespace characters configurable with enum, but parse
//   the union of them anyways.
// - Make a stripped down scanner for parsing integers which maintains less
//   state.

/// An integer literal token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegerToken {
    /// The value this literal represents.
    pub value: Integer,
    /// Whether the integer has a negative sign.
    pub negative: bool,
    /// Lexical style of the sign.
    pub sign_style: SignStyle,
    /// Lexical style of the base.
    pub base_style: BaseStyle,
    /// Digit separators used.
    pub digit_seps: EnumSet<DigitSep>,
    /// Parse errors.
    pub errors: EnumSet<IntegerError>,
}

/// A parse error for an integer literal.
#[derive(EnumSetType, Debug)]
pub enum IntegerError {
    /// Unsupported sign style.
    SignStyle,
    /// Unsupported base style.
    BaseStyle,
    /// No digits, excluding a base prefix.
    NoDigits,
    /// An invalid digit.
    InvalidDigit,
    /// Starts with a hex letter (Palaiologos).
    StartsWithHex,
    /// Unsupported digit separators.
    DigitSep,
    /// Digit separator at an unsupported location.
    DigitSepLocation,
    /// Whitespace at an unsupported location.
    SpaceLocation,
    /// An unpaired parenthesis (Haskell).
    UnpairedParen,
    /// Value out of range.
    Range,
}

/// A description of supported syntax features for integer literals, used for
/// parsing and converting between syntaxes.
///
/// These options control which features are valid syntax, not which are parsed;
/// all combinations are still parsed. See [`ParseConfig`] for configuring parse
/// behavior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Syntax {
    /// The supported signs.
    pub signs: EnumSet<SignStyle>,
    /// The supported base styles.
    pub base_styles: EnumSet<BaseStyle>,
    /// Whether suffix base styles require the first digit to be decimal.
    pub suffix_decimal_first: bool,
    /// The supported digit separators.
    pub digit_seps: EnumSet<DigitSep>,
    /// Locations at which digit separators are supported.
    pub digit_sep_locations: EnumSet<DigitSepLocation>,
    /// Locations at which whitespace characters are supported.
    pub space_locations: EnumSet<SpaceLocation>,
    /// Whether grouping parentheses are supported around an integer.
    pub parens: bool,
    /// The minimum supported value.
    pub min_value: Option<Integer>,
    /// The maximum supported value.
    pub max_value: Option<Integer>,
}

/// Configures parser behavior for integer literals.
///
/// See [`Syntax`] for configuring which features are valid syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseConfig {
    /// Whether a leading zero `0` denotes octal. Usually, it is paired with
    /// `BaseStyle::OctPrefix_0` in `Syntax::base_styles`; when it isn't,
    /// leading zeros produce an error.
    pub leading_zero_octal: bool,
    /// Enable parsing single quote `'` digit separators. This can conflict with
    /// character literal syntax.
    pub quote_digit_sep: bool,
    /// Enable juxtaposing a word after an integer without spaces between.
    pub juxtapose_word: bool,
    /// Whether to parse grouping parentheses around an integer. This rarely
    /// makes sense and conflicts with other syntax.
    pub parens: bool,
}

/// Lexical representation of an integer sign.
#[derive(Debug, EnumSetType)]
pub enum SignStyle {
    /// Implicit positive sign.
    ImplicitPos,
    /// Explicit `-` negative sign.
    Neg,
    /// Explicit `+` positive sign.
    Pos,
    /// Any combination of two or more signs.
    Multiple,
}

/// Style of integer digit separator.
#[derive(Debug, EnumSetType)]
pub enum DigitSep {
    /// Underscore `_` digit separators.
    Underscore,
    /// Single quote `'` digit separators.
    SingleQuote,
}

/// A location at which digit separators may appear in an integer literal.
#[derive(Debug, EnumSetType)]
pub enum DigitSepLocation {
    /// After a base prefix.
    AfterBasePrefix,
    /// After a leading `0` octal base prefix.
    AfterOctalLeadingZero,
    /// After the last digit, but before a base suffix.
    AfterDigits,
    /// Multiple adjacent digit separators.
    MultipleAdjacent,
}

/// A location at which whitespace characters may appear in an integer literal.
#[derive(Debug, EnumSetType)]
pub enum SpaceLocation {
    /// Before the integer.
    Leading,
    /// Between the sign and digits.
    AfterSign,
    /// After the integer.
    Trailing,
    /// Between parentheses and the integer.
    BetweenParens,
}

/// A parser for integer literals with configurable syntax.
#[derive(Debug)]
struct IntegerParser<'s, 'a> {
    scan: &'a mut Scanner<'s>,
    syntax: &'a Syntax,
    cfg: ParseConfig,
    errors: EnumSet<IntegerError>,
    digit_buf: &'a mut Vec<u8>,
}

impl<'s, 'a> IntegerParser<'s, 'a> {
    /// Constructs a new integer parser reading from the scanner with the given
    /// configuration and scratch digit buffer.
    fn new(
        scan: &'a mut Scanner<'s>,
        syntax: &'a Syntax,
        cfg: ParseConfig,
        digit_buf: &'a mut Vec<u8>,
    ) -> Self {
        IntegerParser {
            scan,
            syntax,
            cfg,
            errors: EnumSet::empty(),
            digit_buf,
        }
    }

    /// Parses an integer token.
    fn parse(&mut self) -> IntegerToken {
        self.space(SpaceLocation::Leading);
        let open_parens = self.open_parens();

        let (negative, sign_style) = self.sign();
        let mut base_style = self.base_prefix();
        self.digits();
        if base_style == BaseStyle::Decimal {
            base_style = self.base_suffix();
        }
        if !self.syntax.base_styles.contains(base_style) {
            self.errors |= IntegerError::BaseStyle;
        }

        // TODO

        self.close_parens(open_parens);
        self.space(SpaceLocation::Trailing);

        IntegerToken {
            value: Integer::new(),
            negative,
            sign_style,
            base_style,
            digit_seps: EnumSet::empty(),
            errors: self.errors,
        }
    }

    /// Consumes whitespace characters at the given location.
    fn space(&mut self, location: SpaceLocation) {
        let bumped = !self
            .scan
            .bump_while_char(|ch| ch.is_whitespace())
            .is_empty();
        if bumped && !self.syntax.space_locations.contains(location) {
            self.errors |= IntegerError::SpaceLocation;
        }
    }

    /// Consumes open parentheses and returns the count.
    fn open_parens(&mut self) -> usize {
        let mut open_parens = 0;
        if self.cfg.parens {
            while self.scan.bump_if_ascii(|ch| ch == b'(') {
                open_parens += 1;
                self.space(SpaceLocation::BetweenParens);
            }
        }
        open_parens
    }

    /// Consumes close parentheses until it matches the number of open
    /// parentheses.
    fn close_parens(&mut self, open_parens: usize) {
        let mut close_parens = 0;
        while close_parens < open_parens {
            self.space(SpaceLocation::BetweenParens);
            if self.scan.bump_if_ascii(|ch| ch == b')') {
                close_parens += 1;
            } else {
                self.errors |= IntegerError::UnpairedParen;
                break;
            }
        }
    }

    /// Consumes the sign and returns whether it is negative and the lexical
    /// representation of the sign.
    fn sign(&mut self) -> (bool, SignStyle) {
        let mut negative = false;
        let mut sign = SignStyle::ImplicitPos;
        let mut sign_count = 0usize;
        while let Some(b) = self.scan.peek_byte() {
            sign = match b {
                b'-' => {
                    negative = !negative;
                    SignStyle::Neg
                }
                b'+' => SignStyle::Pos,
                _ => break,
            };
            if !self.syntax.signs.contains(sign) {
                self.errors |= IntegerError::SignStyle;
            }
            sign_count += 1;
            self.scan.bump_ascii_no_lf(1);
            self.space(SpaceLocation::AfterSign);
        }
        if sign_count > 1 {
            sign = SignStyle::Multiple;
            if !self.syntax.signs.contains(SignStyle::Multiple) {
                self.errors |= IntegerError::SignStyle;
            }
        }
        (negative, sign)
    }

    /// Consumes and returns a base prefix. Octal `0` is handled after reading
    /// the digits.
    fn base_prefix(&mut self) -> BaseStyle {
        if self.scan.peek_byte() == Some(b'0') {
            let prefix = match self.scan.peek_byte_at(1) {
                Some(b'b') => BaseStyle::BinPrefix_0b,
                Some(b'B') => BaseStyle::BinPrefix_0B,
                Some(b'o') => BaseStyle::OctPrefix_0o,
                Some(b'O') => BaseStyle::OctPrefix_0O,
                Some(b'x') => BaseStyle::HexPrefix_0x,
                Some(b'X') => BaseStyle::HexPrefix_0X,
                _ => return BaseStyle::Decimal,
            };
            self.scan.bump_ascii_no_lf(2);
            prefix
        } else {
            BaseStyle::Decimal
        }
    }

    /// Consumes digits and parses them to `self.digit_buf`.
    fn digits(&mut self) {
        let digits = &mut *self.digit_buf;
        digits.clear();
        let mut largest_digit = 0;

        while let Some(b) = self.scan.peek_byte() {
            let digit10 = b.wrapping_sub(b'0');
            let digit16 = (b | 0x20).wrapping_sub(b'a');
            let digit = if digit10 < 10 {
                digit10
            } else if digit16 < 6 {
                digit16 + 10
            } else {
                unlikely();
                let digit_sep = if b == b'_' {
                    DigitSep::Underscore
                } else if b == b'\'' && self.cfg.quote_digit_sep {
                    DigitSep::SingleQuote
                } else {
                    break;
                };
                if !self.syntax.digit_seps.contains(digit_sep) {
                    self.errors |= IntegerError::DigitSep;
                }
                continue;
            };
            digits.push(digit);
            largest_digit = largest_digit.max(digit);
            self.scan.bump_ascii_no_lf(1);
        }
    }

    /// Consumes and returns a base suffix.
    fn base_suffix(&mut self) -> BaseStyle {
        let base_suffix = match self.scan.peek_byte() {
            Some(b'o') => BaseStyle::OctSuffix_o,
            Some(b'O') => BaseStyle::OctSuffix_O,
            Some(b'h') => BaseStyle::HexSuffix_h,
            Some(b'H') => BaseStyle::HexSuffix_H,
            _ => return BaseStyle::Decimal,
        };
        if matches!(
            self.scan.peek_byte_at(1),
            Some(b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'\'')
        ) {
            return BaseStyle::Decimal;
        }
        self.scan.bump_ascii_no_lf(1);
        if self.syntax.suffix_decimal_first
            && matches!(base_suffix, BaseStyle::HexSuffix_h | BaseStyle::HexSuffix_H)
            && self.digit_buf.first().is_some_and(|&digit| digit > 9)
        {
            self.errors |= IntegerError::StartsWithHex;
        }
        base_suffix
    }
}

/// Mark a branch as unlikely to the optimizer.
#[cold]
#[inline]
fn unlikely() {}
