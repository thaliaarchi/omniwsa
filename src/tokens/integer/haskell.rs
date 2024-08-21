//! Haskell-style integer parsing with the syntax of
//! [`read :: String -> Integer`](https://hackage.haskell.org/package/base/docs/GHC-Read.html).
//!
//! # Compliance
//!
//! It has been tested to match the behavior of at least GHC 8.8.4 and 9.4.4
//! and matches the source of GHC 9.8.1 by inspection.
//!
//! # GHC definitions
//!
//! See [`Text.Read.Lex.lexNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L418-447)
//! for the number grammar and [`GHC.Read.readNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L557-568)
//! for the handling of spaces, parens, and negative.
//!
//! - [`Text.Read.read`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read.hs#L102-113)
//!   ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:read))
//!   - [`Text.Read.readEither`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read.hs#L64-85)
//!     ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:readEither))
//!     - `readPrec` in instance [`Read Integer`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L616-619)
//!       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Read.html#v:readPrec))
//!       - [`GHC.Read.readNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L557-568)
//!         ([docs](https://hackage.haskell.org/package/base/docs/GHC-Read.html#v:readNumber))
//!         - [`GHC.Read.parens`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L323-330)
//!         - [`GHC.Read.lexP`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L291-293)
//!           ([docs](https://hackage.haskell.org/package/base/docs/GHC-Read.html#v:lexP))
//!           - [`Text.Read.Lex.lex`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L170-171)
//!             ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:lex))
//!             - [`Text.ParserCombinators.ReadP.skipSpaces`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L311-318)
//!               - [`GHC.Unicode.isSpace`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Unicode.hs#L222-235)
//!             - [`Text.Read.Lex.lexToken`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L185-192)
//!               - [`Text.Read.Lex.lexNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L418-447)
//!                 - …
//!               - …
//!       - [`GHC.Read.convertInt`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L571-574)
//!         - [`Text.Read.Lex.numberToInteger`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L87-90)
//!           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-Read-Lex.html#v:numberToInteger))
//!           - [`Text.Read.Lex.val`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L484-525)
//!         - `Num.fromInteger` in `GHC.Num`
//!           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Num.html#v:fromInteger))
//!         - [`Text.ParserCombinators.ReadP.pfail`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L219-221)
//!           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadP.html#v:pfail))
//!     - [`Text.ParserCombinators.ReadPrec.minPrec`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L105-106)
//!       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:minPrec))
//!     - `Text.ParserCombinators.ReadP.skipSpaces` (see above)
//!     - [`Text.ParserCombinators.ReadPrec.lift`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L111-113)
//!       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:lift))
//!     - [`Text.ParserCombinators.ReadPrec.readPrec_to_S`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L172-173)
//!       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:readPrec_to_S))
//!       - [`Text.ParserCombinators.ReadP.readP_to_S`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L418-423)
//!         ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadP.html#v:readP_to_S))
//!   - [`GHC.Err.errorWithoutStackTrace`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Err.hs#L42-47)
//!     ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Err.html#v:errorWithoutStackTrace))

use std::borrow::Cow;

use enumset::EnumSet;

use crate::tokens::integer::{IntegerBase, IntegerError, IntegerSign, IntegerToken};

