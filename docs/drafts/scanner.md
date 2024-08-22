# Conventionally UTF-8 scanner

Create a scanner for processing conventionally UTF-8 inputs, which can be used
by UTF-8 and bytes clients. This will unify `Utf8Scanner` and `ByteScanner` into
one concrete type.

## UTF-8 clients

For UTF-8 clients, this will allow continuing scanning after the first error.
This introduces an error status to each character, which was previously added up
a level by the lexer. This has a risk of being easy to misuse if a sentinel char
like � (U+FFFD, REPLACEMENT CHARACTER) were substituted, similarly to NUL
terminators in C. A safe API should not be cumbersome.

## Bytes clients

For bytes clients, this will give better line/col positions.

For the Palaiologos dialect, this only impacts char literals, which still need
to be effectively bytes. When a char is encoded as more than byte, emit
CharData::Char with a CharError::NotByte. When a single non-ASCII byte is in
quotes, accept it. String literals remain StringData::Bytes. When a client

A bytes client likely wants to deal in terms of bytes, but the scanner should
not yield positions at an offset between bytes in a valid multi-byte encoding of
a char. This suggests that external-style iterators might be necessary here, so
that the iterator can keep the line/col in place while iterating the constituent
bytes of a codepoint and the offset.

## API

The API could either be generic over a trait similar to [`Pattern`](https://doc.rust-lang.org/std/str/pattern/trait.Pattern.html),
which would allow matching with a function, byte or char. As a counterpoint,
bstr chose not to follow that design and [uses](https://docs.rs/bstr/latest/bstr/#differences-with-standard-strings)
type-suffixed functions. That might be better, because there's different
handling for each.

Patterns should only be able to match valid UTF-8. This should be fine, because
any parts of a grammar that could match invalid UTF-8 would be anchored by valid
UTF-8. For example, there could be invalid sequences between `"` and `"` in a
string or between `;` and LF in a comment, but the boundaries of those tokens
are valid UTF-8.

`bump_while` does not yield the text, so the error status is attached to the
bumped text as a whole.

An `Ascii` pattern adaptor or `_ascii` function family deals with bytes within
ASCII, so chars do not need to be decoded. For example, `bump_while_ascii` runs
a pattern on ASCII bytes and breaks when it doesn't match or non-ASCII or
invalid UTF-8 is encountered. Likewise, `bump_until_ascii` would be used to
consume until some delimiter and and non-ASCII or invalid UTF-8 would be
accepted, short-circuiting the pattern.

What about truncated UTF-8 sequences? Emit a zero padded codepoint and consume
maximally? Emit each byte individually? I think it's context-dependent.
