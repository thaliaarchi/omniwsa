# DRAFT: omniwsa assembly

- Source: [code](https://github.com/thaliaarchi/omniwsa)
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

Then, words are lexed further:

```bnf
word ::=
    | integer
    | .+
integer ::=
    | [-+]? [0-9] ("_"? [0-9])*
    | [-+]? "0" [bB] ("_"? [01])+
    | [-+]? "0" [oO] ("_"? [0-7])+
    | [-+]? "0" [oX] ("_"? [0-9a-fA-F])+
    | [-+]? ([01] "_"?)+ [bB]
    | [-+]? ([0-7] "_"?)+ [οΟ]
    | [-+]? [0-9] "_"? ([0-9a-fA-F] "_"?)* [hH]
```

- A semicolon can be either a line comment or an instruction separator.
- A slash can either be an instruction separator or `div`.
- C-style octal is configurable. Otherwise, if any contain '8' or '9', decimal
  will be used, possibly with a warning. TODO: Should the default be octal or
  decimal?
