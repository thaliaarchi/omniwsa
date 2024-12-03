# wconrad

- Source: <https://github.com/wspace/wconrad-ruby>
  (last updated [2021-10-23](https://github.com/wspace/wconrad-ruby/commit/406a09e80f5fd6b6f2beb8e9d9f039536bc8db23))
- Corpus: [ruby/wconrad](https://github.com/wspace/corpus/tree/main/ruby/wconrad)

Wayne Conrad's assembler, whitespace-asm, is the first Whitespace assembler. It
was originally published on [his site](https://web.archive.org/web/20120417161917/http://yagni.com:80/whitespace/index.html),
along with an interpreter and disassembler, and linked from the official
[Whitespace page](https://web.archive.org/web/20150717140342/http://compsoc.dur.ac.uk:80/whitespace/download.php).
It predates Whitespace 0.3, but support for `copy` and `signed` was added by
Tommie Levy to the interpreter in 2016, then later by analogy to whitespace-asm.
It is now maintained [on GitHub](https://github.com/wspace/wconrad-ruby).

## Grammar

```bnf
program ::= (line "\n")* line?
line ::= strip_space* inst line_comment* strip_space*

inst :=
      | "push" signed
      | "dup"
      | "copy" signed
      | "swap"
      | "discard"
      | "slide" signed
      | "add"
      | "sub"
      | "mul"
      | "div"
      | "mod"
      | "store"
      | "retrieve"
      | label
      | "label" unsigned
      | "call" unsigned
      | "jump" unsigned
      | "jz" unsigned
      | "jn" unsigned
      | "ret"
      | "exit"
      | "outchar"
      | "outnum"
      | "readchar"
      | "readnum"

signed ::= space+ [-+]? [0-9]+
unsigned ::= space+ [0-9]+
label := [0-9]+ ":"
line_comment ::= "#" [^\n]*

# Regexp \s
space ::= " " | "\t" | "\n" | "\v" | "\f" | "\r"
# String#strip
strip_space ::= space | "\0"
```

`Regexp` [`\s`](https://docs.ruby-lang.org/en/master/Regexp.html#class-Regexp-label-Shorthand+Character+Classes)
does not include NUL. Before [PR#4164](https://github.com/ruby/ruby/pull/4164)
(Make String#{strip,lstrip}{,!} strip leading NUL bytes, 2021-02-19),
`String#strip` only stripped NUL bytes on the right, even though [documentation](https://docs.ruby-lang.org/en/master/String.html#class-String-label-Whitespace+in+Strings)
indicated that it trimmed NUL on both ends.

## Generation

Integers, including labels, have arbitrary precision.

Labels are encoded as the given unsigned integers without leading zeros.

## Bugs in the assembler

- Any line that does not match any of the instruction patterns is ignored,
  effectively treated as a line comment. This includes if an argument is
  malformed.
- Spaces are stripped before stripping line comments, so a line comment after an
  instruction does not have its spaces stripped and causes the line to not
  match.
