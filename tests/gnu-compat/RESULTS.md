# GNU Bash Compatibility Test Results

## Date: 2026-09-13 (re-run; original 2026-08-28)

## Summary

| Metric | Result |
|--------|--------|
| **Pass rate** | **100% (36/36)** |
| Known bugs | 0 |
| Total tests | 36 |
| Oracle | WSL GNU bash 5.3.0 (rights generated under 5.2.21, verified byte-identical on 5.3.0) |

## Test Results

### Passing (34)

- arith-assign, arith-expr, arithmetic-bool, array-basic
- brace-expand-assign, braces-char-range, braces-prefix, braces-range
- braces-simple, braces-step, case-basic, cmdsub-backtick
- cmdsub-dollar, exit-status, for-loop, function-basic
- glob-star, heredoc-basic, if-else, nested-cmdsub
- param-assign, param-default, param-length, param-prefix
- param-replace, param-suffix, pipe-basic, process-sub
- redirect-stdout, set-e, shopt-pipefail, string-compare
- while-loop, word-split

### Failing (0)

The two brace expansion bugs below were fixed by the brace expansion rework
(commit 31c57d9c, mkseq i64 overflow bound in ffc14d9c). Verified 2026-09-13:
rubash output is byte-identical to WSL GNU bash 5.3.0 for both cases.

| Test | Was broken | Now |
|------|----------|-----|
| braces-nested | `{a,b}{1,2}` expanded as `a b 1 2` | PASS (`a1 a2 b1 b2`) |
| braces-triple | `{a,b}{1,2}{x,y}` expanded flat | PASS (`a1x a1y ... b2x b2y`) |

## Conclusion

rubash is **100% compatible** with GNU Bash on this suite, and matches
GNU bash 5.3.0 output byte-for-byte on all 36 cases.
