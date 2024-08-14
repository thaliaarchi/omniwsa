//! Parsing for Haskell `Integer`.

use rug::Integer;

use crate::token::{IntegerBase, IntegerSign, IntegerToken};

// TODO:
// - Make parser not fallible.

/// An error from parsing a Haskell-syntax integer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HaskellIntegerError {
    InvalidDigit,
    NoDigits,
    UnpairedParen,
    IllegalNeg,
}

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
pub fn parse_haskell_integer(
    mut s: &str,
    digits: &mut Vec<u8>,
) -> Result<IntegerToken, HaskellIntegerError> {
    use HaskellIntegerError as Error;

    digits.clear();

    loop {
        s = s.trim_matches(is_whitespace);
        if !s.is_empty() {
            let (first, last) = (s.as_bytes()[0], s.as_bytes()[s.len() - 1]);
            if first == b'(' && last == b')' {
                s = &s[1..s.len() - 1];
                continue;
            } else if first == b'(' || last == b')' {
                if first == b'-' {
                    return Err(Error::IllegalNeg);
                }
                return Err(Error::UnpairedParen);
            }
        }
        break;
    }

    let sign = if !s.is_empty() && s.as_bytes()[0] == b'-' {
        s = s[1..].trim_start_matches(is_whitespace);
        IntegerSign::Neg
    } else {
        IntegerSign::None
    };

    let b = s.as_bytes();
    let (base, b) = match b {
        [b'0', b'o' | b'O', b @ ..] => (IntegerBase::Octal, b),
        [b'0', b'x' | b'X', b @ ..] => (IntegerBase::Hexadecimal, b),
        _ => (IntegerBase::Decimal, b),
    };
    let leading_zeros = b.iter().take_while(|&&ch| ch == b'0').count();
    let b = &b[leading_zeros..];

    let mut value = Integer::new();
    if !b.is_empty() {
        digits.reserve(b.len());
        match base {
            IntegerBase::Decimal => {
                for &ch in b {
                    let digit = ch.wrapping_sub(b'0');
                    if digit >= 10 {
                        return Err(Error::InvalidDigit);
                    }
                    digits.push(digit);
                }
            }
            IntegerBase::Octal => {
                for &ch in b {
                    let digit = ch.wrapping_sub(b'0');
                    if digit >= 8 {
                        return Err(Error::InvalidDigit);
                    }
                    digits.push(digit);
                }
            }
            IntegerBase::Hexadecimal => {
                for &ch in b {
                    let digit = match ch {
                        b'0'..=b'9' => ch - b'0',
                        b'a'..=b'f' => ch - b'a' + 10,
                        b'A'..=b'F' => ch - b'A' + 10,
                        _ => return Err(Error::InvalidDigit),
                    };
                    digits.push(digit);
                }
            }
            IntegerBase::Binary => unreachable!(),
        }
        // SAFETY: Digits are constructed to be in range for the base.
        unsafe {
            value.assign_bytes_radix_unchecked(digits, base as i32, sign == IntegerSign::Neg);
        }
    } else if leading_zeros == 0 {
        return Err(Error::NoDigits);
    }

    Ok(IntegerToken {
        value,
        sign,
        base,
        leading_zeros,
    })
}

#[inline]
fn is_whitespace(ch: char) -> bool {
    ch.is_whitespace() && ch != '\u{0085}' && ch != '\u{2028}' && ch != '\u{2029}'
}
