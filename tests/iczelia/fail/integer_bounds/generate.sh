#!/usr/bin/env bash
set -eEuo pipefail

write_tests() {
  local value="$1" label="$2"
  mkdir -- "$label"
  pushd -- "$label" >/dev/null
  echo "psh $value" > "psh.asm"
  echo "$value" > "psh_bare.asm"
  echo "copy $value" > "copy.asm"
  echo "slide $value" > "slide.asm"
  echo "add $value" > "add.asm"
  echo "sub $value" > "sub.asm"
  echo "mul $value" > "mul.asm"
  echo "div $value" > "div.asm"
  echo "mod $value" > "mod.asm"
  echo "sto $value" > "sto_one.asm"
  echo "sto 0, $value" > "sto_rhs.asm"
  echo "sto $value, 0" > "sto_lhs.asm"
  echo "sto $value, $value" > "sto_both.asm"
  echo "rcl $value" > "rcl.asm"
  echo "putc $value" > "putc.asm"
  echo "putn $value" > "putn.asm"
  echo "getc $value" > "getc.asm"
  echo "getn $value" > "getn.asm"
  echo "rep dup $value" > "rep_dup.asm"
  echo "rep drop $value" > "rep_drop.asm"
  echo "rep add $value" > "rep_add.asm"
  echo "rep sub $value" > "rep_sub.asm"
  echo "rep mul $value" > "rep_mul.asm"
  echo "rep div $value" > "rep_div.asm"
  echo "rep mod $value" > "rep_mod.asm"
  echo "rep putc $value" > "rep_putc.asm"
  echo "rep putn $value" > "rep_putn.asm"
  popd >/dev/null
}

write_tests -100000000h '-2^32'
write_tests  100000000h  '2^32'
write_tests -0ffffffffh '-2^32+1'
write_tests   80000000h  '2^31'
write_tests  0ffffffffh  '2^32-1'
