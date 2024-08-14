# DRAFT: omniwsa assembly

- Source: here
- Corpus: TBD

## Grammar

First, the file is scanned into simple tokens:

```bnf
program ::= token*
token ::=
    | word
    | string
    | colon
    | comma
    | semi
    | slash
    | line_comment
    | block_comment
    | nested_comment
    | space
    | lf
word ::=
    | [A-Za-z0-9_.\-+$!%&<>=?@^`|~]+
string ::=
    | "\"" ([^"\\\n] | "\\" [^\n])* "\""
    | """ ([^'\\\n] | "\\" [^\n])* """
colon ::= ":"
comma ::= ","
semi ::= ";"
slash ::= "/"
line_comment ::=
    | "#" [^\n]*
    | "--" [^\n]*
    | "//" [^\n]*
block_comment ::=
    | "/*" [^*]* "*"+ ([^/*] [^*]* "*"+)* "/"
nested_comment ::=
    | "{-" ([^{-] | "{"+ [^{-] | "-"+ [^}-] | nested_comment)* "-}"
space ::= \p{White_Space} NOT '\n'
lf ::= "\n"
```

- A semicolon can be either a line comment or an instruction separator.
- A slash can either be an instruction separator or `div`.
