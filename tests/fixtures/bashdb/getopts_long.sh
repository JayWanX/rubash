#!/usr/bin/env bash
# Minimal fixture used by the Rubash getopts_long compatibility probe.
getopts_long() {
  local _optstring=$1
  local _name=$2
  local _option=$3
  local _arg=$4
  local _unused=$5
  local _argv=$6
  # Emulate the real getopts_long termination contract: consume the
  # OPTLIND-th trailing argument while it still looks like a --option,
  # then return non-zero so `while getopts_long ...` loops end. A stub
  # that always returns 0 spins the caller's while loop forever (the
  # CI Rust-tests job burned the full 6h job timeout on this).
  shift 5
  eval "local _current=\${$OPTLIND}"
  if [[ $_current != --* ]]; then
    return 1
  fi
  printf -v "${_name}" '%s' "${_option#--}"
  OPTLARG=''
  OPTLIND=$((OPTLIND + 1))
  return 0
}
