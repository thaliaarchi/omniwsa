# voliva Whitespace assembly

- Source: [code](https://github.com/voliva/wsa)
  (last updated [2024-10-05](https://github.com/voliva/wsa/tree/e632ecfaa11d685364787294599ba542bfacd796))
- Corpus: [typescript/voliva-wsa](https://github.com/wspace/corpus/tree/main/typescript/voliva-wsa)

## Grammar

The program is divided into lines and each line is lexed into tokens:

```bnf
program ::= (line "\n")* line?
line ::= (\s* token)* \s* (DECORATION | LINE_COMMENT)?
token ::= STRING | CHAR | VARIABLE | INTEGER | WORD

STRING ::= "\"" ([^"\\\n] | \\["\\bfnrtv])* "\""
CHAR ::= "'" ([^'\\\n] | \\['\\bfnrtv]) "'"
VARIABLE ::= "_" WORD
INTEGER ::= BIGINT
WORD ::= [^\s;"']+
DECORATION ::= ";#;" " "? [^\n]*
LINE_COMMENT ::= ";" [^\n]*

# BigInt constructor, without whitespace (ref. https://tc39.es/ecma262/multipage/abstract-operations.html#sec-stringtobigint).
BIGINT ::=
    | ("-" | "+") [0-9]+
    | ("0b" | "0B") [01]+
    | ("0o" | "0O") [0-7]+
    | ("0x" | "0X") [0-9 a-f A-F]+

# RegExp \s (ref. https://tc39.es/ecma262/multipage/text-processing.html#sec-compiletocharset)
\s ::=
    | U+0009 # Tab
    | U+000A # Line feed
    | U+000B # Vertical tab
    | U+000C # Form feed
    | U+000D # Carriage return
    | U+0020 # Space
    | U+00A0 # No-break space
    | U+1680 # Ogham space mark
    | U+2000 # En quad
    | U+2001 # Em quad
    | U+2002 # En space
    | U+2003 # Em space
    | U+2004 # Three-per-em space
    | U+2005 # Four-per-em space
    | U+2006 # Six-per-em space
    | U+2007 # Figure space
    | U+2008 # Punctuation space
    | U+2009 # Thin space
    | U+200A # Hair space
    | U+2028 # Line separator
    | U+2029 # Paragraph separator
    | U+202F # Narrow no-break space
    | U+205F # Medium mathematical space
    | U+3000 # Ideographic space
    | U+FEFF # Zero width no-break space
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
    | (?i)"or" integer?
    | (?i)"not"
    | (?i)"and" integer?
    | (?i)"store" integer?
    | (?i)"storestr" string
    | (?i)"retrieve" integer?
    | (?i)"label" label
    | (?i)"call" label
    | (?i)"jump" label
    | (?i)"jumpz" label
    | (?i)"jumpn" label
    | (?i)"jumpp" label
    | ((?i)"jumppn" | (?i)"jumpnp") label
    | (?i)"jumpnz" label
    | (?i)"jumppz" label
    | (?i)"ret"
    | (?i)"exit"
    | (?i)"outn"
    | (?i)"outc"
    | (?i)"readn"
    | (?i)"readc"
    | (?i)"valueinteger" variable integer
    | (?i)"valuestring" variable string
    | (?i)"debugger"
    | (?i)"include" filename

integer ::= INTEGER | CHAR | VARIABLE
index ::= INTEGER | VARIABLE
string ::= STRING | WORD | VARIABLE
label ::= WORD | VARIABLE
variable ::= VARIABLE
filename ::= WORD | VARIABLE | STRING
```

Mnemonics are compared with effectively ASCII case folding.

Programs are decoded as UTF-8 and each invalid *byte* is replaced with the
U+FFFD replacement character. Thus, the input cannot have unpaired surrogate
halves. The tokenizer is careful to process UTF-8 code points instead of UTF-16
code units.

## Types

- Integers can be integer or char literals or variable references.
- Index integers can be integer literals or variable references.
- Strings can be quoted or unquoted, with no semantic difference, or variable
  references.
- Labels are unquoted words and may start with `_`.
- Variables are denoted by a prefix of `_`.
- Filenames can be unquoted or quoted and may start with `_`.

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
- `jumppn l` or `jumpnp l` => `jz __internal_label_{id} / jmp l / __internal_label_{id}:`
- `jumpnz l` => `push 1 / sub / jn l`
- `jumppz l` => `jn __internal_label_{id} / jmp l / __internal_label_{id}:`
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

Decoration comments (`;#;`) specify a line of text that is to be included in the
assembled program. The concatenated text of all decoration comments is prepended
to the first lines in the assembled program (not interspersed with
Whitespace-syntax whitespace characters). The decoration text is sanitized so
that space becomes non-breaking space (U+00A0), tab becomes two non-breaking
spaces, and an optional leading space on each line is stripped. Decorations do
not apply to the locally adjacent code. When there are more decoration comments
than lines in the assembled program, any extra decorations are omitted.

## Bugs in the assembler

- The assembler uses `disassembler`, but the disassembler uses `dbg` and that
  was [decided](https://github.com/voliva/wsa/pull/1#issuecomment-2316998205) to
  be the canonical mnemonic.
- Exception handling within the promises is messy and results in poor error
  messages.
- When there are more decoration comments than lines in the assembled program,
  any extra decorations are omitted.
