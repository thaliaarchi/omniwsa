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

## Tagged bytes

Let's try to fit the CST into a byte array. In the common case, there are six
tokens: space, tab, line feed, comment, invalid UTF-8, and invalid token. This
fits into 3 bits, leaving space for 2 more variants. One of those could denote
that it is some extension token and the other can remain reserved for now. That
leaves 5 bits, or 32 values, to be used.

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
should be able to dynamically construct new token kinds for dialects to use. The
5 bits would be the short token kind index. The following byte would be the
short lexeme index. If the short token kind is 31, the next 4 bytes after that
are the 32-bit token kind index. If the short lexeme index is 255, the next 4
bytes after that are the 32-bit interned lexeme index. If both the token kind
and lexeme index are long, the long token kind precedes the long lexeme index;
both follow the short lexeme index byte. The slightly larger space for the
inline lexeme index compensates for cramming all extension tokens into one
variant.

This byte encoding is not synchronizing like UTF-8.

Spans aren't stored. However, they can be reconstructed on the fly from lengths
like done by incremental compilation systems.

As an aside, token encoding is a better term than token mapping for denoting a
custom lexical format of tokens. A token decoder lexes and a token encoder
serializes to the canonical representation for each token. Layered onto that is
a prefix tree decoder for most dialects; this would even work for szm
(java/azige).
