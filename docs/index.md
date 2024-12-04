# omniwsa documentation

## Whitespace assembly dialects

Specifications for Whitespace assembly dialects following their implementations,
including grammar, semantics, code generation, and bugs.

- [Burghard](dialects/burghard.md) (haskell/burghard-wsa)
- [CensoredUsername](dialects/censoredusername.md) (rust/censoredusername-whitespacers)
- [Lime](dialects/lime.md) (c/manarice)
- [littleBugHunter](dialects/littlebughunter.md) (csharp/littlebughunter-assembler)
- [omniwsa](dialects/omniwsa.md) (rust/thaliaarchi-omniwsa)
- [Palaiologos](dialects/palaiologos.md) (c/kspalaiologos-asm2ws)
- [rdebath](dialects/rdebath.md) (c/rdebath)
- [Respace](dialects/respace.md) (cpp/thaliaarchi-respace)
- [voliva](dialects/voliva.md) (typescript/voliva-wsa)
- [wconrad](dialects/wconrad.md) (ruby/wconrad)
- [Whitelips](dialects/whitelips.md) (javascript/vii5ard-whitelips)
- [wsf](dialects/wsf.md) (whitespace/thaliaarchi-wslib)
- [Others](dialects/others.md)

## Design drafts

Design drafts describing features and goals before implementing them in omniwsa.
They have not (yet) been updated to document the implementation, but provide a
good intuition. Listed reverse-chronologically.

- [Syntax inference through fuzzing](drafts/fuzz_syntax.md)
- [Configurable dynamic lexing and parsing](drafts/dynamic_parsing.md)
- [Conventionally UTF-8 scanner](drafts/scanner.md)
- [Revamp of CST for Palaiologos](drafts/cst_revamp.md)
- [omniwsa assembly](dialects/omniwsa.md)
- [An interoperable CST for Whitespace assembly](drafts/interop_cst.md)
- [Whitespace assembly Macros](drafts/macros.md)
- [List of Whitespace assembly mnemonics](drafts/mnemonics.md)
- [Parsing any Whitespace assembly dialect](drafts/parsing.md)
