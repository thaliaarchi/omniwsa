# Generalized integer parsing

Currently, integers are scanned then the substring is parsed. This is ideal for
parsers which segment into words first, but leads to a lot of logic duplication
for scanning parsers. Since a scanner can be applied to a segment, scanning is
the more general approach. The following details generalized integer parsing
while scanning.

## Algorithm

**Leading whitespace**

- If strip whitespace: Error unless `space_positions` includes `Leading`.

**Open parens**

- Initialize open count to 0.
- If `enable_parens` in config:
  - While bump '(':
    - Increment open count.
    - If strip whitespace: Error unless `space_positions` includes `BetweenParens`.

**Signs**

- Initialize sign to `None` in `enum { None, Neg, Pos }`
- Initialize sign count to 0.
- Loop:
  - If bump '-':
    - Set sign to: Match sign: Case `None` or `Pos`: `Neg`; Case `Neg`: `Pos`.
    - Error unless `signs` includes `Neg`.
  - Else if bump '+':
    - Set sign to: Match sign: Case `None` or `Pos`: `Pos`; Case `Neg`: `Neg`.
    - Error unless `signs` includes `Pos`.
  - Else:
    - If sign count > 1: Error unless `signs` includes `Repeated`.
    - Break.
  - Increment sign count.
  - If strip whitespace: Error unless `space_positions` includes `AfterSign`.

**Base prefix**

- Initialize base style to `None`.
- Initialize marked base to `None`.
- If bump "0b" or "0B": Set base style to "0b" or "0B" prefix. Set marked base to 2.
- Else if bump "0o" or "OO": Set base style to "0o" or "OO" prefix. Set marked base to 8.
- Else if bump "0x" or "0X": Set base style to "0x" or "0X" prefix. Set marked base to 16.

**Digits**

- Initialize digits array to empty.
- Initialize inferred base to 0.
- Initialize last was digit sep to false.
- Initialize digit sep after octal zero to false.
- Initialize first letter pos to `None`.
- While not EOF:
  - Initialize ch to bump
  - Match ch:
    - Case '0':
    - Case '1': Set inferred base to max of itself and 2.
    - Case '2' to '7': Set inferred base to max of itself and 8.
    - Case '8' to '9': Set inferred base to max of itself and 10.
    - Case 'a' to 'f' or 'A' to 'F':
      - If ch is 'b' or 'B'
        and base style is `None` and inferred base <= 2
        and peek is not any of '0' to '9', 'a' to 'z', 'A' to 'Z', '_', or '\'':
        - Set base style to "b" or "B" suffix. Set marked base to 2.
        - Break.
      - Else:
        - Set inferred base to max of itself and 16.
        - If first letter pos is `None`: Set first letter pos to current pos - 1.
    - Case '_' or '\'':
      - If ch is '_': Error unless `digit_sep` includes `Underscore`.
      - If ch is '\'': Error unless `digit_sep` includes `SingleQuote`.
      - If digits array is empty:
        - If base style is `None`: Error.
        - Else: Error unless `digit_sep_positions` includes `AfterBasePrefix`.
      - Else if `leading_zero_octal` in config and digits array is [0]:
        - Set digit sep after octal zero to true.
      - If last was digit sep: Error unless `digit_sep_positions` includes `Repeated`.
      - Continue.
    - Default:
      - Backtrack 1. Break.
  - Push ch to digits array.
- If last was digit sep and digits is not empty:
  - Error unless `digit_sep_positions` includes `Trailing`.

**Base suffix**

- If base style is `None`
  and if peek is 'o', 'O', 'h', or 'H'
  and peek after is not any of '0' to '9', 'a' to 'z', 'A' to 'Z', '_', or '\'':
  - Bump.
  - Set base style to "o", "O", "h", or "H" suffix. Set marked base to 8 or 16.
- Error unless `base_styles` includes base style.

**Fused mnemonic**

- If `enable_fused_mnemonics` in config
  and base style is `None`
  and peek is 'a' to 'z' or 'A' to 'Z':
  - Backtrack to first letter pos. Pop excess from digits array.
  - Adjust inferred base to 2, 8, or 10.

**Leading zero octal**

- If `leading_zero_octal` in config
  and base style is `None`
  and first element of digits array is 0:
  - If digits array is [0]: Set marked base to 10.
  - Else:
    - Pop front of digits array.
    - Set base style to '0' prefix. Set marked base to 8.
    - If digit sep after octal zero: Error unless `digit_sep_positions` includes `AfterOctalLeadingZero`.

