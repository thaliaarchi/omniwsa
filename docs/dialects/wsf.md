# wsf dialect

- Source: <https://github.com/thaliaarchi/wslib>
  (last updated [2024-11-20](https://github.com/thaliaarchi/wslib/commit/8e1c1050c824a6797e4a398d4be436e1c63262cb)
- Corpus: [whitespace/thaliaarchi-wslib](https://github.com/wspace/corpus/tree/main/whitespace/thaliaarchi-wslib)

wsf (“Whitespace Forth”) is a Whitespace assembly dialect by Thalia Archibald,
made for the [wslib](https://github.com/thaliaarchi/wslib) standard library and
also used for various [coding challenges](https://github.com/thaliaarchi/ws-challenges).
It is inspired by Forth and uses postfix argument styles and numbers fused with
mnemonics, unlike other dialects. The assembler is written in sed and bash and
targets whitespace-rs assembly.

## Grammar

```bnf
program ::= (line "\n")* line?
line ::= space? inst? (space inst)* space? line_comment?
inst ::=
    | path space "import"
    | path space "export"
    | integer
    | string
    | dup
    | copy
    | "swap"
    | drop
    | slide
    | integer? arith
    | "store"
    | "retrieve"
    | label_def
    | jump space label
    | bool_func
    | int_func
    | "ret"
    | "end"
    | "printc"
    | "printi"
    | string space "prints"
    | "readc"
    | "readi"

path ::= "\"" [^"\n#]+ "\""
string ::= "\"" ([^"\\#] | "\\" [abtnvfre"\\]){,40} "\""
integer ::=
    | "-"? [0-9]+
    | "-"? ("0x" | "0X") [0-9 a-f A-F]{2}
    | "-"? ("0b" | "0B") "0"* [01]{1,8}
    | "-"? char
char ::=
    | "'" ([\ -~] NOT ['\\]) "'"
    | "'\\" [abtnvfre'\\] "'"
    | "'\\x" [0-9 a-f A-F]{2} "'"
arith ::= "+" | "-" | "*" | "/" | "%"
dup ::= "^" | ([2-9] | "10")? "dup"
copy ::= "^" integer
drop ::= ([2-9] | "10")? "drop"
slide ::= "-"? [0-9]+ "slide"
label ::=
    | [A-Za-z0-9_.-]+
    | "%" [0-9]+
label_def ::=
    | label ":"
    | "label" space label
jump ::=
    | "call" | "jmp" | "jz" | "jn"
    | "j=" | "j<" | "j>" | "j<=" | "j>="
    | "jeof"
bool_func ::=
    | "&&" | "||" | "!" | "!!"
    | "=" | "!=" | "<" | ">" | "<=" | ">="
    | "pos?" | "neg?"
int_func ::=
    | "&" | "|" | "^" | "&~" | "<<" | ">>" | "**" | "~" | "neg"

comment ::= "#" [^\n]*
# `sed -E` \s+
space ::= (" " | "\t" | "\v" | "\f" | "\r")+
```

Space is not required around strings or chars. The grammar for `sep` is overly
restrictive.

`\s` in GNU sed (via Gnulib) [defers to](https://git.savannah.gnu.org/cgit/gnulib.git/tree/lib/regcomp.c?id=38b5fabdfcf0ddd516fdd9105ccb1b2ac38cb62c#n3515)
`isspace` from `<ctype.h>`.

## Generation

- `{n}` => `push n`
- `"{s}"` => `push c` for each char in `s` in reverse order
- `"{s}" prints` => `push c / printc` for each char in `s` in reverse order.
- `{n}+` => `push n / add`
- `{n}-` => `push n / sub`
- `{n}*` => `push n / mul`
- `{n}/` => `push n / div`
- `{n}%` => `push n / mod`
- `^` => `dup`
- `^{n}` => `copy n`
- `2dup` => `copy 1 / copy 1`
- `3dup` => `copy 2 / copy 2 / copy 2`
- `4dup` => `copy 3 / copy 3 / copy 3 / copy 3`
- `5dup` => `copy 4 / copy 4 / copy 4 / copy 4 / copy 4`
- `6dup` => `copy 5 / copy 5 / copy 5 / copy 5 / copy 5 / copy 5`
- `7dup` => `copy 6 / copy 6 / copy 6 / copy 6 / copy 6 / copy 6 / copy 6`
- `8dup` => `copy 7 / copy 7 / copy 7 / copy 7 / copy 7 / copy 7 / copy 7 / copy 7`
- `9dup` => `copy 8 / copy 8 / copy 8 / copy 8 / copy 8 / copy 8 / copy 8 / copy 8 / copy 8`
- `10dup` => `copy 9 / copy 9 / copy 9 / copy 9 / copy 9 / copy 9 / copy 9 / copy 9 / copy 9 / copy 9`
- `2drop` => `drop / drop`
- `3drop` => `drop / drop / drop`
- `4drop` => `slide 3 / drop`
- `5drop` => `slide 4 / drop`
- `6drop` => `slide 5 / drop`
- `7drop` => `slide 6 / drop`
- `8drop` => `slide 7 / drop`
- `9drop` => `slide 8 / drop`
- `10drop` => `slide 9 / drop`
- `{n}slide` => `slide n`
- `j= l` => `sub / jz l`
- `j< l` => `sub / jn l`
- `j> l` => `swap / sub / jn l`
- `j<= l` => `push 1 / add / sub / jn l`
- `j>= l` => `swap / push 1 / add / sub / jn l`
- `jeof l` => `jn`, `jz`, or `push 1 / sub / jn l` (default)
- `&&` => `call bool.and`
- `||` => `call bool.or`
- `!` => `call bool.not`
- `!!` => `call bool.cast`
- `=` => `call bool.is_eq`
- `!=` => `call bool.is_ne`
- `<` => `call bool.is_lt`
- `>` => `call bool.is_gt`
- `<=` => `call bool.is_le`
- `>=` => `call bool.is_ge`
- `pos? !` => `call bool.is_non_pos`
- `neg? !` => `call bool.is_non_neg`
- `pos?` => `call bool.is_pos`
- `neg?` => `call bool.is_neg`
- `&` => `call int.and`
- `|` => `call int.or`
- `^` => `call int.xor`
- `&~` => `call int.andnot`
- `<<` => `call int.shl`
- `>>` => `call int.shr`
- `**` => `call math.exp`
- `~` => `push 1 / add / push -1 / mul`
- `neg` => `push -1 / mul`

where `{n}` is an integer literal and `"{s}"` is a string literal.

Labels are transformed to work for whitespace-rs: A leading `.` is changed to
`_` and `.` elsewhere is changed to `__`. An `L_` prefix is added to labels that
start with a digit.

`push`, `copy`, and `slide` with 0 are assembled with a positive sign (S) in the
fork of whitespace-rs used by wsf. Upstream emits no sign.

### Imports and modules

Labels that do not contain a `.` or have only a leading `.` are local to the
current module. The module name is prepended after the possible leading `.` to
make labels globally unique.

Labels starting with `.` are private by convention, but this scoping is not
enforced.

TODO: Expand section.

## Notes

- Although `dup` is implicitly allowed, `^` is used exclusively in wslib.
- `copy` and `slide` with negative indices are explicitly assembled by
  wsf-assemble, but rejected by whitespace-rs.
- Char literals can be negated.
- If the final line in the assembled program is not `jmp`, `jz`, `jn`, `ret`, or
  `end`, it exits with an error.
- wsf uses a [fork](https://github.com/wspace/censoredusername-whitespace-rs) of
  whitespace-rs, which changes it emit a positive sign for 0 and to exit with
  nonzero on error.

## Bugs in the assembler

- Non-space whitespace characters in strings and chars are replaced with space.
- `#` cannot appear in strings or paths.
- Strings are limited to 40 characters.
- Many features are limited to a fixed number of occurrences per line.
- Separation between tokens is suspect.
