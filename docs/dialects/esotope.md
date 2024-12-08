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
```

Mnemonics are compared case-insensitively.

### TODO

- What line separators does string `splitlines` consider?
- What whitespace does string `split` consider?
- What case folding does string `tolower` use?

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
