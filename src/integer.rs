//! Parsing for Haskell `Integer`.

use enumset::EnumSet;
use rug::Integer;

use crate::token::{IntegerBase, IntegerError, IntegerSign, IntegerToken};

/// Parses an integer with the syntax of [`read :: String -> Integer`](https://hackage.haskell.org/package/base/docs/GHC-Read.html)
/// in Haskell, given a buffer of digits to reuse allocations.
///
/// # Syntax
///
/// Octal literals are prefixed with `0o` or `0O` and hexadecimal literals with
/// `0x` or `0X`. Binary literals with `0b` or `0B` are not supported. A leading
/// zero is interpreted as decimal, not octal. It may have a negative sign. It
/// may be surrounded by any number of parentheses. Unicode whitespace
/// characters may occur around the digits, sign, or parentheses. Positive
/// signs, underscore digit separators, and exponents are not allowed.
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
///
/// # Compliance
///
/// It has been tested to match the behavior of at least GHC 8.8.4 and 9.4.4 and
/// matches the source of GHC 9.8.1 by inspection.
///
/// # GHC definitions
///
/// See [`Text.Read.Lex.lexNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L418-447)
/// for the number grammar and [`GHC.Read.readNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L557-568)
/// for the handling of spaces, parens, and negative.
///
/// - [`Text.Read.read`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read.hs#L102-113)
///   ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:read))
///   - [`Text.Read.readEither`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read.hs#L64-85)
///     ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:readEither))
///     - `readPrec` in instance [`Read Integer`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L616-619)
///       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Read.html#v:readPrec))
///       - [`GHC.Read.readNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L557-568)
///         ([docs](https://hackage.haskell.org/package/base/docs/GHC-Read.html#v:readNumber))
///         - [`GHC.Read.parens`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L323-330)
///         - [`GHC.Read.lexP`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L291-293)
///           ([docs](https://hackage.haskell.org/package/base/docs/GHC-Read.html#v:lexP))
///           - [`Text.Read.Lex.lex`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L170-171)
///             ([docs](https://hackage.haskell.org/package/base/docs/Text-Read.html#v:lex))
///             - [`Text.ParserCombinators.ReadP.skipSpaces`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L311-318)
///               - [`GHC.Unicode.isSpace`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Unicode.hs#L222-235)
///             - [`Text.Read.Lex.lexToken`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L185-192)
///               - [`Text.Read.Lex.lexNumber`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L418-447)
///                 - …
///               - …
///       - [`GHC.Read.convertInt`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Read.hs#L571-574)
///         - [`Text.Read.Lex.numberToInteger`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L87-90)
///           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-Read-Lex.html#v:numberToInteger))
///           - [`Text.Read.Lex.val`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/Read/Lex.hs#L484-525)
///         - `Num.fromInteger` in `GHC.Num`
///           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Num.html#v:fromInteger))
///         - [`Text.ParserCombinators.ReadP.pfail`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L219-221)
///           ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadP.html#v:pfail))
///     - [`Text.ParserCombinators.ReadPrec.minPrec`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L105-106)
///       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:minPrec))
///     - `Text.ParserCombinators.ReadP.skipSpaces` (see above)
///     - [`Text.ParserCombinators.ReadPrec.lift`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L111-113)
///       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:lift))
///     - [`Text.ParserCombinators.ReadPrec.readPrec_to_S`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadPrec.hs#L172-173)
///       ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadPrec.html#v:readPrec_to_S))
///       - [`Text.ParserCombinators.ReadP.readP_to_S`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/Text/ParserCombinators/ReadP.hs#L418-423)
///         ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/Text-ParserCombinators-ReadP.html#v:readP_to_S))
///   - [`GHC.Err.errorWithoutStackTrace`](https://gitlab.haskell.org/ghc/ghc/-/blob/ghc-9.8.1-release/libraries/base/GHC/Err.hs#L42-47)
///     ([docs](https://hackage.haskell.org/package/base-4.19.0.0/docs/GHC-Err.html#v:errorWithoutStackTrace))
pub fn parse_haskell_integer(mut s: &str, digits: &mut Vec<u8>) -> IntegerToken {
    let mut errors = EnumSet::new();

    // Rather than add another sign variant for multiple negations, just use
    // `Pos`, since that is only exercised for errors as a best effort.
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

    let (base, s) = match s.as_bytes() {
        [b'0', b'o' | b'O', s @ ..] => (IntegerBase::Octal, s),
        [b'0', b'x' | b'X', s @ ..] => (IntegerBase::Hexadecimal, s),
        [b'0', b'b' | b'B', s @ ..] => {
            // Extend the syntax to handle binary, just for errors.
            errors |= IntegerError::InvalidBase;
            (IntegerBase::Binary, s)
        }
        s => (IntegerBase::Decimal, s),
    };

    parse_integer_digits(s, sign, base, errors, digits)
}

