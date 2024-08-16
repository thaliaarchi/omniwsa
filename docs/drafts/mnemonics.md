# List of Whitespace assembly mnemonics

- `push`:
  - `push`, `psh`, `pus`
  - `pushnumber`, `pushnum`
  - `pushchar`, `pushch`
  - `append`
  - `<number>`, `<char>`
- `dup`:
  - `duplicate`, `dupli`, `dupl`, `dup`
  - `dupe`
  - `doub`
  - `^`
- `copy`:
  - `copy`
  - `copyn`, `copynth`, `copyat`
  - `dupn`, `dupnth`, `dupat`
  - `duplicaten`, `duplicatenth`, `duplicateat`
  - `pick`
  - `ref`
  - `take`
  - `pull`
  - `^<number>`
- `swap`:
  - `swap`, `swp`, `swa`
  - `exchange`, `exch`, `xchg`
  - `switch`
- `drop`:
  - `drop`
  - `discard`, `disc`, `dsc`
  - `pop`
  - `away`
  - `delete`, `del`
- `slide`:
  - `slide`, `slid`
  - `sliden`
  - `slideoff`
  - `<unsigned>slide`
- `add`:
  - `add`
  - `addition`
  - `adding`
  - `plus`
  - `sum`
  - `+`
- `sub`:
  - `subtract`, `subt`, `sub`
  - `subtraction`
  - `minus`
  - `-`
- `mul`:
  - `multiply`, `multi`, `mult`, `mul`
  - `multiplication`
  - `multiple`
  - `times`
  - `*`
- `div`:
  - `divide`, `div`
  - `division`
  - `integerdivision`, `intdiv`
  - `/`
- `mod`:
  - `modulo`, `mod`
  - `remainder`, `rem`
  - `divisionpart`
  - `%`
- `store`:
  - `store`, `stor`, `sto`, `st`
  - `set`
  - `put`
- `retrieve`:
  - `retrieve`, `retrive`, `retri`, `retrv`, `retr`, `reti`
  - `load`, `lod`, `ld`
  - `fetch`
  - `get`
  - `recall`, `rcl`
- `label`:
  - `label`, `lbl`
  - `mark`, `mrk`, `marksub`, `marklabel`, `marklocation`
  - `defun`, `def`, `deflabel`
  - `part`
  - `<label>:`
  - `%<label>:`
  - `@<label>`
  - `<<label>>:`
  - `L<number>:`
  - `label_<number>:`, `label_<number>`
- `call`:
  - `call`, `cll`
  - `callsubroutine`, `callsub`, `calls`, `cas`
  - `jsr`
  - `gosub`
  - `subroutine`
- `jmp`:
  - `jump`, `jmp`, `jm`, `jp`, `j`
  - `branch`, `br`, `b`
  - `goto`
- `jz`:
  - [`jump`, `jmp`, `jm`, `jp`, `j`, `branch`, `br`, `b`, `goto`] * [`zero`, `zer`, `ze`, `z`, `null`, `nil`, `ez`, `0`]
  - [`jump`, `jmp`, `branch`, `goto`] * [`ifzero`, `if0`, `iz`]
  - `zero`
- `jn`:
  - [`jump`, `jmp`, `jm`, `jp`, `j`, `branch`, `br`, `b`, `goto`] * [`negative`, `nega`, `neg`, `ne`, `n`, `ltz`, `lz`, `l0`]
  - [`jump`, `jmp`, `branch`, `goto`] * [`ifnegative`, `ifneg`, `ifn`, `in`]
  - `negative`
- `ret`:
  - `return`, `ret`, `rts`
  - `endsubroutine`, `endsub`, `ends`, `ens`
  - `subroutineend`, `subend`
  - `endfunction`, `endfunc`
  - `exitsub`
  - `controlback`, `back`
  - `leave`
- `end`:
  - `endprogram`, `endprog`, `endp`, `end`
  - `exit`
  - `halt`, `hlt`
  - `terminate`
  - `quit`
  - `die`
  - `finishprogram`, `finish`
- `printc`:
  - [`print`, `output`, `out`, `write`] * [`character`, `char`, `chr`, `ch`, `c`]
  - [`put`, `p`, `o`, `w`] * [`char`, `chr`, `ch`, `c`]
  - [`prt`, `ot`, `wr`, `wt`] * [`chr`, `ch`, `c`]
  - [`char`, `chr`, `ch`, `c`] * [`out`]
- `printi`:
  - [`print`, `output`, `out`, `write`] * [`integer`, `int`, `i`, `number`, `num`, `n`]
  - [`put`, `prt`, `p`, `o`, `w`] * [`int`, `i`, `num`, `n`]
  - [`ot`, `wr`, `wt`] * [`i`, `n`]
  - [`int`, `i`, `num`, `n`] * [`out`]
- `readc`:
  - [`read`, `input`, `get`] * [`character`, `char`, `chr`, `ch`, `c`]
  - [`in`, `r`, `i`] * [`char`, `chr`, `ch`, `c`]
  - [`red`, `re`, `rd`, `inp`] * [`chr`, `ch`, `c`]
  - [`char`, `chr`, `ch`, `c`] * [`in`]
- `readi`:
  - [`read`, `input`, `get`] * [`integer`, `int`, `i`, `number`, `num`, `n`]
  - [`red`, `re`, `rd`, `r`, `inp`, `in`] * [`int`, `i`, `num`, `n`]
  - [`i`] * [`int`, `i`, `num`]
  - [`int`, `i`, `num`, `n`] * [`in`]
- `shuffle`:
  - `shuffle`
  - `permr`
- `dumpstack`:
  - `dumpstack`
  - `debugprintstack`
- `dumpheap`:
  - `dumpheap`
  - `debugprintheap`
- `dumptrace`:
  - `dumptrace`
  - `trace`