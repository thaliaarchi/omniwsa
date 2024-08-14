# Palaiologos assembly

- Source: [code](https://github.com/kspalaiologos/asm2ws)
  (last updated [2022-01-24](https://github.com/kspalaiologos/asm2ws/tree/92e33991c5465ec108206db1f028816d3d1e64d6))
- Corpus: [c/kspalaiologos-asm2ws](https://github.com/wspace/corpus/blob/main/c/kspalaiologos-asm2ws/project.json)

## Grammar

```bnf
program ::= lf? (inst lf | lbl lf?)*
inst ::=
    | psh numeric_const?
    | numeric_const
    | dup
    | copy numeric_const
    | xchg
    | drop
    | slide numeric_const
    | add numeric_const?
    | sub numeric_const?
    | mul numeric_const?
    | div numeric_const?
    | mod numeric_const?
    | sto (numeric_const (comma numeric_const)?)?
    | rcl numeric_const?
    | call numeric_const
    | jmp numeric_const
    | jz numeric_const
    | jltz numeric_const
    | ret
    | end
    | putc numeric_const?
    | putn numeric_const?
    | getc numeric_const?
    | getn numeric_const?
    | rep dup numeric_const
    | rep drop numeric_const
    | rep add numeric_const
    | rep sub numeric_const
    | rep mul numeric_const
    | rep div numeric_const
    | rep mod numeric_const
    | rep putn numeric_const
numeric_const ::= number | char | ref
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

lbl ::= "@" [a-zA-Z_] [a-zA-Z0-9_]*
ref ::= "%" [a-zA-Z_] [a-zA-Z0-9_]*
number ::=
    | "-"? [0-9]+
    | "-"? [01]+ [bB]
    | "-"? [0-9] [0-9a-fA-F]* [hH]
char ::= "'" "\\"? . "'"
string ::= "\"" ([^\\] | \\.)* "\""
comma ::= ","
lf ::= "\n"+ | "/"+
```

Ignored:

```bnf
comment ::= ";" .+? "\n"
space ::= [ \t\r\f]
```

TODO: Does `.` in the Flex pattern `";"(.*)\n` include LF?

## Mnemonics

The first mnemonic in the grammar listed for each instruction is the name of the
corresponding token in `asm.y`.

Token and AST names correspond, except for `end`, which is the token `END` and
the AST kind `STOP`.

## Generation

- Instructions prefixed with `rep` are repeated as many times as specified.
- Bare `push` is `push 0`.
- `xchg / xchg` is replaced with nothing. This is always done and is the only
  optimization, regardless of the optimization level with `-Os` or `-Of` or
  neither.

TODO: Labels

## Notes

- The assembler is run with `./wsi --masm <file>`.
- Byte–oriented.
- NUL does not truncate the file and is just an invalid char.
- A binary number takes precedence over a decimal number juxtaposed with `b`.

## Bugs in the assembler

- Mnemonics do not need to be followed by spaces, so, e.g., `repdrop5` is valid.
- A comment may not be terminated with EOF.
- Comments consume `\n`, rather than emit an `lf` token.
- `lf` pattern should be `[\n/]+` to allow consecutive `\n` and `/`.
- Char token `'\'` is parsed as `'\''`.
- String tokens are unused in the parser.
