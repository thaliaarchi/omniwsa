# Test case packer

Tests for Whitespace assemblers are often tiny input files with short Whitespace
output or an error message. Since I am testing external assemblers which expect
file input, that's a lot of small files. Sometimes I've automated the generation
of these inputs with shell scripts, but this leads to duplication.

I want a human-readable structured data format for snapshot testing of many
small files, which can encode strings that can have arbitrary bytes. The strings
are usually valid UTF-8, but they also need to be able to encode invalid UTF-8
sequences. JSON does not have this property.

## Prior art

jq [.test](https://github.com/jqlang/jq/blob/master/tests/jq.test) files

Rust [UI](https://github.com/rust-lang/rust/tree/master/tests/ui) tests

TypeScript has a testing format with the following files:
- .ts: TypeScript source file
- .js: TypeScript source delimited with `//// [filename.ts]` and JavaScript
  output delimited with `//// [filename.js]`
- .symbols: symbol information
- .types: inline type information
- .errors.txt: type errors

Two examples of it:
- `compiler/evolvingArrayTypeInAssert`:
  [.ts](https://github.com/microsoft/TypeScript/blob/main/tests/cases/compiler/evolvingArrayTypeInAssert.ts),
  [.js](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/evolvingArrayTypeInAssert.js),
  [.symbols](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/evolvingArrayTypeInAssert.symbols),
  [.types](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/evolvingArrayTypeInAssert.types)
- `conformance/parser/ecmascript5/ErrorRecovery/ArrowFunctions/ArrowFunction1`:
  [.ts](https://github.com/microsoft/TypeScript/blob/main/tests/cases/conformance/parser/ecmascript5/ErrorRecovery/ArrowFunctions/ArrowFunction1.ts),
  [.js](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/ArrowFunction1.js),
  [.symbols](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/ArrowFunction1.symbols),
  [.types](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/ArrowFunction1.types),
  [.errors.txt](https://github.com/microsoft/TypeScript/blob/main/tests/baselines/reference/ArrowFunction1.errors.txt)

TypeScript also has fourslash tests for IDE integration tests, e.g.,
[`fourslash/addAllMissingImportsNoCrash.ts`](https://github.com/microsoft/TypeScript/blob/main/tests/cases/fourslash/addAllMissingImportsNoCrash.ts).
