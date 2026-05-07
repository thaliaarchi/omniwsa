# Dynamic parsing

## Scanning

- Albino: line term (LF), whitespace (all), line comment (";"), integer, word
- Burghard: custom
- CensoredUsername: line term (LF), whitespace (all), line comment (";"), comma,
  colon (":"), integer, word
- Esotope: line term (LF/CR/CRLF), whitespace (all), line comment (";"), colon
  (":"), integer, word
- Lime: line term (LF), whitespace (all), line comment ("//", ";"), block
  comment ("/*"), char ("'"), colon (":"), bracket ("[", "]"), integer, word
- littleBugHunter: line term (LF/CR/CRLF), whitespace (all), line comment
  ("//"), ampersand ("@"), asterisk ("*"), char ("'"), integer, word
- LukePebody: line term (LF), whitespace (all), line comment ("#"), integer,
  word
- Nossembly: line term (LF), whitespace (all), hash ("#"), number, word
- Iczelia: maybe custom
- rdebath: line term (LF), whitespace (all), line comment (";", "#"), colon
  (":"), integer, word
- voliva: line term (LF), whitespace (all), decoration (";#;"), line comment
  (";"), char ("'"), string ("\""), integer, word
- wconrad: line term (LF), whitespace (all), line comment ("#"), integer, word
- Whitelips: line term (LF), whitespace (all), line comment (";", "#", "--"),
  block comment ("{-"), string ("\"", "'"), colon (":"), integer, word
- wsf: line term (LF), whitespace (all), line comment ("#"), char ("'"), string
  ("\""), word

Since it's usually unintuitive for most symbols to be allowed in an identifier
and some dialects use them for macros or other syntax, such symbols should be
error tokens.

Strings should be lexed even in dialects which do not support them, unless
quotes are valid in words.

Burghard has complicated comment precedence which does not compose well, so
probably should be manually lexed. Iczelia doesn't require spaces between words,
so might be more difficult to do more generally. wsf integer prefixes and
suffixes suggest that those should probably be handled after scanning into
words.

## Lexing

Perhaps integers should be resolved here, instead of while scanning.

## Parsing

Delimited by lines, spaces, or delimiters.

## Validation

Validate that the CST is valid for the dialect, since it over-accepts.
