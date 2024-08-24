# omniwsa

An assembler for all dialects of Whitespace assembly.

The Whitespace programming language is unique in that it has no visible syntax,
so just about [every implementation](https://github.com/wspace/corpus) invents
an assembly language to make writing programs easier. These syntactic layers
over Whitespace share many features, but are all incompatible in different ways.
This project bridges those gaps through a common concrete syntax tree and
transformations between dialects.

This is a descriptivist implementation with pedantic bug-for-bug compatibility.
It is considered a bug in omniwsa if its behavior differs from the reference
assemblers. Since most projects for Whitespace are not actively maintained, it
is unreasonable to expect fixes; however, if upstream is fixed, open an issue
here and omniwsa will be updated to match it.
