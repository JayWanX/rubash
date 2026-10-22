//! The assignment fast path in `executor/assignment_expansion.rs` must not run
//! the restored value back through the `bytes_to_shell_text` /
//! `shell_text_to_raw_bytes` boundary. That round-trip treats every control
//! byte as data and re-encodes it as a U+E000 marker pair, which also swept the
//! lexer's marker-role C0 chars (0x17 hoisted quote, 0x14 backslash) into data
//! pairs. The downstream quote-restore passes then found no 0x17 to turn back
//! into a literal quote, so the character was dropped.
//!
//! Data bytes coming from `$'...'` must stay as U+E000 pairs (that is how the
//! ANSI-C decoder emits them) and be decoded at their own exact-once boundary,
//! so the fix must not lose those either.

use std::process::Command;

fn rubash_raw(script: &str) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_rubash"))
        .arg("-c")
        .arg(script)
        .output()
        .expect("run rubash");
    output.stdout
}

fn rubash(script: &str) -> String {
    String::from_utf8_lossy(&rubash_raw(script)).into_owned()
}

#[test]
fn quoted_assignment_survives_pattern_replacement() {
    // quote.tests 77-80/88: the hoisted single quotes must be restored, not
    // swallowed by the round-trip.
    assert_eq!(rubash("x=\"a'b'c\"\necho \"${x//\\'/\\'}\""), "a'b'c\n");
}

#[test]
fn quoted_assignment_reads_back_verbatim() {
    assert_eq!(rubash("x=\"a'b'c\"\necho \"$x\""), "a'b'c\n");
}

#[test]
fn ansi_c_control_bytes_stay_raw_at_their_boundary() {
    // The bytes must survive as data, not be re-encoded into extra markers.
    assert_eq!(
        rubash_raw("printf '%s' $'\\x11\\x16'"),
        b"\x11\x16".to_vec()
    );
}

// niubash issue #103: the lexer's \x18 carrier for `\"` inside double quotes
// was not restored by the quoted-assignment fast path, so the stored value
// kept a raw CAN (0x18) byte: length intact, rc 0, silent corruption that
// survives into redirected files. GNU bash stores the literal `"` (0x22).
#[test]
fn escaped_double_quote_in_assignment_is_literal_quote() {
    assert_eq!(
        rubash_raw("x=\"q\\\"q\"; printf '%s' \"$x\""),
        b"q\"q".to_vec()
    );
}

#[test]
fn multiple_escaped_double_quotes_all_restore() {
    assert_eq!(
        rubash_raw("x=\"a\\\"b\\\"c\"; printf '%s' \"$x\""),
        b"a\"b\"c".to_vec()
    );
}

#[test]
fn escaped_double_quote_survives_expansion_and_heredoc() {
    assert_eq!(
        rubash("p=\"a\\\"b\"\nx=\"$p-$p\"\necho \"[$x]\""),
        "[a\"b-a\"b]\n"
    );
    assert_eq!(rubash("x=\"q\\\"q\"\necho \"[${x:-none}]\""), "[q\"q]\n");
    assert_eq!(rubash("x=\"q\\\"q\"\ncat <<EOF\n[$x]\nEOF"), "[q\"q]\n");
}

// niubash #103 side finding: compound array elements ran quote removal
// twice -- escape decoding (`\"` -> data `"`) followed by a second
// remove_shell_quotes pass that re-parsed the decoded DATA quote as an
// operator pair and stripped it. GNU bash 5.3.0(1) stores the literal
// quote byte (0x22) for all three element quote forms.
#[test]
fn escaped_quote_in_unquoted_array_element_is_literal() {
    assert_eq!(
        rubash_raw("x=(q\\\"q); printf '%s' \"${x[0]}\""),
        b"q\"q".to_vec()
    );
}

#[test]
fn escaped_quote_in_quoted_array_element_is_literal() {
    assert_eq!(
        rubash_raw("x=(\"q\\\"q\"); printf '%s' \"${x[0]}\""),
        b"q\"q".to_vec()
    );
}

// The storage render (quote_array_value -> sh_double_quote) must escape a
// bare `"` with a SINGLE backslash; the previous `\\"` doubled it and the
// element re-decode surfaced a spurious backslash byte.
#[test]
fn array_element_quote_storage_roundtrip() {
    assert_eq!(
        rubash_raw("x=('a\"b'); printf '%s' \"${x[0]}\""),
        b"a\"b".to_vec()
    );
    assert_eq!(
        rubash_raw("x=(\"a\\\"b\"); printf '%s' \"${x[0]}\""),
        b"a\"b".to_vec()
    );
}
