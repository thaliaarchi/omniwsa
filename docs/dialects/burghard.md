# Burghard Whitespace assembly

- Source: <https://github.com/wspace/burghard-wsa>
  [[docs](https://github.com/wspace/burghard-wsa/blob/main/intro.md)]
  (last updated [2023-06-26](https://github.com/wspace/burghard-wsa/commit/82b12552dd7a17124f4d5288b7a4e300a32f4086))
- Corpus: [haskell/burghard-wsa](https://github.com/wspace/corpus/tree/main/haskell/burghard-wsa)

Oliver Burghard's assembler is the second assembler for Whitespace. It was
originally published on [his site](https://web.archive.org/web/20110911114338/http://www.burghard.info/Code/Whitespace/),
along with wsinterws and wsintercpp, and linked from the official
[Whitespace page](https://web.archive.org/web/20150717140342/http://compsoc.dur.ac.uk:80/whitespace/download.php).
It is now maintained on GitHub in repos for [wsa](https://github.com/wspace/burghard-wsa),
[wsinterws](https://github.com/wspace/burghard-wsinterws), and [wsintercpp](https://github.com/wspace/burghard-wsintercpp).

## Parsing

The file must be valid UTF-8. It is decoded strictly (in effect), because
everything is demanded before it starts writing to a file.

The file is first preprocessed by removing comments, that are outside of
strings. Line comments are removed first, so an empty block comment, `{--}`, is
not possible and `--` and `;` take precedence over `{-` and `-}` in a block
comment.

When block comments are removed, they are replaced with nothing and splice
adjacent unquoted words, when no whitespace is between. For example, `add 1`,
`add"1"`, `"add"1`, `"add""1"`, `add{-c-}"1"`, `"add"{-c-}1`, and
`"add"{-c-}"1"` are parsed as `add` `1`, but `add{-c-}1` is parsed as `add1`.

```bnf
preprocess_program ::=
    (block_comment | line_comment "\n" | string | rest)* line_comment?
line_comment ::= (";" | "--") [^\n]*
block_comment ::= "{-" (block_comment_text_initial block_comment_text*)? "-}"
block_comment_text_initial ::=
    | [^{-;]
    | "{"+ [^{-;]
    | "{"* ";" [^\n]* "\n"
    | "{"* block_comment
block_comment_text ::=
    | [^{-;]
    | "-"? "{"+ [^{-;]
    | "-" [^{}-;]
    | "--" [^\n]* "\n"
    | "-"? "{"* ";" [^\n]* "\n"
    | "-"? "{"* block_comment
string ::= "\"" [^"]* "\""
rest ::= …
```

Then, it is split into lines separated by LF. Here, strings may not contain LF,
unlike the previous step, so such strings count as unterminated. CRLF and CR
do not count as line terminators.

Lines are split into word and string tokens. Space is required between two
words, but not around strings. `{-`, `-}`, `;`, and `--` cannot occur within a
word, but can be in a string. `"` cannot occur in either a word or string, since
it has no escapes.

Strings are unquoted, making them indistinguishable from words afterwards.

Everything is also lowercased, which makes mnemonics case-insensitive, but also
lowercases strings.

```bnf
lines ::= line*
line ::=
    | word? (space | string) line
    | lf
token ::= word | string
word ::= [^ \t"]+
string ::= "\"" [^"\n]* "\""
space ::= [ \t]+
lf ::= "\n" | EOF
```

After this point, all grammars are in terms of tokens. Since strings have been
unquoted to words, it is just `word` and `lf` now.

Then, preprocess all includes. The extension `.wsa` is appended to included
filenames and only the first reference to a file is included.

```bnf
include ::= "include" word lf
```

Then, preprocess options. `option` declares a named boolean option to be true
for the rest of the program.

```bnf
option_program ::= option_line*
option_line ::=
    | "option" word lf
    | "ifoption" word lf option_line*
      (("elseifoption" word | "elseoption") lf option_line*)*
      "endoption" lf
    | word* lf
```

Finally, parse the instructions.

```bnf
program ::= (inst? lf)*
inst ::=
    | "push" integer
    | "pushs" string
    | "doub"
    | "swap"
    | "pop"
    | "add" integer?
    | "sub" integer?
    | "mul" integer?
    | "div" integer?
    | "mod" integer?
    | "store" integer?
    | "retrive" integer?
    | "label" label
    | "call" label
    | "jump" label
    | "jumpz" label
    | "jumpn" label
    | "jumpp" label
    | ("jumpnp" | "jumppn") label
    | "jumpnz" label
    | "jumppz" label
    | "ret"
    | "exit"
    | "outc"
    | "outn"
    | "inc"
    | "inn"
    | "debug_printstack"
    | "debug_printheap"
    | "test" integer
    | "valueinteger" integer_variable integer
    | "valuestring" string_variable string
integer ::= integer_literal | integer_variable
string ::= string_literal | string_variable
integer_literal ::= word
string_literal ::= word
integer_variable ::= "_":word
string_variable ::= "_":word
label ::= word
```

Where `"_":word` denotes a word that starts with `_`. The rest after the prefix
may be empty.

Integer literals are parsed with `read :: Integer`, so it follows the same
grammar as `readi` in the reference interpreter.

## Generating

Extension control flow instructions generate auxiliary labels of the form
`__trans__{pc}__{kind}__`, where `{pc}` is the 1-indexed position of the
instruction in the instruction list and `{kind}` is unique to the operation.

Extension instructions translate as follows:

- `pushs` translates to a `0`-terminated sequence of pushes in reverse order
- `jumpp l` branches on positive and translates to:

  ```wsa
      dup / jn __trans__{pc}__0__
      dup / jz __trans__{pc}__0__
      drop / jmp l
  __trans__{pc}__0__:
      drop
  ```

- `jumpnp l` (and its alias `jumppn`) branches on negative or positive and
  translates to:

  ```wsa
      jz __trans__{pc}__1__
      jmp l
  __trans__{pc}__1__:
  ```

- `jumpnz l` branches on negative or zero and translates to:

  ```wsa
      dup / jn __trans__{pc}__2__
      dup / jz __trans__{pc}__2__
      jmp __trans__{pc}__3__
  __trans__{pc}__2__:
      drop / jmp l
  __trans__{pc}__3__:
      drop
  ```

- `jumppz l` branches on positive or zero and translates to:

  ```wsa
      jn __trans__{pc}__4__
      jmp l
  __trans__{pc}__4__:
  ```

- `test n` translates to `dup / push n / sub`
- `add n` translates to `push n / add`
- `sub n` translates to `push n / sub`
- `mul n` translates to `push n / mul`
- `div n` translates to `push n / div`
- `mod n` translates to `push n / mod`
- `store n` translates to `push n / swap / store`
- `retrive n` translates to `push n / retrieve`

`0` is encoded with a sign and digit as `SS`.

Labels are encoded as signed integers incrementing from 0 in definition order.
Duplicate labels are forbidden.

Variables defined with `valueinteger` and `valuestring` do not conflict and can
share names. They are visible to any successive instructions.

Imports are resolved relative to the current directory, not to the file
including it.

An extra `\n\n\n` is appended to the encoded program.

## Mnemonics

Mnemonics are case-insensitive. All are consistently written as lowercase,
except for the I/O instructions, which are mostly written as `outC`, `outN`,
`inC`, and `inN`.

In programs, they occur with the following counts:

- wsinterws.wsa: `outN` (17), `outn` (6), `outc` (1), `inC` (1), `inc` (1),
  `inn` (1)
- libs/io.wsa: `outC` (3)
- examples/prim.wsa: `outN` (1)

In wsintercpp, `debug_printstack` and `debug_printheap` are known as
`debugprintstack` and `debugprintheap`, which it prints in a listing when the
`-d` debug flag is passed.

## Bugs in assembler

- Anything can be `"`-quoted.
- Strings are lowercased.
- Strings containing `"` or starting with `_` cannot be represented.
- Block quotes are replaced with nothing, instead of with a space or LF, which
  splices adjacent unquoted words.
- `elseoption` can appear before `elseoption` or there can be multiple
  `elseoption`.
