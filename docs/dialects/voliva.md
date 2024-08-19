# voliva Whitespace assembly

- Source: [code](https://github.com/voliva/wsa)
  (last updated [2024-08-19](https://github.com/voliva/wsa/pull/1) in fork)
- Corpus: [typescript/voliva-wsa](https://github.com/wspace/corpus/blob/main/typescript/voliva-wsa/project.json)

## Grammar

The program is divided into lines and each line is lexed into tokens:

```bnf
program ::= (line "\n")* line?
line ::= (SPACE? token)* SPACE? LINE_COMMENT?
token ::= STRING | CHAR | VARIABLE | INTEGER | WORD

STRING ::= "\"" ([^"\\\n] | \\["\\bfnrtv])* "\""
CHAR ::= "'" ([^'\\\n] | \\['\\bfnrtv]) "'"
VARIABLE ::= "_" WORD
INTEGER ::= BIGINT
WORD ::= [^\s;"']+
LINE_COMMENT ::= ";" [^\n]*
SPACE ::= \s+

# BigInt constructor, without whitespace (ref. https://tc39.es/ecma262/multipage/abstract-operations.html#sec-stringtobigint).
BIGINT ::=
    | ("-" | "+") [0-9]+
    | ("0b" | "0B") [01]+
    | ("0o" | "0O") [0-7]+
    | ("0x" | "0X") [0-9 a-f A-F]+
```

Each line is then parsed as an instruction:

```bnf
instruction ::=
    | (?i)"push" integer
    | (?i)"dup"
    | (?i)"copy" index
    | (?i)"swap"
    | (?i)"pop"
    | (?i)"slide" index
    | (?i)"add" integer?
    | (?i)"sub" integer?
    | (?i)"mul" integer?
    | (?i)"div" integer?
    | (?i)"mod" integer?
    | (?i)"and" integer?
    | (?i)"or" integer?
    | (?i)"not"
    | (?i)"store" integer?
    | (?i)"storestr" string
    | (?i)"retrieve" integer?
    | (?i)"label" label
    | (?i)"call" label
    | (?i)"jump" label
    | (?i)"jumpz" label
    | (?i)"jumpn" label
    | (?i)"jumpp" label
    | (?i)"jumpnz" label
    | (?i)"jumppz" label
    | ((?i)"jumppn" | (?i)"jumpnp") label
    | (?i)"ret"
    | (?i)"exit"
    | (?i)"outn"
    | (?i)"outc"
    | (?i)"readn"
    | (?i)"readc"
    | (?i)"valuestring" variable string
    | (?i)"valueinteger" variable integer
    | (?i)"debugger"
    | (?i)"include" filename

integer ::= INTEGER | CHAR | VARIABLE
string ::= STRING | WORD | VARIABLE
index ::= INTEGER | VARIABLE
label ::= WORD | VARIABLE
variable ::= VARIABLE
filename ::= WORD | VARIABLE | STRING
```

Mnemonics are compared with effectively ASCII case folding.

## Generation

- `add 0` => nothing
- `sub 0` => nothing
- `mul 1` => nothing
- `div 1` => nothing
- `add n` => `push n / add`
- `sub n` => `push n / sub`
- `mul n` => `push n / mul`
- `div n` => `push n / div`
- `mod n` => `push n / mod`
- `store n` => `push n / swap / store`
- `storestr s` => `dup / push c / store / push 1 / add` for each character in
  `s` and 0
- `retrieve n` => `push n / retrieve`
- `jumpp l` => `push 0 / swap / sub / jn l`
- `jumpnz l` => `push 1 / sub / jn l`
- `jumppz l` => `jn __internal_label_{id} / jmp l / __internal_label_{id}:`
- `jumppn l` => `jz __internal_label_{id} / jmp l / __internal_label_{id}:`
- `or n` => `push n / or`
- `and n` => `push n / and`
- `or` => Whitespace TSLS
- `not` => Whitespace TSLT
- `and` => Whitespace TSLL
- `debugger` => Whitespace LLS

Labels are assigned integers by first use (definitions and references), starting
at 0. Labels are not encoded with a leading S (positive) sign.

Internal labels are displayed with names of the form `__internal_label_{id}`,
where `{id}` is substituted for an integer, globally counting from 0 for every
internal label constructed, in lexical order. These names do not conflict with
user labels of the same form, even though the debugger instruction listing shows
them as the same.

Variables can be reassigned. Unlike the Burghard dialect, `valueinteger` and
`valuestring` use the same symbol table. A variable used as an integer or index
argument must have an integer value and a variable used as a string argument
must have a string value. A variable used as a label or filename is interpreted
as a word.