**Suffix and empty prefix conflict**

- If digits array is empty:
  - If base style is "0b" prefix and `base_styles` contains "b" suffix
    or base style is "0B" prefix and `base_styles` contains "B" suffix
    or base style is "0o" prefix and `base_styles` contains "o" suffix
    or base style is "0O" prefix and `base_styles` contains "O" suffix:
    - Set base style to "b", "B", "o", or "O" suffix. Set marked base to 2 or 8.
    - Set digits array to [0].

**Validate base**

- If marked base < inferred base:
  - Error with out of range digit.

**Close parens**

- Initialize close count to 0.
- If `enable_parens` in config:
  - While close count < open count:
    - If strip whitespace: Error unless `space_positions` includes `BetweenParens`.
    - If bump ')': Increment close count.
    - Else: Error with unclosed parens. Break.

**Trailing whitespace**

- If strip whitespace: Error unless `space_positions` includes `Trailing`.

## Common case algorithm

There should also be a simple algorithm, which handles the common case of
`[-+]?[0-9]+`. If it fails, switch to the full algorithm. If at least one digit
was read, jump to digit reading in the full algorithm, reusing the digit buffer
and leading zero count.

## Fields for `IntegerSyntax`

- `signs: EnumSet<SignStyle>` with `enum SignStyle { Neg, Pos, Repeated }`.
- `base_styles: EnumSet<BaseStyle>` as currently.
- `digit_seps: EnumSet<DigitSep>` with `enum DigitSep { Underscore, SingleQuote }`.
- `digit_sep_positions: EnumSet<DigitSepPosition>` with
  `enum DigitSepPosition { AfterBasePrefix, AfterOctalLeadingZero, Trailing, Repeated }`.
- `parens: bool`. Implies `enable_parens` in the parser config.
- `spaces` configures the whitespace characters allowed. Probably an enum set,
  since everything will use a subset of Unicode whitespace plus NUL.
- `space_positions: EnumSet<SpacePosition>` with
  `enum SpacePosition { Leading, AfterSign, Trailing, BetweenParens }`.

This algorithm only works if `base_styles` includes `Decimal` (i.e.,
unprefixed).

## Parsing config

Only these three options affect the parsing behavior; the rest above determine
which constructs are errors.

- `enable_fused_mnemonics: bool`, when enabled, prefers the shortest valid
  decimal integer interpretation if there is trailing text. This is for
  supporting integers fused to a mnemonic as in wsf.
- `enable_parens: bool` determines whether to parse parens.
- `leading_zero_octal: bool` determines whether a leading '0' denotes octal.

## Extensions

An 'r' (or 'R') suffix would be compatible with all integer syntaxes. It could
denote a "raw" integer, where the leading zeros are significant in the binary
serialization. Add field `raw_suffix: bool` to `IntegerSyntax`.

Other bases up to 62 would require a prefix and could not be inferred.

## Fused mnemonics

Fused mnemonics cannot start with an ASCII digit. A mnemonic that starts with an
ASCII letter can only be used with decimal integers.

Suppose the mnemonic `xchg` can be fused. Then `0xchg` is unclear: it could be
either hex `0xc` fused with `hg` or `0` fused with `xchg`. For simplicity, `0x`
at the start should always be interpreted as a hex prefix. For the latter
behavior, the user should write `0_xchg`, `00xchg`, or possibly `0 xchg`.

Suppose the mnemonic `add` can be fused. It's entirely hex digits, so cannot be
used after a hex integer.

Suppose the mnemonic `div` can be fused. It would be partially read as hex
digits.

## Tests

- "0101b": binary 0b101.
- "0101b123slide": decimal 0101 followed by "b123slide".
- "255add": decimal 255 followed by "add"
- "0xffadd": hex 0xffadd.
- "0xff_add": hex 0xffadd.
- "0xchg": hex 0xc followed by error "hg" (even if fused enabled).
- "0_xchg": decimal 0 followed by "xchg".
- "00xchg": decimal 00 followed by "xchg".

A test system that runs the parser on all possible configurations would be
useful. However, there are around 2^29 combinations of config options, so this
would not be practical. Instead, it could be fuzzed. For each component of the
output and every kind of error, a small routine could be written that would
compute only that, to be used for comparison.
