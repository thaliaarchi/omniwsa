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
    | label ":"
    | ("CALL" | "call") space label
    | ("JMP" | "jmp") space label
    | ("JZ" | "jz") space label
    | ("JN" | "jn") space label
    | "RET" | "ret"
    | "END" | "end"
    | "PRINTC" | "printc"
    | "PRINTI" | "printi"
    | "READC" | "readc"
    | "READI" | "readi"
    | ("MACRO" | "macro")
        space word
        space? "["
        (space? (word | label | number)
                (space (word | label | number))*)?
        space? "]"
    | word

word ::= [^ \t\n.:;\[\]*/\\'"#$-]+
label ::= "." word
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
- Numbers are limited to 64 digits when parsing and encoding.
- Labels and macro names cannot be mnemonics.
- `'\n'` and `'\t'` escapes are handled, while any other characters are used
  unchanged.

## Generation

- Labels are encoded as signed integers incrementing from 0x4a00 in definition
  order and are limited to 64 bits.
- `push 0` and `push -0` are encoded as `SS SSL`.
- It prepends the shebang `#!lwsvm`.

## Notes

- The mnemonics are inspired by Whitelips IDE.
- Numbers and labels are stored as signed 64-bit integers.

## Bugs in the assembler

- The first digit of a number and the first byte of a label may be any value.
- Negative hex numbers and a `0X` prefix are not handled.
- Line comments can't end with EOF.
- A label may be used in place of a number.
- Space is optional between tokens, when either is one of `.` `:` `;` `[` `]`
  `*` `/` `\` `'` `"` `#` `$` `-` or `` ` ``. This means that space isn't
  required before or after a label definition, after a macro definition, before
  a negative number, or before or after a char.
