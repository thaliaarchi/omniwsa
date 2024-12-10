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

## Assembler

The following describes the assembly syntax of the esotope-ws assembler.

### Grammar

```bnf
program ::= (line line_term)* line?
line ::=
    | space* ((label_def space+)? inst junk? | label_def)? space* comment?
inst ::=
    | (?i)"push" space+ int
    | (?i)"dup"
    | (?i)"copy" space+ int
    | (?i)"swap"
    | (?i)"pop"
    # BUG: No argument is taken
    | (?i)"slide"
    | (?i)"add"
    | (?i)"sub"
    | (?i)"mul"
    | (?i)"div"
    | (?i)"mod"
    | (?i)"store"
    | (?i)"retrieve"
    | (?i)"call" space+ label_ref
    | (?i)"jmp" space+ label_ref
    | (?i)"jz" space+ label_ref
    | (?i)"jn" space+ label_ref
    | (?i)"ret"
    | (?i)"halt"
    | (?i)"putchar"
    | (?i)"putint"
    | (?i)"getchar"
    | (?i)"getint"
label_def ::= word? ":"
label_ref ::= word
int ::= ("-" | "+")? [0-9]+
comment ::= ";" [^\n\r]*
# BUG: Ignored text
junk ::= (space+ word)+
word ::= [^ \t\n\v\f\r;]+

line_term ::= "\n" | "\r" | "\r\n"
space ::= " " | "\t" | "\v" | "\f"
```

Mnemonics are compared ASCII case-insensitively.

esotope-ws operates on `str` objects, which use Latin-1, not `unicode` objects.
No UTF-8 decoding or validation is performed.

### Generation

- Zero (including negative zero) is encoded without a sign.
- `slide` is encoded without an argument or its terminating LF.
- Labels are encoded as the bits of the byte text, most significant bit first.

### Bugs in assembler

- `slide` does not take an argument.
- Extra trailing words on a line are ignored.
- Empty labels definitions (`:`) are allowed, but not empty label references.

### Python 2 reference

esotope-ws was created 2004-12-08, so likely was written with [Python 2.3.4](https://www.python.org/downloads/release/python-234/)
(released 2004-05-27) or [Python 2.4.0](https://www.python.org/downloads/release/python-240/)
(released 2004-11-30).

- [`int(s)`](https://github.com/python/cpython/blob/v2.4/Objects/intobject.c#L875)
  with a string argument and no base parses it in base 10 to an int.
- [`string.split`](https://github.com/python/cpython/blob/v2.4/Objects/stringobject.c#L1306)
  with the default separator splits a string into words separated by whitespace
  using [`isspace`](https://en.cppreference.com/w/c/string/byte/isspace) from
  `<ctype.h>`.
- [`string.splitlines`](https://github.com/python/cpython/blob/v2.4/Objects/stringobject.c#L3193):
  splits by LF, CR, and CRLF.
- [`ord`](https://github.com/python/cpython/blob/v2.4/Python/bltinmodule.c#L1226)

## Disassembler

The following describes the assembly syntax of the esotope-ws disassembler.
Esotope also has a function `string_of_node` to format instructions with the
same assembly syntax, but it is unused.

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

TODO: Document disassemblers.