/// Parses the byte string as digits in the given base. No digit separators like
/// `_` are supported.
pub fn parse_integer_digits(
    s: &[u8],
    sign: IntegerSign,
    base: IntegerBase,
    mut errors: EnumSet<IntegerError>,
    digits: &mut Vec<u8>,
) -> IntegerToken {
    digits.clear();

    let leading_zeros = s.iter().take_while(|&&ch| ch == b'0').count();
    let s = &s[leading_zeros..];

    let mut value = Integer::new();
    if !s.is_empty() {
        digits.reserve(s.len());
        match base {
            IntegerBase::Decimal => {
                for &b in s {
                    let digit = b.wrapping_sub(b'0');
                    if digit >= 10 {
                        if digit == b'_' - b'0' {
                            errors |= IntegerError::InvalidDigitSep;
                            continue;
                        }
                        errors |= IntegerError::InvalidDigit;
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
                                errors |= IntegerError::InvalidDigitSep;
                                continue;
                            }
                            errors |= IntegerError::InvalidDigit;
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
                            errors |= IntegerError::InvalidDigitSep;
                            continue;
                        }
                        errors |= IntegerError::InvalidDigit;
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
                            errors |= IntegerError::InvalidDigitSep;
                            continue;
                        }
                        errors |= IntegerError::InvalidDigit;
                        break;
                    }
                    digits.push(digit);
                }
            }
        }
        // SAFETY: Digits are constructed to be in range for the base.
        unsafe {
            value.assign_bytes_radix_unchecked(digits, base as i32, sign == IntegerSign::Neg);
        }
    } else if leading_zeros == 0 {
        errors |= IntegerError::NoDigits;
    }

    IntegerToken {
        value,
        sign,
        base,
        leading_zeros,
        errors,
    }
}

/// Returns whether the char is considered whitespace for the purposes of
/// parsing a Haskell `Integer`.
fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() && ch != '\u{0085}' && ch != '\u{2028}' && ch != '\u{2029}'
}

#[cfg(test)]
mod tests {
    use enumset::EnumSet;
    use rug::Integer;

    use crate::{
        integer::parse_haskell_integer,
        token::{IntegerBase, IntegerError, IntegerSign, IntegerToken},
    };

    use IntegerBase::{Binary as Bin, Decimal as Dec, Hexadecimal as Hex, Octal as Oct};
    use IntegerError::*;
    use IntegerSign::{Neg, None as No, Pos};

    struct Test {
        input: String,
        output: IntegerToken,
    }

    impl Test {
        fn new<S: Into<String>>(
            input: S,
            output: &'static str,
            sign: IntegerSign,
            base: IntegerBase,
            leading_zeros: usize,
            errors: EnumSet<IntegerError>,
        ) -> Self {
            Test {
                input: input.into(),
                output: IntegerToken {
                    value: Integer::parse(output).unwrap().into(),
                    sign,
                    base,
                    leading_zeros,
                    errors,
                },
            }
        }
    }

    macro_rules! test(
        ($input:expr $(,)? => $output:expr, $sign:expr, $base:expr, $leading_zeros:expr $(,)?) => {
            Test::new($input, $output, $sign, $base, $leading_zeros, EnumSet::new())
        };
        ($input:expr $(,)? => $output:expr, $sign:expr, $base:expr, $leading_zeros:expr; $err:expr $(,)?) => {
            Test::new($input, $output, $sign, $base, $leading_zeros, EnumSet::from($err))
        };
    );

