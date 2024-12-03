# rdebath Whitespace assembly

- Source: <https://github.com/rdebath/whitespace>
  (last updated [2015-08-22](https://github.com/rdebath/whitespace/commit/eeabf4ed507cd1c70d91caf8aeddc5707de6b4b2),
  fork last updated [2023-06-16](https://github.com/wspace/rdebath-c/commit/646b907b0c33292816d95d030f37871cdf65a049))
- Corpus: [c/rdebath](https://github.com/wspace/corpus/tree/main/c/rdebath)

## wsa.l

### Grammar

```bnf
program ::= (line lf)* line?
line ::= space? (label_def space?)? inst? space? comment?

inst ::=
    | "push" number
    | "dup"
    | ("copy" | "pick") number
    | "swap"
    | "drop" | "discard"
    | "slide" number
    | "add"
    | "sub"
    | "mul"
    | "div"
    | "mod"
    | "store"
    | "fetch" | "retrieve" | "retrive" | "retreive"
    | "label" label
    | "call" label
    | ("jmp" | "jump") label
    | "jz" label
    | "jn" label
    | "ret" | "return"
    | "quit" | "exit" | "end"
    | "outc" | "outchar" | "printc"
    | "outn" | "outnum" | "printi"
    | "readc" | "readchar"
    | "readn" | "readnum" | "readi"
label_def ::=
    | [0-9]+ ":"
    | "."? name space? ":"

number ::=
    | space [0-9]+
    | space? "-" [0-9]+
    | space? "'" [^\\] "'"
    | space? "'\\" [ntab'] "'"
label ::=
    | space [0-9]+
    | space name
    | space? "." name
name ::= [a-z A-Z _ $] [a-z A-Z 0-9 _ $]*
comment ::=
    | ";" .*?
    | "#" .*?
space ::= [ \t]+
lf ::= "\n"
```

### Bugs in assembler

- Char literals with unescaped `'` (`'''`) or a literal LF are allowed (fixed in
  fork).

## wsa.sed

### Grammar

```bnf
program ::= (line lf)* line?
line ::= space? inst? space?
inst ::=
    | "push" integer
    | "pushs" string
    | "doub"
    | "swap"
    | "pop"
    | "add" integer?
    | "sub" integer?
    | "mul"
    | "div"
    | "mod"
    | "store" integer?
    | "retrive" integer?
    | "label" label
    | "call" label
    | "jump" label
    | "jumpz" label
    | "jumpn" label
    | "jumppz" label
    | "jumpnz" label
    | "jumpp" label
    | "jumppn" label
    | "jumpnp" label
    | "ret"
    | "exit"
    | (?i)"outC"
    | (?i)"outN"
    | (?i)"InC"
    | (?i)"InN"
    | "test" integer
    | "ifoption" identifier
    | "endoption"
    | "include" identifier
    | "debug_printheap"
    | "debug_printstack"

integer ::= space? "-"? [0-9]+
string ::= space? "\"" [^"]* "\""
identifier ::= space? [a-z A-Z][a-z A-Z 0-9 _]*
label ::= identifier
space ::= [ \t]+
lf ::= "\n"
```

## Generation

- `add n` => `push n / add`
- `sub n` => `push n / sub`
- `store n` => `push n / swap / store`
- `retrive n` => `push n / retrieve`
- `test n` => `dup / push n / sub`

### Notes

- Assembles a near-Burghard dialect.

### Bugs in assembler

- Only the I/O mnemonics are case-insensitive.
- Tabs may only appear before an instruction.

## Mnemonics

The assemblers allow various mnemonics and emit code that's linked with
`ws_gencode.h`. The mnemonics defined therein can be considered the preferred
mnemonics.

Standard instructions:
- `push`
- `dup`
- `pick`
- `swap`
- `drop`
- `slide`
- `add`
- `sub`
- `mul`
- `div`
- `mod`
- `store`
- `fetch`
- `label`
- `call`
- `jump`
- `jz`
- `jn`
- `return`
- `exit`
- `outc`
- `outn`
- `readc`
- `readn`

Extension instructions:
- `pushs` (Burghard `pushs`)
- `jp` (Burghard `jumpp`)
- `jzp` (Burghard `jumppz`)
- `jzn` (Burghard `jumpnz`)
- `jnz` (Burghard `jumpnp`/`jumppn`)
