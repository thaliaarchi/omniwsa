# Lime Whitespace assembly

- Source: <https://github.com/ManaRice/whitespace>,
  [[docs](https://github.com/ManaRice/whitespace/blob/master/ws/wsa/README.md)]
  (last updated [2022-05-30](https://github.com/ManaRice/whitespace/commit/e8db8719e170c12875dac571c39ac811c7d0ec52))
- Corpus: [c/manarice-lime](https://github.com/wspacze/corpus/tree/main/c/manarice-lime)

## Grammar

```bnf
program ::= space? (inst space)* inst? space?
inst ::=
    | ("PUSH" | "push") space number
    | "DUPE" | "dupe" | "DUP" | "dup"
    | ("COPY" | "copy") space number
    | "SWAP" | "swap"
    | "DROP" | "drop"
    | ("SLIDE" | "slide") space number
    | "ADD" | "add"
    | "SUB" | "sub"
    | "MUL" | "mul"
    | "DIV" | "div"
    | "MOD" | "mod"
    | "STORE" | "store"
    | "FETCH" | "fetch" | "RETRIEVE" | "retrieve"
    | label_def ":"
    | ("CALL" | "call") space label_ref
    | ("JMP" | "jmp") space label_ref
    | ("JZ" | "jz") space label_ref
    | ("JN" | "jn") space label_ref
    | "RET" | "ret"
    | "END" | "end"
    | "PRINTC" | "printc"
    | "PRINTI" | "printi"
    | "READC" | "readc"
    | "READI" | "readi"
    | ("MACRO" | "macro")
        space word
        space? "["
        (space? (word | label_ref | number)
                (space (word | label_ref | number))*)?
        space? "]"
    | word

word ::= [^ \t\n.:;\[\]*/\\'"#$-]+
label_def ::= "." word
label_ref ::= "."? word
number ::=
    | [0-9]{1,64}
    | "-" [0-9]{0,63}
    | "0x" [0-9a-fA-F]{1,64}
    | "'" ([^\\] | \\[nt] | \\.) "'"
space ::= ([ \t\n] | comment)+
comment ::=
    | "//" .*? "\n"
    | "/*" .*? "*/"
    | ";" .*? "\n"
```

The pattern `.` includes `\n` here.

## Parsing

- Byte-oriented.
- Number tokens are limited to 64 bytes.
- Labels and macro names cannot be mnemonics.
- `'\n'` and `'\t'` escapes are handled, while any other characters are used
  unchanged.

TODO: Check that EOF and NUL are handled in all cases.

## Generation

- Labels are encoded as signed integers incrementing from 0x4a00 in definition
  order and are limited to 64 bits.
- `push 0` and `push -0` are encoded as `SS SSL`.
- It prepends the shebang `#!lwsvm`.
- Redeclared labels or macros are forbidden.

## Notes

- The mnemonics are inspired by Whitelips IDE.
- Numbers and labels are stored as signed 64-bit integers.

## Bugs in the assembler

- NUL truncates the file.
- Minus without digits is allowed and is `0`.
- Negative hex numbers and a `0X` prefix are not handled.
- Integer values outside the range of `int64_t` are saturated.
- A label may be used in place of a number and vice versa. They do not work in
  these incorrect positions.
- Label references do not need to start with `.`, while definitions require it,
  so such labels always fail from referencing a non-existent definition.
- Space is optional between tokens, when either of the adjacent bytes is one of
  `.` `:` `;` `[` `]` `*` `/` `\` `'` `"` `#` `$` or `-`. This means that space
  isn't required before or after a label definition, after a macro definition,
  before a negative number, or before or after a char.
- Characters `*` `\` `"` `#` and `$` are treated as special, but unused.
  However, this probably only impacts error messages.
- Unterminated block comments are allowed.
- Macros without a closing `]` are not handled.
- Open `[` in a macro list is not handled.

## Disassembler

The functions `print_token` in `inc/common.h` and `print_op` in `wsa.c` both
identically disassemble a Whitespace instruction and were supposedly used to
generate several of the programs.

It uses the following mnemonics: `PUSH`, `DUPE`, `COPY`, `SWAP`, `DROP`,
`SLIDE`, `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `STORE`, `FETCH`, `CALL`, `JMP`,
`JZ`, `JN`, `RET`, `END`, `PRINTC`, `PRINTI`, `READC`, and `READI`. Instructions
are indented with 4 spaces. Label definitions are printed with a `:` and are not
indented. It uses LF line terminators. Labels are formatted as hex integers
without leading zeros, prefixed with `.`.
