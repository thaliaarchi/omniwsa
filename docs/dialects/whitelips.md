# Whitelips Whitespace assembly

- Source: [https://github.com/vii5ard/whitespace](https://github.com/vii5ard/whitespace/blob/master/ws_asm.js)
  [[docs](https://vii5ard.github.io/whitespace/help.html#assembly)]
  (last updated [2023-08-31](https://github.com/vii5ard/whitespace/commit/b2a65e8d7f4c1aa8d3d1a235c473a45635343ad0))
- Corpus: [javascript/vii5ard-whitelips](https://github.com/wspace/corpus/tree/main/javascript/vii5ard-whitelips)

## Grammar

```bnf
program ::=
    | space? (inst space | unspaced_inst space?)* (inst | unspaced_inst)? space?
inst ::=
    | "push" space (number | string)
    | "dup"
    | "copy" space number
    | "swap"
    | "drop"
    | "slide" space number
    | "add" (space number)?
    | "sub" (space number)?
    | "mul" (space number)?
    | "div" (space number)?
    | "mod" (space number)?
    | "store"
    | "retrieve" (space number)?
    | "label" space label
    | "call" space label
    | "jmp" space label
    | "jz" space label
    | "jn" space label
    | "ret"
    | "end"
    | "printc"
    | "printi"
    | "readc" (space number)?
    | "readi" (space number)?
    | "include" space string
    | "macro" space macro_name ":" space? (macro_inst space)* "$$"
    | macro_name (space (label | number | string))*
# Does not require space after
unspaced_inst ::=
    | label ":"
macro_inst ::=
    | inst
    | "$label"
    | "$number"
    | "$string"
    | "$redef"
    | "$" [0-9]+

# Value types
number ::= number_lit | char_lit
string ::= string_lit
char ::= char_lit
label ::= label_lit | number_lit
macro_name ::= label_lit

label_lit ::= [a-zA-Z_$.][a-zA-Z0-9_$.]*
number_lit ::= [+-]?\d+
string_lit ::=
    | "\"" ([^"\n\\] | \\[nt] | \\[0-9]+ | \\.)*? "\""
    | "'" ([^'\n\\] | \\[nt] | \\[0-9]+ | \\.)*? "'"
char_lit ::=
    | "'" ([^'\n\\] | \\[nt] | \\[0-9]+ | \\.) "'"
space ::= ([ \t\n\r] | comment)+
comment ::=
    | ";" [^\n]*
    | "#" [^\n]*
    | "--" [^\n]*
    | block_comment
block_comment ::= "{-" block_comment_text* "-"+ "}"
block_comment_text ::=
    | [^{-]
    | "-"+ [^{}-]
    | "-"* "{"+ [^{-]
    | "-"* "{"* block_comment
```

The pattern `.` includes `\n` here.

Whitespace (or a comment) is required between any two tokens, except for after
a `:`-label or around comments.

## Generation

- `push s` => `push c` for each character in `s` in reverse order
- `add n` => `push n / add`
- `sub n` => `push n / sub`
- `mul n` => `push n / mul`
- `div n` => `push n / div`
- `mod n` => `push n / mod`
- `retrieve n` => `push n / retrieve`
- `readc n` => `push n / readc`
- `readi n` => `push n / readi`

## Semantics

Strings may contain escape sequences: `\n` as LF; `\t` as tab; `\` followed by
greedy decimal digits, parsed as the decimal value; or `\` followed by any UTF-8
character (including LF, allowing for line continuations). `"`-strings are
NUL-terminated, but `'`-strings are not.

Labels starting with `.` are local labels. A non-local label definition (the
parent) opens a block until the next non-local label definition. Any local label
definitions and references in a block are scoped to that block and are encoded
by prepending the parent label to the local labels. The entry block before the
first non-local label prepends nothing.

`include` assembles the named file, with all already seen symbols visible to the
child assembler, and appends it to the end of the including file. Only the first
`include` for a filename is expanded. `include`s are not expanded in a macro
definition.

### Macros

Macros are only applied when the types of the successive tokens match the
parameter types expected by the macro. When the argument types do not match, if
it has the name of a mnemonic, that instruction is used instead; otherwise, it
produces an error.

A macro can be named anything, that's a valid label token. This includes
instruction mnemonics, the names of previously defined macros, or labels. It
only shadows a mnemonic if the arguments are of the appropriate types. Macro
definitions have dynamic scoping and replace previously defined macros of the
same name. Macros do not expand in label position, so do not conflict with
labels. Unless shadowed, `$$`, `$label`, `$number`, `$string`, and `$redef` are
reserved outside of a macro. The `${n}` label form is not reserved. A macro
named as any of these keywords cannot be referenced within a macro.

Labels can be generated in a macro using the form `${n}`, where `{n}` is a
decimal number. These are replaced with a token of the form
`.__{name}${n}_{id}`, where `{name}` is the name of the macro and `{id}` is the
global count that this macro has been expanded, starting from 1.

A macro cannot contain `$$`, so it cannot expand to a full macro definition
without the caller adding `$$`.

Macros are in scope for their body and any successive instructions, as well as
any files included after the definition. Macros defined in an included file are
introduced into the scope of the parent file. Macros are supposed to have a
recursion depth of 16, but it does not seem to work.

## Notes

- UTF-8–oriented.
- Uses `BigInt` for numbers and string characters.
- Labels are assigned sequentially from `0` in definition order, even for
  numeric labels.

## Bugs in the assembler

- Macro recursion is not properly implemented or restricted.
- Macro definitions have dynamic scoping and replace previously defined macros
  of the same name.
- The site on GitHub Pages is out of date ([issue #10](https://github.com/vii5ard/whitespace/issues/10)).
