# GNU Bash Compatibility Test Suite

rubash vs GNU Bash (WSL) - .right file based testing

## Overview

This test suite verifies rubash compatibility against real GNU Bash running in WSL.
Unlike Git Bash (MSYS2), WSL provides a true GNU Bash environment.

## Versions

- **rubash**: self-reports `5.3.0(1)-release` (matches GNU Bash 5.3 semantics)
- **GNU Bash (WSL) oracle**: 5.3.0(1)-release (rights were originally generated under 5.2.21; re-verified byte-identical against 5.3.0 on 2026-09-13 across all 36 cases - zero oracle drift)

## .right Files Are Static Assets

The `.right` files under `rights/` were generated once from the WSL GNU bash
oracle and are never regenerated as a side effect of running tests. If a
`.right` file is missing, the runner fails loudly instead of silently
recreating it. To regenerate (e.g. after deliberately adding a test or
upgrading the oracle bash), run explicitly:

```bash
NIU_RIGHTS_REGEN=1 ./tests/gnu-compat/run-test.sh
```

Rationale: auto-generation bakes in whatever bash version currently lives in
WSL and swallows test-script errors into empty `.right` files that auto-skip -
both make results irreproducible.

## Running Tests

```bash
# Run all tests
./tests/gnu-compat/run-test.sh

# Run specific test
./tests/gnu-compat/run-test.sh braces-nested

# List all tests
./tests/gnu-compat/run-test.sh list
```

## Test Results (Current)

- **Pass**: 36/36 (100%) — re-verified 2026-09-13 against bash 5.3.0 semantics
- **Fail**: 0

### Formerly Known Bugs (fixed)

| Test | Issue | Fixed by |
|------|-------|----------|
| braces-nested | `{a,b}{1,2}` not expanding | brace expansion rework (commit 31c57d9c) |
| braces-triple | `{a,b}{1,2}{x,y}` not expanding | brace expansion rework (commit 31c57d9c) |

## File Structure

```
tests/gnu-compat/
├── run-test.sh          # Test runner
├── README.md            # This file
├── tests/               # Test scripts (.sh)
├── rights/              # Expected output (.right)
└── work/                # Test artifacts
```

## Adding Tests

1. Create a test script in `tests/`
2. Generate .right file: `./run-test.sh <test-name>`
3. Verify the .right file is correct
4. Run tests to verify

## Why WSL Instead of Git Bash?

Git Bash (MSYS2) has:
- Missing tools (recho, zecho, printenv)
- Non-standard exit codes
- MSYS2-specific patches

WSL provides real GNU Bash, making it the authoritative reference.

## CI Integration

This suite is designed for manual use, not CI. For CI, use the conformance suite:
```bash
./tests/conformance/runner.sh
```
