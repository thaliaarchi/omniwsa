# Esotope Whitespace assembly

- Source: <https://github.com/wspace/lifthrasiir-esotope-ws>
  (last updated [2004-12-08](https://github.com/wspace/lifthrasiir-esotope-ws/commit/f2108f9d835924aad912c2f1dbc05dce64abb01c)),
  <https://github.com/wspace/lifthrasiir-esotope>
  (last updated [2012-05-31](https://github.com/wspace/lifthrasiir-esotope/commit/b444a519fcceab24d733232af913a06d419d7427))
- Corpus: [python/lifthrasiir-esotope-ws](https://github.com/wspace/corpus/tree/main/python/lifthrasiir-esotope-ws),
  [ocaml/lifthrasiir-esotope](https://github.com/wspace/corpus/tree/main/ocaml/lifthrasiir-esotope)

Esotope is the name of [several](https://archive.softwareheritage.org/browse/search/?q=https://bitbucket.org/lifthrasiir/esotope)
esolangs projects by [Kang Seonghoon (“lifthrasiir”)](https://github.com/lifthrasiir),
including two for Whitespace: [esotope-ws](https://github.com/wspace/lifthrasiir-esotope-ws)
(2004), an obfuscated Whitespace interpreter, assembler, and disassembler, and
[Esotope](https://github.com/wspace/lifthrasiir-esotope) (2009), which
implements several esolangs including a Whitespace interpreter. esotope-ws is an
early assembler.

The following describes the assembly syntax of esotope-ws. Esotope also has a
function `string_of_node` to format instructions with the same assembly syntax,
but it is unused.

## Grammar

```bnf
inst ::=
    | "push"
    | "dup"
    | "copy"
    | "swap"
    | "pop"
    | "slide"
    | "add"
    | "sub"
    | "mul"
    | "div"
    | "mod"
    | "store"
    | "retrieve"
    | [^;]* ":"
    | "call"
    | "jmp"
    | "jz"
    | "jn"
    | "ret"
    | "halt"
    | "putchar"
    | "putint"
    | "getchar"
    | "getint"
comment ::= ";" …*

space ::= …
isspace ::= " " | "\t" | "\n" | "\v" | "\f" | "\r"
iswspace ::=
  | U+0009..U+000D | U+0020 | U+1680 | U+180E | U+2000..U+2006
  | U+2008..U+200A | U+2028 | U+2029 | U+205F | U+3000
_PyUnicode_IsWhitespace :=
  | U+0009..U+000D | U+001C..U+001F | U+0020 | U+0085 | U+00A0 | U+1680
  | U+2000..U+200B | U+2028 | U+2029 | U+202F | U+205F | U+3000
```

Mnemonics are compared case-insensitively.

esotope-ws was created 2004-12-08, so likely was written with [Python 2.3.4](https://www.python.org/downloads/release/python-234/)
(released 2004-05-27) or [Python 2.4.0](https://www.python.org/downloads/release/python-240/)
(released 2004-11-30).

[`string.split`](https://github.com/python/cpython/blob/v2.4/Objects/stringobject.c#L1306)
with the default separator splits a string into words separated by whitespace
using [`isspace`](https://en.cppreference.com/w/c/string/byte/isspace) from
`<ctype.h>`. [`unicode.split`](https://github.com/python/cpython/blob/v2.4/Objects/unicodeobject.c#L4225)
splits likewise, but using [`Py_UNICODE_ISSPACE`](https://github.com/python/cpython/blob/v2.4/Include/unicodeobject.h#L303-L326),
which is either [`iswspace`](https://en.cppreference.com/w/c/string/wide/iswspace)
from `<wctype.h>` or [`_PyUnicode_IsWhitespace`](https://github.com/python/cpython/blob/v2.4/Objects/unicodectype.c#L327-L335).
`_PyUnicode_IsWhitespace` considers Unicode characters with the bidirectional
type WS, B, or, S or the category Zs to be whitespace.

[`ord`](https://github.com/python/cpython/blob/v2.4/Python/bltinmodule.c#L1226).

### TODO

- Is `string` or `unicode` used?
- What line separators does string `splitlines` consider?
- What whitespace does string `split` consider?
- What case folding does string `tolower` use?
- What about NUL? `string.split` seems to allow it as a character.
- How are encoding errors handled?
- How is negative zero handled?
- Does `ord` in label generation use bytes or codepoints or code units? If not
  bytes, then bits are not included in generation.

### Generation

Zero is encoded without a sign. Labels are encoded as the bits of the text, most
significant bit first.

## Disassembler

```bnf
inst ::=
    | "push"
    | "dup"
    | "copy"
    | "swap"
    | "pop"
    | "slide"
    | "add"
    | "sub"
    | "mul"
    | "div"
    | "mod"
    | "store"
    | "retrieve"
    | … ":"
    | "label" … "_" …
    | "call"
    | "jmp"
    | "jz"
    | "jn"
    | "ret"
    | "halt"
    | "putchar"
    | "putint"
    | "getchar"
    | "getint"
```
