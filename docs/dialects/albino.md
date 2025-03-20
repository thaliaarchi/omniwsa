# Albino Whitespace assembly

- Source: <https://github.com/faultier/whitebase> (last updated [2014-08-19](https://github.com/faultier/whitebase/commit/d8a72e8090d7650d7fe033dab68a710f8dc5a8f3)),
  <https://github.com/faultier/albino> (last updated [2014-07-26](https://github.com/faultier/albino/commit/3f052809d4215493a2da0a74472af9e54a84ddc0)),
  <https://github.com/wspace/faultier-whitebase> (last updated [2025-03-20](https://github.com/wspace/faultier-whitebase/commit/2305de09b0e55f0ead0a68b6d8bd13575d6b95b4)),
  <https://github.com/wspace/faultier-albino> (last updated [2025-03-20](https://github.com/wspace/faultier-albino/commit/ac0791acec3ae9db6281c57c117cbd1e40c50171))
- Corpus: [rust/faultier-whitebase](https://github.com/wspace/corpus/tree/main/rust/faultier-whitebase),
  [rust/faultier-albino](https://github.com/wspace/corpus/tree/main/rust/faultier-albino)

Albino is an interpreter, assembler, and disassembler for Whitespace by
faultier, which also supports Brainfuck, Ook!, by converting them to Whitespace
instructions, and a Whitespace mapping called DT. It is split into a CLI and a
library, Whitebase. There are no known programs in its dialect.

They were originally written with Rust v0.12.0-pre, but I have updated them to
modern Rust in my forks. There are no differences in the grammar caused by
changes in the standard library in that time and I have not deliberately changed
behavior in my forks. However, CLI behavior and error messages are slightly
different.

## Grammar

The source must be valid UTF-8 without surrogate halves (Rust `String`).

```bnf
program ::= (line "\n")* line?
line ::=
    | inst
    # Must be valid UTF-8.
    | ";" [^\n]*
    | ""

inst ::=
    | "PUSH" number_arg
    | "DUP" none
    | "COPY" number_arg
    | "SWAP" none
    | "DISCARD" none
    | "SLIDE" number_arg
    | "ADD" none
    | "SUB" none
    | "MUL" none
    | "DIV" none
    | "MOD" none
    | "STORE" none
    | "RETRIEVE" none
    | "MARK" number_arg
    | "CALL" number_arg
    | "JUMP" number_arg
    | "JUMPZ" number_arg
    | "JUMPN" number_arg
    | "RETURN" none
    | "EXIT" none
    | "PUTC" none
    | "PUTN" none
    | "GETC" none
    | "GETN" none

number_arg ::= " " i64
i64 ::= ("-" | "+")? [0-9]+
none ::= " "?
```

In Rust v0.12.0, integer arguments are parsed with
[`<i64 as std::from_str::FromStr>::from_str`](https://github.com/rust-lang/rust/blob/0.12.0/src/libstd/num/int_macros.rs#L39),
which defers to [`std::strconv::from_str_common`](https://github.com/rust-lang/rust/blob/0.12.0/src/libstd/num/strconv.rs#L756),
then [`std::strconv::from_str_bytes_common`](https://github.com/rust-lang/rust/blob/0.12.0/src/libstd/num/strconv.rs#L549).
As of Rust 1.85.1, [`<i64 as std::str::FromStr>::from_str`](https://github.com/rust-lang/rust/blob/1.85.1/library/core/src/num/mod.rs#L1399)
still has the same grammar and overflow behavior. The rest of the assembler has
the same behavior between the two versions.

### Semantics

Integer arguments are parsed and stored as `i64`. Overflow while parsing returns
an error.

## Disassembler

```bnf
program ::= (inst "\n")*
inst ::=
    | "PUSH" " " i64
    | "DUP"
    | "COPY" " " i64
    | "SWAP"
    | "DISCARD"
    | "SLIDE" " " i64
    | "ADD"
    | "SUB"
    | "MUL"
    | "DIV"
    | "MOD"
    | "STORE"
    | "RETRIEVE"
    | "MARK" " " i64
    | "CALL" " " i64
    | "JUMP" " " i64
    | "JUMPZ" " " i64
    | "JUMPN" " " i64
    | "RETURN"
    | "EXIT"
    | "PUTC"
    | "PUTN"
    | "GETC"
    | "GETN"
i64 ::= "-"? ("0" | [1-9][0-9]*)
```