    #[test]
    fn haskell_integer() {
        let mut tests = vec![
            test!("42" => "42", No, Dec, 0),
            // C-style bases
            test!("0o42" => "34", No, Oct, 0),
            test!("0O42" => "34", No, Oct, 0),
            test!("0xff" => "255", No, Hex, 0),
            test!("0Xff" => "255", No, Hex, 0),
            test!("0Xff" => "255", No, Hex, 0),
            test!("0b101" => "5", No, Bin, 0; InvalidBase),
            test!("0B101" => "5", No, Bin, 0; InvalidBase),
            // Leading zeros
            test!("000" => "0", No, Dec, 3),
            test!("042" => "42", No, Dec, 1),
            test!("00042" => "42", No, Dec, 3),
            test!("0o00042" => "34", No, Oct, 3),
            test!("0x000ff" => "255", No, Hex, 3),
            // Other styles
            test!("0d42" => "0", No, Dec, 1; InvalidDigit),
            test!("2#101" => "2", No, Dec, 0; InvalidDigit),
            test!("2#101#" => "2", No, Dec, 0; InvalidDigit),
            test!("&b101" => "0", No, Dec, 0; InvalidDigit),
            test!("&o42" => "0", No, Dec, 0; InvalidDigit),
            test!("&hff" => "0", No, Dec, 0; InvalidDigit),
            // Signs
            test!("-42" => "-42", Neg, Dec, 0),
            test!("+42" => "42", Pos, Dec, 0; InvalidSign),
            // Parentheses
            test!("(42)" => "42", No, Dec, 0),
            test!("((42))" => "42", No, Dec, 0),
            test!("(((42)))" => "42", No, Dec, 0),
            test!(" ( ( ( 42 ) ) ) " => "42", No, Dec, 0),
            test!("(-42)" => "-42", Neg, Dec, 0),
            test!("-(42)" => "-42", Neg, Dec, 0; InvalidSign),
            test!("-(-42)" => "42", Pos, Dec, 0; InvalidSign),
            test!("(--42)" => "42", Pos, Dec, 0; InvalidSign),
            test!("(- -42)" => "42", Pos, Dec, 0; InvalidSign),
            test!("(-(-42))" => "42", Pos, Dec, 0; InvalidSign),
            test!("(42" => "42", No, Dec, 0; UnpairedParen),
            test!("42)" => "42", No, Dec, 0; UnpairedParen),
            test!("-(42" => "-42", Neg, Dec, 0; UnpairedParen | InvalidSign),
            test!("-42)" => "-42", Neg, Dec, 0; UnpairedParen),
            test!("((42)" => "42", No, Dec, 0; UnpairedParen),
            test!("(42))" => "42", No, Dec, 0; UnpairedParen),
            // Exponent
            test!("1e3" => "1", No, Dec, 0; InvalidDigit),
            // Decimal point
            test!("3.14" => "3", No, Dec, 0; InvalidDigit),
            // Digit separators
            test!("1_000" => "1000", No, Dec, 0; InvalidDigitSep),
            test!("1 000" => "1", No, Dec, 0; InvalidDigit),
            test!("1,000" => "1", No, Dec, 0; InvalidDigit),
            test!("1'000" => "1", No, Dec, 0; InvalidDigit),
            test!("0o_42" => "34", No, Oct, 0; InvalidDigitSep),
            test!("0Xf_f" => "255", No, Hex, 0; InvalidDigitSep),
            test!("0O42_" => "34", No, Oct, 0; InvalidDigitSep),
            // Larger than 128 bits
            test!(
                "31415926535897932384626433832795028841971693993751" =>
                "31415926535897932384626433832795028841971693993751",
                No,
                Dec,
                0,
            ),
            // Empty
            test!("" => "0", No, Dec, 0; NoDigits),
            test!("-" => "0", Neg, Dec, 0; NoDigits),
            // Operations
            test!("1+2" => "1", No, Dec, 0; InvalidDigit),
            test!("1-2" => "1", No, Dec, 0; InvalidDigit),
            test!("1*2" => "1", No, Dec, 0; InvalidDigit),
            test!("1/2" => "1", No, Dec, 0; InvalidDigit),
            test!("1%2" => "1", No, Dec, 0; InvalidDigit),
            // Non-digits
            test!("9000over" => "9000", No, Dec, 0; InvalidDigit),
            test!("invalid" => "0", No, Dec, 0; InvalidDigit),
        ];

        // All characters with the Unicode property White_Space, excluding non-ASCII
        // line-breaks, are allowed before or after the digits, or between the `-`
        // sign and the digits.
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
            tests.push(test!(format!("{space}") => "0", No, Dec, 0; NoDigits));
            tests.push(test!(format!("{space}-42") => "-42", Neg, Dec, 0));
            tests.push(test!(format!("-{space}42") => "-42", Neg, Dec, 0));
            tests.push(test!(format!("-4{space}2") => "-4", Neg, Dec, 0; InvalidDigit));
            tests.push(test!(format!("-42{space}") => "-42", Neg, Dec, 0));
        }
        for space in err_spaces {
            tests.push(test!(format!("{space}") => "0", No, Dec, 0; InvalidDigit));
            tests.push(test!(format!("{space}-42") => "0", No, Dec, 0; InvalidDigit));
            tests.push(test!(format!("-{space}42") => "0", Neg, Dec, 0; InvalidDigit));
            tests.push(test!(format!("-4{space}2") => "-4", Neg, Dec, 0; InvalidDigit));
            tests.push(test!(format!("-42{space}") => "-42", Neg, Dec, 0; InvalidDigit));
        }

        let mut digits = Vec::new();
        for test in tests {
            assert_eq!(
                parse_haskell_integer(&test.input, &mut digits),
                test.output,
                "parse_haskell_integer({:?})",
                test.input,
            );
        }
    }
}
