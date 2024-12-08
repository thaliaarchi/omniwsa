# omniwsa

An assembler for all dialects of Whitespace assembly.

The Whitespace programming language is unique in that it lacks visible syntax,
so [most implementations](https://github.com/wspace/corpus) construct an
assembly language to simplify writing programs. These custom syntaxes share many
features, but are mutually incompatible. This project bridges those gaps through
a common concrete syntax tree and facilitates transformations between dialects.

This is a descriptivist implementation which maintains strict bug-for-bug
compatibility with the various assemblers. While many Whitespace projects are
not actively maintained, omniwsa will be updated if upstream issues are
resolved.

For specifications of supported dialects and information about the architecture
of omniwsa, see the documentation in [docs/](docs/index.md).

License: MPL-2.0
