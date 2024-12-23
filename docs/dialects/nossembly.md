# Nossembly Whitespace assembly

- Source: <https://github.com/LeahHirst/nospace>
  (last updated [2024-11-26](https://github.com/LeahHirst/nospace/commit/60de08b7c18257161e4fb459a653fa6c4d237d28))
- Corpus: [typescript/leahhirst-nospace](https://github.com/wspace/corpus/tree/main/typescript/leahhirst-nospace)

Nossembly is Leah Hirst's Whitespace assembly dialect in her [Nospace](https://github.com/LeahHirst/nospace)
project.

## Assembler

### Grammar

```bnf
program ::= (line "\n")* line?
line ::=
    | whitespace* inst junk? whitespace*
    | pragma whitespace*
    | comment
    | whitespace*
inst ::=
    | "Push" space number
    | "Duplicate"
    | "Copy" space number
    | "Swap"
    | "Pop"
    | "Slide" space number
    | "Add"
    | "Subtract"
    | "Multiply"
    | "Divide"
    | "Mod"
    | "Store"
    | "Retrieve"
    | "Label" space label
    | "Call" space label
    | "Jump" space label
    | "JumpZero" space label
    | "JumpNegative" space label
    | "Return"
    | "End"
    | "WriteChar"
    | "WriteInt"
    | "ReadChar"
    | "ReadInt"
    | "Cast" space type
    | "Assert" space type
    | "Strict"
    | "UnknownInstruction"
pragma ::=
    | "#if" space key space value space inst junk?
    | "#define" space key space value
word ::= (NOT space)+
number ::= to_number
label ::= word
key ::= word
value ::= word
type ::=
    | "Never"
    | "Any"
    | "Unknown"
    | "Int"
    | "Char"
    | word
junk ::= (space word)+
space ::= " "
comment ::= "# " [^\n]*

# JavaScript Number constructor (ref. https://tc39.es/ecma262/multipage/abstract-operations.html#sec-tonumber).
to_number ::=
    | whitespace*
    | whitespace* numeric_literal whitespace*
numeric_literal ::=
    | decimal_literal
    | non_decimal_integer_literal
decimal_literal ::= ("+" | "-")? unsigned_decimal_literal
unsigned_decimal_literal ::=
    | "Infinity"
    | decimal_digits "." decimal_digits? exponent_part?
    | "." decimal_digits exponent_part?
    | decimal_digits exponent_part?
decimal_digits ::= [0-9]+
exponent_part ::= ("e" | "E") ("+" | "-")? decimal_digits
non_decimal_integer_literal ::=
    | ("0b" | "0B") [01]+
    | ("0o" | "0O") [0-7]+
    | ("0x" | "0X") [0-9a-fA-F]+

# JavaScript whitespace, specifically ECMAScript WhiteSpace and LineTerminator
# productions, excluding LF. It is used by String.prototype.trim and Number
# (ref. https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.trim).
whitespace ::=
    | U+0009 | U+000B | U+000C | U+000D | U+0020 | U+00A0 | U+1680 | U+2000
    | U+2001 | U+2002 | U+2003 | U+2004 | U+2005 | U+2006 | U+2007 | U+2008
    | U+2009 | U+200A | U+2028 | U+2029 | U+202F | U+205F | U+3000 | U+FEFF
```

Mnemonics are case sensitive.

### Semantics

`#if` and `#define` take a key and a value. `#define` sets the key to the value
and `#if` conditionally compiles the instruction following it on the line if the
key has the given value. They are preprocessed in lexical order.

The typechecker (in `packages/typecheck/`) statically checks that stack
operations have the correct types. `Cast` casts the top of the stack to
the given type. `Assert` throws a compilation error if the top of the stack is
not compatible with the given type.

### Generation

Numbers and labels are serialized the same and always with a sign. Labels are
assigned integers increasing from 0 in order of first occurrence and have a
leading space.

Types are assigned integer values as follows:
- `Never`: `TTL`
- `Any`: `TSL`
- `Unknown`: `TSSL`
- `Int`: `SSL`
- `Char`: `STL`
- custom: integers increasing from 10 in order of first occurrence and have a
  leading space

### Bugs

Assembler:

- Before lines are parsed, `# ` anywhere on the line is replaced with nothing.
  If this is supposed to handle comments, it is incorrect. Anything else doesn't
  make sense. This is not represented in the grammar.
- Pragmas and comments must be at the start of the line.
- Comments require a space after `#`.
- Arguments are separated by U+0020, but JavaScript whitespace is used for
  trimming. This is imprecisely represented in the grammar.
- Extra arguments are ignored for all instructions.
- `UnknownInstruction` is a valid mnemonic.
- `Strict` instruction seems unused.
- It should use `BigInt`.

Typechecker:

- The separation between `Int` and `Char` is dubious.

## Disassembler

Mnemonics are as in the assembler. Numbers are printed in base 10.

After the first label, instructions, excluding labels, are indented with two
spaces.

Built-in types are resolved to their name (`Never`, `Any`, `Unknown`, `Int`, or
`Char`) and custom types are resolved as the form `Type{id}`, where `{id}` is an
integer, increasing from 0 in order of first occurrence of types.

## Interpreter

### Bugs

- `WriteChar` and `ReadChar` use UTF-16 code units instead of Unicode
  codepoints.
- `ReadInt` only parses the next UTF-16 code unit instead of the full line.
