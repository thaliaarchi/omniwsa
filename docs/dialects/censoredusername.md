# CensoredUsername Whitespace assembly

- Source: <https://github.com/CensoredUsername/whitespace-rs>
  (last updated [2024-04-24](https://github.com/CensoredUsername/whitespace-rs/commit/f52bd3d27f8dd2094d700c5f7ae0e8880c5fdc79))
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
newline ::= "\n" ("\n" | " " | "\t")*
space ::= (" " | "\t")+
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
`isize` and be non-negative. Negative zero generates equivalent code to zero and
counts as non-negative. Positive zero generates no sign space token.

Mnemonics and labels are case-sensitive.

## Disassembler

The disassembler prints instructions with 4-space indentation, except for
labels. Opcodes that take an argument are right-padded with spaces to match the
width of the longest mnemonic (`slide`), so that all arguments start at column
11. Labels are printed with colon syntax, using the identifier from assembly, if
available, otherwise `_` followed by the integer value. Each line is terminated
with LF.
