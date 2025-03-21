# LukePebody Whitespace assembly dialect

- Source: <https://github.com/wspace/lukepebody-advent-of-code>
  (last updated [2022-12-14](https://github.com/wspace/lukepebody-advent-of-code/commit/5843192a5d1b33f91d1fd9072a85271cab0994b8))
- Corpus: [whitespace/lukepebody-advent-of-code](https://github.com/wspacze/corpus/tree/main/c/manarice-lime)

A Whitespace assembly dialect created by Luke Pebody for Advent of Code 2022
solutions in Whitespace.

## Grammar

```bnf
program ::= (line "\n")* line?
line ::= space* inst? space* line_comment?

inst ::=
    | "PUSH" arg_no_space
    | "DUP"
    | "GRAB" arg_no_space
    | "SWAP"
    | "DROP"
    | "ADD"
    | "SUB"
    | "MUL"
    | "DIV"
    | "MOD"
    | "STORE"
    | "RETRIEVE"
    | "LABEL" arg_no_space
    | "GOSUB" arg_space
    | "GOTO" arg_space
    | "GOTOIFZERO" arg_space
    | "GOTOIFNEG" arg_space
    | "RETURN"
    | "END"
    | "PRINTC"
    | "PRINTN"
    | "READC"
    | "READN"
    | "SPLURGE"

line_comment ::= "#" [^\n]*

# BUG: Anything between the mnemonic and the int is ignored.
# BUG: The space is inconsistent with other whitespace characters.
arg_space ::= " " space* (not_space+ space+)* int
# BUG: Anything between the mnemonic and the int is ignored.
# BUG: Should require a space after the mnemonic.
arg_no_space ::= (not_space* space+)+ int

int ::= TODO

space ::=
    | U+0009 | U+000B | U+000C | U+000D | U+001C | U+001D | U+001E | U+001F
    | U+0020 | U+0085 | U+00A0 | U+1680 | U+2000 | U+2001 | U+2002 | U+2003
    | U+2004 | U+2005 | U+2006 | U+2007 | U+2008 | U+2009 | U+200A | U+2028
    | U+2029 | U+202F | U+205F | U+3000
not_space ::= NOT space
```

TODO: Python `str.split()`, `str.strip()`, `int(str)`

Duplicate label definitions (by parsed number) are an error.

`SPLURGE` is an assembly-only extension instruction with no Whitespace
equivalent, which dumps the stack and memory.
