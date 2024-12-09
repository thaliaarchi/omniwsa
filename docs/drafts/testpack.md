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

jq [.test](https://github.com/jqlang/jq/blob/master/tests/jq.test) files are
a set test cases, each with three or more lines: the program, an input, and the
expected outputs. Comments are ignored and tests are separated by blank lines.

Rust [UI tests](https://rustc-dev-guide.rust-lang.org/tests/ui.html) are
snapshot integration tests in [tests/ui/](https://github.com/rust-lang/rust/blob/master/tests/ui/README.md).
There is [ongoing work](https://github.com/rust-lang/compiler-team/issues/536)
to migrate to an [out-of-tree](https://github.com/oli-obk/ui_test) test runner
shared with Miri and Clippy.

The TypeScript compiler has unit tests, baseline tests, and fourslash tests.

- [Baseline tests](https://github.com/microsoft/TypeScript-Compiler-Notes/blob/main/systems/testing/baselines.md)
  are snapshot integration tests with a .ts/.tsx TypeScript source file in tests/cases/ and
  corresponding files in tests/baselines/reference/: .js with the TypeScript
  source delimited with `//// [filename.ts]` and JavaScript output delimited with
  `//// [filename.js]`, .symbols with symbol information, .types with inline type
  information, and, if applicable, .errors.txt with type errors.

  Two examples:
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

- [Fourslash tests](https://github.com/microsoft/TypeScript-Compiler-Notes/blob/main/systems/testing/fourslash.md)
  are integration tests for the user-facing parts of the API, such as for IDE
  clients. Lines starting with `////` represent the text of an input file and
  lines starting with `//` are TSConfig settings or filename declarations. The
  syntax is described in [`fourslash.ts`](https://github.com/microsoft/TypeScript/blob/main/tests/cases/fourslash/fourslash.ts).

  An example: [`addAllMissingImportsNoCrash.ts`](https://github.com/microsoft/TypeScript/blob/main/tests/cases/fourslash/addAllMissingImportsNoCrash.ts)

[`insta`](https://docs.rs/insta/latest/insta/) is a snapshot testing library for
Rust. As an example, it is used by [if-to-let-chain](https://github.com/Alexendoo/if-to-let-chain/tree/master/src/snapshots).
