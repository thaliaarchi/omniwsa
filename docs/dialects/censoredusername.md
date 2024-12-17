# CensoredUsername Whitespace assembly

- Source: <https://github.com/CensoredUsername/whitespace-rs>
  (last updated [2024-12-12](https://github.com/CensoredUsername/whitespace-rs/commit/9028eba04b40af4a23f99dac058b3ac06c5967ff))
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

## Semantics

The argument to `push` has arbitrary precision. The arguments to `copy` and
`slide` are also parsed as arbitrary precision, but are required to fit in Rust
`isize` and be non-negative. Zero (and positive and negative zero) are encoded
with a positive sign space token. Negative zero counts as a non-negative
argument.

Labels are encoded as the ASCII representation of the text with 8 bits per byte
(big-endian), except for labels matching the pattern `_[01]*`, which are encoded
as their binary representation (big-endian). Both styles are minified when
`--minify` is passed.

Mnemonics and labels are case-sensitive.

### History

- Until [2024-12-10](https://github.com/CensoredUsername/whitespace-rs/commit/35d4aa422867f9bd0e4eaf43437deeb0157fab33),
  zero was encoded without a sign.
- Until [2024-12-10](https://github.com/CensoredUsername/whitespace-rs/commit/3ad9036a4cf17bd578f38ac0aca3fff30b316689),
  the recognized whitespace characters were only space and tab.

## Disassembler

The disassembler prints instructions with 4-space indentation. Opcodes that take
an argument are right-padded with spaces to match the width of the longest
mnemonic (`slide`), so that all arguments start at column 11. Labels are printed
as ASCII if the representation consists of 8-bit bytes in `[a-zA-Z_]`; otherwise
as `_` followed by binary digits. Labels use colon syntax and are not indented.
Every line is terminated with LF.

### Bugs

- Labels are not disassembled as ASCII if they contain `[0-9]`.