impl<'s> IntegerToken<'s> {
    /// Parses an integer with the syntax of [`read :: String -> Integer`](https://hackage.haskell.org/package/base/docs/GHC-Read.html)
    /// in Haskell, given a buffer of digits to reuse allocations.
    ///
    /// # Syntax
    ///
    /// Octal literals are prefixed with `0o` or `0O` and hexadecimal literals
    /// with `0x` or `0X`. Binary literals with `0b` or `0B` are not supported.
    /// A leading zero is interpreted as decimal, not octal. It may have a
    /// negative sign. It may be surrounded by any number of parentheses.
    /// Unicode whitespace characters may occur around the digits, sign, or
    /// parentheses. Positive signs, underscore digit separators, and exponents
    /// are not allowed.
    ///
    /// Haskell's `String` must be UTF-8 and excludes surrogate halves, so it is
    /// equivalent to Rust strings and validation happens outside of `read`.
    ///
    /// ```bnf
    /// read        ::= space* "(" read ")" space*
    ///               | space* integer space*
    /// integer     ::= "-"? space* (dec_integer | oct_integer | hex_integer)
    /// dec_integer ::= [0-9]+
    /// oct_integer ::= "0" [oO] [0-7]+
    /// hex_integer ::= "0" [xX] [0-9 a-f A-F]+
    /// space       ::= \p{White_Space} NOT (U+0085 | U+2028 | U+2029)
    /// ```
    pub fn parse_haskell(literal: Cow<'s, str>, digits: &mut Vec<u8>) -> Self {
        let mut int = IntegerToken::new();
        let (sign, s, sign_errors) = IntegerToken::strip_haskell_sign(&literal);
        int.sign = sign;
        int.errors |= sign_errors;
        let (base, s) = IntegerToken::strip_base_rust(s.as_bytes());
        int.base = base;
        if base == IntegerBase::Binary {
            int.errors |= IntegerError::InvalidBase;
        }
        int.parse_digits(s, digits);
        if int.has_digit_sep {
            int.errors |= IntegerError::InvalidDigitSep;
        }
        int.literal = match literal {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        };
        int
    }

