# Compact concrete syntax tree for Whitespace

A concrete syntax tree for Whitespace needs to support the standard tokens,
comments, errors, extension tokens, in addition to an unbounded number of custom
encodings of those. All of that, and it should be compact.

## Tokens

- Standard tokens: space, tab, line feed
- Comment
- Errors: invalid UTF-8, invalid token (for encodings where invalid tokens are
  not comments)
- Extension tokens: GrassMudHorse river crab (in addition to standard), szm
  tokens (instead of standard)

## Token source

For lexing and later reference in the CST, a sequence of tokens in the source
should be maintained.

Let's try to fit the token source into a byte array. In the common case, there
are six tokens: space, tab, line feed, comment, invalid UTF-8, and invalid
token. This fits into 3 bits, leaving space for 2 more variants. One of those
could denote that it is some extension token and the other can remain reserved
for now. That leaves 5 bits, or 32 values, to be used.

Since some token encodings represent tokens with regular expressions, the number
of unique lexemes is infinite, but typically only a few are used. An out of band
indexed table could intern such lexemes. The first 31 lexemes would be
referenced directly and anything else would be marked as 31 with the next 4
bytes being a 32-bit integer for the index.

Comments, on the other hand, are rarely repeated, so should not be interned. A
value of 1 to 31 denotes a short length and a value of 0 denotes that the length
is the next 4 bytes as a 32-bit integer. The text then follows it inline.

Errors need to store the erroneous sequence. This can be stored inline just like
comments.

Extension tokens need to represent both the token kind and the text. Clients
should be able to dynamically construct new token kinds for dialects to use.
Store the lexeme in the 5 bits as in standard tokens. Store the token kind as an
8-bit index in the following byte (before the large lexeme index). 256 possible
custom token kinds seems sufficient, even composing all known extensions.

This byte encoding is not synchronizing like UTF-8 and cannot be iterated in
reverse. Tokens are referenced by their byte offset in the token source.

Spans aren't stored. However, they can be reconstructed on the fly from lengths
like done by incremental compilation systems.

As an aside, token encoding is a better term than token mapping for denoting a
custom lexical format of tokens. A token decoder lexes and a token encoder
serializes to the canonical representation for each token.

## Multiple files

Tokens in a token source all belong to a single dialect. Even with imports,
it would be nonsensical to simply concatenate token sources of different
dialects, as it could not be parsed back. If you're joining them into what is
actually a union of the two dialects, this should be intentionally handled.

## Syntax tree

The CST references tokens in the token source by their byte offset within the
sequence.

After lexing into a token source, most dialects parse tokens with a prefix tree
encoding. Since dialects extensibly define their own token kinds, the prefix
tree parser should be fully dynamic; this would even work for szm (java/azige),
which uses its own tokens and prefix tree.
