#!/usr/bin/env bash
set -eEuo pipefail

# Get $(LIME_WSA) from the Makefile
lwsa="$(make -f <(echo $'include Makefile\nget-lwsa:\n\t@echo $(LIME_LWSA)') get-lwsa)"

rm -rf fail/tokens/{label,macro}_{first,rest} pass/tokens/{label_{first,rest}_part*,macro_{first,rest}}.wsa
mkdir -p fail/tokens/{label,macro}_{first,rest} pass/tokens

test_label() {
  name="$1"
  prefix="$2"
  comment='; This test is split into parts to sidestep bugs in the hash map.'
  part=1
  echo "$comment" > "pass/tokens/${name}_part1.wsa"
  for i in $(seq 0 255); do
    b="$(printf '\\x%02x' "$i")"
    printf "jmp .$prefix$b%s .$prefix$b: push $i\n" > "byte_$i.wsa"
    if "$lwsa" "byte_$i.wsa" /dev/null >/dev/null 2>&1; then
      # Try to combine it with the current part. If it fails, start a new part.
      cat "pass/tokens/${name}_part$part.wsa" "byte_$i.wsa" > curr.wsa
      if "$lwsa" curr.wsa /dev/null >/dev/null 2>&1; then
        mv curr.wsa "pass/tokens/${name}_part$part.wsa"
      else
        part=$((part+1))
        rm curr.wsa
        cat <(echo "$comment") "byte_$i.wsa" > "pass/tokens/${name}_part$part.wsa"
      fi
      rm "byte_$i.wsa"
    else
      mv "byte_$i.wsa" "fail/tokens/$name/"
    fi
  done
}

test_label label_first ''
test_label label_rest l

test_macro() {
  name="$1"
  prefix="$2"
  for i in $(seq 0 255); do
    if [[ $i = 9 ]] || [[ $i = 10 ]] || [[ $i == 32 ]]; then
      continue
    fi
    b="$(printf '\\x%02x' "$i")"
    printf "macro $prefix$b%s [push $i] $prefix$b\n" > "byte_$i.wsa"
    if "$lwsa" "byte_$i.wsa" /dev/null >/dev/null 2>&1; then
      cat "byte_$i.wsa" >> "pass/tokens/$name.wsa"
      rm "byte_$i.wsa"
    else
      mv "byte_$i.wsa" "fail/tokens/$name/"
    fi
  done
}

test_macro macro_first ''
test_macro macro_rest m