    /// Strips parentheses groupings and a sign for an integer literal with
    /// Haskell `Integer` syntax. See [`IntegerToken::parse_haskell`] for the
    /// grammar.
    pub fn strip_haskell_sign(mut s: &str) -> (IntegerSign, &str, EnumSet<IntegerError>) {
        fn is_whitespace(ch: char) -> bool {
            ch.is_whitespace() && ch != '\u{0085}' && ch != '\u{2028}' && ch != '\u{2029}'
        }

        let mut errors = EnumSet::new();
        let mut sign = IntegerSign::None;
        let mut has_sign = false;
        s = s.trim_matches(is_whitespace);
        loop {
            if s.is_empty() {
                break;
            }
            let (first, last) = (s.as_bytes()[0], s.as_bytes()[s.len() - 1]);
            if first == b'-' {
                sign = match sign {
                    IntegerSign::None | IntegerSign::Pos => IntegerSign::Neg,
                    IntegerSign::Neg => IntegerSign::Pos,
                };
                if has_sign {
                    errors |= IntegerError::InvalidSign;
                }
                has_sign = true;
                s = s[1..].trim_start_matches(is_whitespace);
            } else if first == b'+' {
                if sign == IntegerSign::None {
                    sign = IntegerSign::Pos;
                }
                has_sign = true;
                errors |= IntegerError::InvalidSign;
                s = s[1..].trim_start_matches(is_whitespace);
            } else if first == b'(' && last == b')' {
                if has_sign {
                    errors |= IntegerError::InvalidSign;
                }
                s = s[1..s.len() - 1].trim_matches(is_whitespace);
            } else if first == b'(' {
                if has_sign {
                    errors |= IntegerError::InvalidSign;
                }
                errors |= IntegerError::UnpairedParen;
                s = s[1..].trim_start_matches(is_whitespace);
            } else if last == b')' {
                errors |= IntegerError::UnpairedParen;
                s = s[..s.len() - 1].trim_end_matches(is_whitespace);
            } else {
                break;
            }
        }
        (sign, s, errors)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use enumset::EnumSet;

    use crate::tokens::integer::{Integer, IntegerBase, IntegerError, IntegerSign, IntegerToken};

    use IntegerBase::{Binary as Bin, Decimal as Dec, Hexadecimal as Hex, Octal as Oct};
    use IntegerError::*;
    use IntegerSign::{Neg, None as No, Pos};

    const T: bool = true;
    const F: bool = false;

    struct Test {
        input: String,
        output: IntegerToken<'static>,
    }

    impl Test {
        fn new<S: Into<String> + Clone>(
            input: S,
            output: &'static str,
            sign: IntegerSign,
            base: IntegerBase,
            leading_zeros: usize,
            has_digit_sep: bool,
            errors: EnumSet<IntegerError>,
        ) -> Self {
            Test {
                input: input.clone().into(),
                output: IntegerToken {
                    literal: Cow::Owned(input.into().into_bytes()),
                    value: Integer::parse(output).unwrap().into(),
                    sign,
                    base,
                    leading_zeros,
                    has_digit_sep,
                    errors,
                },
            }
        }
    }

    macro_rules! test(
        ($input:expr $(,)? => $output:expr, $sign:expr, $base:expr, $leading_zeros:expr, $has_digit_sep:expr $(,)?) => {
            Test::new($input, $output, $sign, $base, $leading_zeros, $has_digit_sep, EnumSet::new())
        };
        ($input:expr $(,)? => $output:expr, $sign:expr, $base:expr, $leading_zeros:expr, $has_digit_sep:expr; $err:expr $(,)?) => {
            Test::new($input, $output, $sign, $base, $leading_zeros, $has_digit_sep, EnumSet::from($err))
        };
    );

    #[test]
    fn parse_haskell() {
        let mut tests = vec![
            test!("42" => "42", No, Dec, 0, F),
            // C-style bases
            test!("0o42" => "34", No, Oct, 0, F),
            test!("0O42" => "34", No, Oct, 0, F),
            test!("0xff" => "255", No, Hex, 0, F),
            test!("0Xff" => "255", No, Hex, 0, F),
            test!("0Xff" => "255", No, Hex, 0, F),
            test!("0b101" => "5", No, Bin, 0, F; InvalidBase),
            test!("0B101" => "5", No, Bin, 0, F; InvalidBase),
            // Leading zeros
            test!("000" => "0", No, Dec, 3, F),
            test!("042" => "42", No, Dec, 1, F),
            test!("00042" => "42", No, Dec, 3, F),
            test!("0o00042" => "34", No, Oct, 3, F),
            test!("0x000ff" => "255", No, Hex, 3, F),
            // Other styles
            test!("0d42" => "0", No, Dec, 1, F; InvalidDigit),
            test!("2#101" => "2", No, Dec, 0, F; InvalidDigit),
            test!("2#101#" => "2", No, Dec, 0, F; InvalidDigit),
            test!("&b101" => "0", No, Dec, 0, F; InvalidDigit),
            test!("&o42" => "0", No, Dec, 0, F; InvalidDigit),
            test!("&hff" => "0", No, Dec, 0, F; InvalidDigit),
            // Signs
            test!("-42" => "-42", Neg, Dec, 0, F),
            test!("+42" => "42", Pos, Dec, 0, F; InvalidSign),
            // Parentheses
            test!("(42)" => "42", No, Dec, 0, F),
            test!("((42))" => "42", No, Dec, 0, F),
            test!("(((42)))" => "42", No, Dec, 0, F),
            test!(" ( ( ( 42 ) ) ) " => "42", No, Dec, 0, F),
            test!("(-42)" => "-42", Neg, Dec, 0, F),
            test!("-(42)" => "-42", Neg, Dec, 0, F; InvalidSign),
            test!("-(-42)" => "42", Pos, Dec, 0, F; InvalidSign),
            test!("(--42)" => "42", Pos, Dec, 0, F; InvalidSign),
            test!("(- -42)" => "42", Pos, Dec, 0, F; InvalidSign),
            test!("(-(-42))" => "42", Pos, Dec, 0, F; InvalidSign),
            test!("(42" => "42", No, Dec, 0, F; UnpairedParen),
            test!("42)" => "42", No, Dec, 0, F; UnpairedParen),
            test!("-(42" => "-42", Neg, Dec, 0, F; UnpairedParen | InvalidSign),
            test!("-42)" => "-42", Neg, Dec, 0, F; UnpairedParen),
            test!("((42)" => "42", No, Dec, 0, F; UnpairedParen),
            test!("(42))" => "42", No, Dec, 0, F; UnpairedParen),
            // Exponent
            test!("1e3" => "1", No, Dec, 0, F; InvalidDigit),
            // Decimal point
            test!("3.14" => "3", No, Dec, 0, F; InvalidDigit),
            // Digit separators
            test!("1_000" => "1000", No, Dec, 0, T; InvalidDigitSep),
            test!("1 000" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1,000" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1'000" => "1", No, Dec, 0, F; InvalidDigit),
            test!("0o_42" => "34", No, Oct, 0, T; InvalidDigitSep),
            test!("0Xf_f" => "255", No, Hex, 0, T; InvalidDigitSep),
            test!("0O42_" => "34", No, Oct, 0, T; InvalidDigitSep),
            // Larger than 128 bits
            test!(
                "31415926535897932384626433832795028841971693993751" =>
                "31415926535897932384626433832795028841971693993751",
                No,
                Dec,
                0,
                F,
            ),
            // Empty
            test!("" => "0", No, Dec, 0, F; NoDigits),
            test!("-" => "0", Neg, Dec, 0, F; NoDigits),
            // Operations
            test!("1+2" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1-2" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1*2" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1/2" => "1", No, Dec, 0, F; InvalidDigit),
            test!("1%2" => "1", No, Dec, 0, F; InvalidDigit),
            // Non-digits
            test!("9000over" => "9000", No, Dec, 0, F; InvalidDigit),
            test!("invalid" => "0", No, Dec, 0, F; InvalidDigit),
        ];

        // All characters with the Unicode property White_Space, excluding
        // non-ASCII line-breaks, are allowed before or after the digits, or
        // between the `-` sign and the digits.
        let ok_spaces = [
            // Unicode White_Space
            '\t',       // Tab
            '\n',       // Line feed
            '\x0b',     // Vertical tab
            '\x0c',     // Form feed
            '\r',       // Carriage return
            ' ',        // Space
            '\u{00A0}', // No-break space
            '\u{1680}', // Ogham space mark
            '\u{2000}', // En quad
            '\u{2001}', // Em quad
            '\u{2002}', // En space
            '\u{2003}', // Em space
            '\u{2004}', // Three-per-em space
            '\u{2005}', // Four-per-em space
            '\u{2006}', // Six-per-em space
            '\u{2007}', // Figure space
            '\u{2008}', // Punctuation space
            '\u{2009}', // Thin space
            '\u{200A}', // Hair space
            '\u{202F}', // Narrow no-break space
            '\u{205F}', // Medium mathematical space
            '\u{3000}', // Ideographic space
        ];
        let err_spaces = [
            // Unicode White_Space
            '\u{0085}', // Next line
            '\u{2028}', // Line separator
            '\u{2029}', // Paragraph separator
            // Related Unicode characters
            '\u{180E}', // Mongolian vowel separator
            '\u{200B}', // Zero width space
            '\u{200C}', // Zero width non-joiner
            '\u{200D}', // Zero width joiner
            '\u{200E}', // Left-to-right mark
            '\u{200F}', // Right-to-left mark
            '\u{2060}', // Word joiner
            '\u{FEFF}', // Zero width non-breaking space
        ];
        for space in ok_spaces {
            tests.push(test!(format!("{space}") => "0", No, Dec, 0, F; NoDigits));
            tests.push(test!(format!("{space}-42") => "-42", Neg, Dec, 0, F));
            tests.push(test!(format!("-{space}42") => "-42", Neg, Dec, 0, F));
            tests.push(test!(format!("-4{space}2") => "-4", Neg, Dec, 0, F; InvalidDigit));
            tests.push(test!(format!("-42{space}") => "-42", Neg, Dec, 0, F));
        }
        for space in err_spaces {
            tests.push(test!(format!("{space}") => "0", No, Dec, 0, F; InvalidDigit));
            tests.push(test!(format!("{space}-42") => "0", No, Dec, 0, F; InvalidDigit));
            tests.push(test!(format!("-{space}42") => "0", Neg, Dec, 0, F; InvalidDigit));
            tests.push(test!(format!("-4{space}2") => "-4", Neg, Dec, 0, F; InvalidDigit));
            tests.push(test!(format!("-42{space}") => "-42", Neg, Dec, 0, F; InvalidDigit));
        }

        let mut digits = Vec::new();
        for test in tests {
            assert_eq!(
                IntegerToken::parse_haskell(test.input.clone().into(), &mut digits),
                test.output,
                "IntegerToken::parse_haskell({:?})",
                test.input,
            );
        }
    }
}
