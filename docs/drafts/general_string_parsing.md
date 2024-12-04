# Generalized string parsing

Parse strings into sub-tokens of literal sequences, escapes, and invalid
sequences.

```rust
struct StringToken<'s> {
    chunks: Vec<StringChunk<'s>>,
}

enum StringChunk {
    Literal(Cow<'s, [u8]>),
    Escape {
        literal: Cow<'s, [u8]>,
        unescaped: Cow<'s, [u8]>,
        errors: EnumSet<EscapeError>,
    },
    Invalid {
        literal: Cow<'s, [u8]>,
        value: Cow<'s, [u8]>,
    },
}
```
