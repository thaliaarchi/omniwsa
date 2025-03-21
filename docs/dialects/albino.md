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

### Generation

Integers and labels both have a sign. Zero is serialized as SS SSL. Negative
labels are valid.

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

### Generation

When parsing Whitespace or DT syntax, labels are replaced with a number,
incrementing starting at 1, in order of first definition or use. This is not
done when parsing Whitespace assembly.

## Bytecode format

Bytecode instructions are serialized as a byte for the opcode, followed by the
`i64` integer argument encoded as big-endian bytes, if the opcode takes a
parameter. Opcodes have the following byte values, where the high nibble is for
the IMP.

```bnf
PUSH     ::= 0b0011_0011
DUP      ::= 0b0011_0100
COPY     ::= 0b0011_1000
SWAP     ::= 0b0011_0110
DISCARD  ::= 0b0011_0101
SLIDE    ::= 0b0011_1001
ADD      ::= 0b1000_0000
SUB      ::= 0b1000_0010
MUL      ::= 0b1000_0001
DIV      ::= 0b1000_1000
MOD      ::= 0b1000_1010
STORE    ::= 0b1010_0011
RETRIEVE ::= 0b1010_1011
MARK     ::= 0b0111_0000
CALL     ::= 0b0111_0010
JUMP     ::= 0b0111_0001
JUMPZ    ::= 0b0111_1000
JUMPN    ::= 0b0111_1010
RETURN   ::= 0b0111_1001
EXIT     ::= 0b0111_0101
PUTC     ::= 0b1001_0000
PUTN     ::= 0b1001_0010
GETC     ::= 0b1001_1000
GETN     ::= 0b1001_1010
```

## DT mapping

DT is a simple token mapping for Whitespace. Characters which are not used in
the tokens are comments. A token which is broken up with comment chars or is
incomplete is a comment. The entire source must be valid UTF-8. The author
introduced it in a [blog post](https://faultier.blog.jp/archives/1139763.html)
in Japanese.

```bnf
token ::= space | tab | lf | comment
space ::= "ど"
tab   ::= "童貞ちゃうわっ！"
lf    ::= "…"

comment ::=
  | [^ど童…]
  | "童" (?! "貞")
  | "童貞" (?! "ち")
  | "童貞ち" (?! "ゃ")
  | "童貞ちゃ" (?! "う")
  | "童貞ちゃう" (?! "わ")
  | "童貞ちゃうわ" (?! "っ")
  | "童貞ちゃうわっ" (?! "！")
```
