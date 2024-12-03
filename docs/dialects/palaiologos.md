# Palaiologos Whitespace assembly

- Source: <https://github.com/kspalaiologos/asm2ws>
  (last updated [2024-08-21](https://github.com/kspalaiologos/asm2ws/commit/89054a73a8ac3766f222a1b2438c63b64952a445))
- Corpus: [c/kspalaiologos-asm2ws](https://github.com/wspace/corpus/tree/main/c/kspalaiologos-asm2ws)

## Grammar

```bnf
program ::= (line lf)* line?
line ::= ((lbl_def slash? | inst slash)* (lbl_def | inst))?

inst ::=
    | psh numeric?
    | numeric
    | dup
    | copy numeric
    | xchg
    | drop
    | slide numeric
    | add numeric?
    | sub numeric?
    | mul numeric?
    | div numeric?
    | mod numeric?
    | sto (numeric (comma numeric)?)?
    | rcl numeric?
    | call lbl_ref
    | jmp lbl_ref
    | jz lbl_ref
    | jltz lbl_ref
    | ret
    | end
    | putc numeric?
    | putn numeric?
    | getc numeric?
    | getn numeric?
    | rep dup numeric
    | rep drop numeric
    | rep add numeric
    | rep sub numeric
    | rep mul numeric
    | rep div numeric
    | rep mod numeric
    | rep putc numeric
    | rep putn numeric
numeric ::= integer | char
```

Tokens:

```bnf
psh   :== (?i)("psh" | "push")
dup   :== (?i)"dup"
copy  :== (?i)("copy" | "take" | "pull")
xchg  :== (?i)("xchg" | "swp" | "swap")
drop  :== (?i)("drop" | "dsc")
slide :== (?i)"slide"
add   :== (?i)"add"
sub   :== (?i)"sub"
mul   :== (?i)"mul"
div   :== (?i)"div"
mod   :== (?i)"mod"
sto   :== (?i)"sto"
rcl   :== (?i)"rcl"
call  :== (?i)("call" | "gosub" | "jsr")
jmp   :== (?i)("jmp" | "j" | "b")
jz    :== (?i)("jz" | "bz")
jltz  :== (?i)("jltz" | "bltz")
ret   :== (?i)"ret"
end   :== (?i)"end"
putc  ::= (?i)"putc"
putn  ::= (?i)"putn"
getc  ::= (?i)"getc"
getn  ::= (?i)"getn"
rep   ::= (?i)"rep"

lbl_def ::= "@" [a-z A-Z _] [a-z A-Z 0-9 _]*
lbl_ref ::= "%" [a-z A-Z _] [a-z A-Z 0-9 _]*
integer ::=
    | "-"? [0-9]+
    | "-"? [01]+ [bB]
    | "-"? [0-7]+ [oO]
    | "-"? [0-9] [0-9 a-f A-F]* [hH]
char ::= "'" ([^\\'\n] | \\[^\n]) "'"
string ::= "\"" ([^\\"\n] | \\[^\n])* "\""
comma ::= ","
lf ::= "\n"
slash ::= "/"
```

Ignored:

```bnf
comment ::= ";" [^\n]*
space ::= [ \t\r\f]
```

Chars can have `\a`, `\b`, `\f`, `\n`, `\r`, `\t`, and `\v` escape sequences as
in C. All other escapes have the value of the char after the slash.

## Mnemonics

The first mnemonic in the grammar listed for each instruction is the name of the
corresponding token in `asm.y`.

Token and AST names correspond, except for `end`, which is the token `END` and
the AST kind `STOP`.

## Generation

- `push` => `push 0`
- `xchg / xchg` => nothing
- `add n` -> `push n / add`
- `sub n` -> `push n / sub`
- `mul n` -> `push n / mul`
- `div n` -> `push n / div`
- `mod n` -> `push n / mod`
- `store n` => `push n / store`
- `store x, y` => `push y / push x / store`
- `rcl n` -> `push n / rcl`
- `putc n` -> `push n / putc`
- `putn n` -> `push n / putn`
- `getc n` -> `push n / getc`
- `getn n` -> `push n / getn`
- `rep dup` -> `dup` repeated `n` times
- `rep drop` -> `drop` repeated `n` times
- `rep add` -> `add` repeated `n` times
- `rep sub` -> `sub` repeated `n` times
- `rep mul` -> `mul` repeated `n` times
- `rep div` -> `div` repeated `n` times
- `rep mod` -> `mod` repeated `n` times
- `rep putc` -> `putc` repeated `n` times
- `rep putn` -> `putn` repeated `n` times

Instructions prefixed with `rep` are repeated as many times as specified. The
repeated instruction does not take a constant argument.

Double-`xchg` replacement is always done, regardless of the optimization level
with `-Os`, `-Of`, or neither, and is the only optimization.

### Labels

Labels are assigned, starting from 0, in order from the most references. Ties
are broken by the earlier first occurrence.

A label defined multiple times is an error. A label defined, but never used, is
not emitted. Label arguments are emitted with no sign.

## Notes

- The assembler is run with `./wsi --masm <file>`.
- Byte–oriented.
- Integers have a precision of `int32_t` and values outside this range yield a
  parse error.
- Mnemonics do not need to be followed by spaces, so, e.g., `repdrop5` is valid.
- A binary number takes precedence over a decimal number juxtaposed with `b`.
- String tokens are unused in the parser.
- NUL does not truncate the file and is just an invalid char.
