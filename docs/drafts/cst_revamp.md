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

## `Void` token

Add a void token for errors when a value is not present, e.g., to denote a
missing argument between multiple `,` separators.
