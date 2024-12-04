# Configurable dynamic lexing and parsing

In order to dynamically define dialects, lexing and parsing need to be
dynamically configurable.

Currently, string lexing is somewhat generic with `Scanner::string_lit_oneline`
and similarly for chars, and integer parsing is dynamically configurable with
`IntegerSyntax`. If extended to all tokens and composed with combinators, lexing
would be dynamically generic.

Parsing has more complex error recovery and has a few different styles, that
most dialects should fit into. Each dialect would pick one style, which is
implemented generic over lexing. Styles would include:
punctuation-delimited/line-terminated instructions and space-delimited
instructions.

Identifier first and follow sets vary, so need to be dynamically encoded. For
ASCII-only a 128-bit bitset would work and for Unicode a perfect hash table
would work. An ASCII-to-char table could be a perfect hash table or a 128-bit
bitset with a dense table indexed by popcnt. At this point, it resembles the
tables produced by parser generators. I want to be able to write wsa grammars at
a higher level than yacc/lex grammars, so that common things are simplified;
however, it could lower to a more general purpose parser generator.
