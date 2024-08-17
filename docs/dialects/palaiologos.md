# Palaiologos Whitespace assembly

- Source: [code](https://github.com/kspalaiologos/asm2ws)
  (last updated [2022-01-24](https://github.com/kspalaiologos/asm2ws/tree/92e33991c5465ec108206db1f028816d3d1e64d6))
- Corpus: [c/kspalaiologos-asm2ws](https://github.com/wspace/corpus/blob/main/c/kspalaiologos-asm2ws/project.json)

## Grammar

```bnf
program ::= lf* (inst lf | lbl lf*)*
inst ::=
    | psh arg?
    | arg
    | dup
    | copy arg
    | xchg
    | drop
    | slide arg
    | add arg?
    | sub arg?
    | mul arg?
    | div arg?
    | mod arg?
    | sto (arg (comma arg)?)?
    | rcl arg?
    | call arg
    | jmp arg
    | jz arg
    | jltz arg
    | ret
    | end
    | putc arg?
    | putn arg?
    | getc arg?
    | getn arg?
    | rep dup arg
    | rep drop arg
    | rep add arg
    | rep sub arg
    | rep mul arg
    | rep div arg
    | rep mod arg
    | rep putn arg
arg ::= number | char | ref
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
- `rep putn` -> `putn` repeated `n` times

Instructions prefixed with `rep` are repeated as many times as specified. The
repeated instruction does not take a constant argument.

Double-`xchg` replacement is always done, regardless of the optimization level
with `-Os`, `-Of`, or neither, and is the only optimization.

### Labels

Labels are assigned, starting from 1, in order from the most references. The
sort algorithm used (`qsort`) is unstable, so the order of labels with equal
number of references is undefined.

A label defined multiple times is an error. A label defined, but never used, is
not emitted.

Labels may be used in place of an integer arguments and are resolved just as
labels used as label arguments. Likewise, integer literals may be used in place
of label arguments. Since labels can only be defined with label syntax, integers
cannot usefully reference them. Since labels without a definition yield an
error, they cannot be easily used as automatically assigned variables.

Label arguments are emitted with no sign, but labels used as integer arguments
are emitted with a S (positive) sign. Integers used as label arguments are not
emitted with a sign and negative integers cause an infinite loop.

## Notes

- The assembler is run with `./wsi --masm <file>`.
- Byte–oriented.
- NUL does not truncate the file and is just an invalid char.
- A binary number takes precedence over a decimal number juxtaposed with `b`.
- Integers have a precision of `int32_t`. They are parsed without the sign, then
  negated. Integers between -2147483647 and 2147483647 (2^31-1), inclusive, work
  correctly; outside that, they wrap with twos complement.

## Bugs in the assembler

- Mnemonics do not need to be followed by spaces, so, e.g., `repdrop5` is valid.
- Line comments consume a line feed, so an `lf` token is not emitted after.
  Thus, line comments can appear, e.g., between arguments, and cannot be on the
  last line if it is not terminated with LF.
- LF and `/` instruction separators cannot be mixed. The parser should repeat
  the LF token as LF* and LF+, instead of LF? and LF.
  - Consecutive LFs are allowed only without tokens between, including spaces or
    line comments. Thus, blank lines with spaces or line comments produce
    errors.
  - Except for the first line, `/` may not start a line, and except for the last
    line if there is no final LF, `/` may not end a line. Consecutive `/` are
    allowed only without tokens between, including spaces.
- Char token `'\'` is parsed as `'\''`.
- String tokens are unused in the parser.
- Integer, char, and label tokens are interchangeable, but not useful in the
  incorrect places.
- Out-of-range integer parse errors are not reported.
- `push 2147483648` (2^31) and `push -2147483648` (-2^31) both parse as
  -2147483648, which is its own negation. This causes the first loop in
  `asm_gen.c:numeral` to loop infinitely, because the sign is extended on right
  shift.
- Negative integer literals used as a label argument cause the first loop in
  `asm_gen.c:unumeral` to similarly loop infinitely.
