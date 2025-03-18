# CensoredUsername Whitespace assembly

- Source: <https://github.com/CensoredUsername/whitespace-rs>
  (last updated [2025-03-18](https://github.com/CensoredUsername/whitespace-rs/commit/7b856299b98601ff7d6e57a618cfe4c895e1248f))
- Corpus: [rust/censoredusername-whitespace-rs](https://github.com/wspace/corpus/tree/main/rust/censoredusername-whitespace-rs)

The Whitespace assembly dialect of CensoredUsername's whitespace-rs JIT and
assembler.

## Grammar

A program source is first lexed into tokens, dropping space tokens. The source
must be valid UTF-8 without surrogate halves (Rust `String`).

```bnf
token ::=
    | name
    | integer
    | colon
    | comma
    | comment
    | newline
    | space
    | eof

name ::= [a-z A-Z _] [a-z A-Z 0-9 _]*
integer ::= "-"? [0-9]+
colon ::= ":"
comma ::= ","
comment ::= ";" [^\n]*
newline ::= "\n" ("\n" | space)*
space ::= (" " | "\t" | "\f" | "\r")+
eof ::= EOF
```

It is then parsed into labels and ops with comma-separated arguments.

```bnf
program ::= line*
line ::= label* op? comment? (newline+ | eof)

label ::= name colon
op ::= name (arg (comma arg)*)?
arg ::= name | integer
```

Ops are then validated.

```bnf
valid_op ::=
    | "push" integer
    | "dup"
    | "copy" integer
    | "swap"
    | "pop"
    | "slide" integer
    | "add"
    | "sub"
    | "mul"
    | "div"
    | "mod"
    | "set"
    | "get"
    | label
    | "lbl" name
    | "call" name
    | "jmp" name
    | "jz" name
    | "jn" name
    | "ret"
    | "exit"
    | "pchr"
    | "pnum"
    | "ichr"
    | "inum"
```

### Semantics

The argument to `push` has arbitrary precision. The arguments to `copy` and
`slide` are also parsed as arbitrary precision, but are required to fit in Rust
`isize` and be non-negative. Zero (and positive and negative zero) are encoded
with a positive sign space token. Negative zero counts as a non-negative
argument.

Labels are encoded as the ASCII representation of the text with 8 bits per byte
(big-endian), except for labels matching the pattern `_[01]*`, which are encoded
as their binary representation (big-endian). Both styles are minified.

When `--minify` is passed, labels are minified by replacing them with minimal
bit sequences, including leading zeros.

Mnemonics and labels are case-sensitive.

### History

- [2024-12-10](https://github.com/CensoredUsername/whitespace-rs/commit/35d4aa422867f9bd0e4eaf43437deeb0157fab33):
  Encode zero with a positive sign. Before it was encoded without a sign.
- [2024-12-10](https://github.com/CensoredUsername/whitespace-rs/commit/3ad9036a4cf17bd578f38ac0aca3fff30b316689):
  Recognize whitespace characters with `char::is_ascii_whitespace`. Before only
  space and tab were whitespace.
- [2024-12-12](https://github.com/CensoredUsername/whitespace-rs/commit/f7d1fe1d995358924952ec7e56346f42add4a6e0):
  Encode labels which match the pattern `_[01]*` as their binary representation
  (big-endian). Before, they were encoded as their ASCII representation.
- [2025-03-18](https://github.com/CensoredUsername/whitespace-rs/commit/7b856299b98601ff7d6e57a618cfe4c895e1248f):
  Assign minified labels deterministically, ordered by number of references and
  breaking ties by earlier definition. Before, labels with the same number of
  references were ordered non-deterministically.
- [2025-03-18](https://github.com/CensoredUsername/whitespace-rs/commit/7b856299b98601ff7d6e57a618cfe4c895e1248f):
  Minify labels with big-endian bit order. Before, they were little-endian.

## Disassembler

The disassembler prints instructions with 4-space indentation. Opcodes that take
an argument are right-padded with spaces to match the width of the longest
mnemonic (`slide`), so that all arguments start at column 11. Labels are printed
as ASCII if the representation consists of 8-bit bytes in `[a-zA-Z_]`; otherwise
as `_` followed by binary digits. Labels use colon syntax and are not indented.
Every line is terminated with LF.

### History

- [2025-03-18](https://github.com/CensoredUsername/whitespace-rs/commit/c2096bfd332d6cd28cc33e0e9b94c61b75e77d7f):
  Disassemble labels as ASCII which match the pattern `[a-zA-Z_][a-zA-Z0-9_]*`.
  Before, the pattern was `[a-zA-Z_]+`.
