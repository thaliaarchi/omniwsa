# Revamp of CST for Palaiologos

The Palaiologos dialect has some features, that run counter to the assumptions
of a line-oriented dialect like Burghard:
- A bare integer is a `push` without a mnemonic.
- Opcodes with multiple arguments use `,` to separate them (`store`).

## Mnemonic and argument tokens

Instructions are now no longer guaranteed to start with a mnemonic, so that
should not receive special placement in the struct. Additionally, as seen with
the unsplice transform, this placement separate from arguments makes
manipulating spaces after the mnemonic more annoying.

Instead, the mnemonic and arguments should be unified into a separated list,
`Separated<T, U>`, like `syn` [`Punctuated<T, U>`](https://docs.rs/syn/latest/syn/punctuated/struct.Punctuated.html),
which is iterable with the spaces before and after.

## Opcode arguments

Each dialect defines different types for the arguments accepted by opcodes, but
they follow general patterns. Since only a few combinations of types and arities
are used, an enum is appropriate. Any identical type combinations that generate
different Whitespace instructions need different variants.

When parsing, a dialect often overloads the arguments accepted by an opcode;
store the possible signatures as a bitset. Each parsed instruction is tagged
with its signature.

The mnemonic is included in the signature, so that mnemonic-less push can be
represented.

## Missing and skipped tokens

Add error tokens tokens [like Tolerant PHP Parser](https://github.com/microsoft/tolerant-php-parser/blob/main/docs/HowItWorks.md#error-tokens):
`Missing` is for a token expected to be present, but is not. `Skipped` is for an
extra token that can't be dealt with.

For example, a missing argument multiple `,` separators could use `Missing` for
the hole or `Skipped` for the comma.
