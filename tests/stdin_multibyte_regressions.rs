use std::io::Write;
use std::process::{Command, Stdio};

/// Pipe `input` into `rubash` and return raw stdout bytes. Arguments select
/// script mode (`-c 'script'`) or bare-stdin mode (no args).
fn run_with_stdin(args: &[&str], input: &[u8]) -> Vec<u8> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rubash"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn rubash");
    child
        .stdin
        .as_mut()
        .expect("piped stdin")
        .write_all(input)
        .expect("write stdin");
    drop(child.stdin.take());
    child.wait_with_output().expect("wait rubash").stdout
}

#[test]
fn piped_script_source_preserves_multibyte_text() {
    // `rubash` reading script source from a pipe used to widen each byte
    // with `byte as char`, turning `中文` into `ä¸­æ–‡`.
    let stdout = run_with_stdin(&[], "echo 中文测试\n".as_bytes());
    assert_eq!(stdout, "中文测试\n".as_bytes());
}

#[test]
fn read_n_from_process_stdin_counts_characters() {
    // `read -n1` must consume one whole UTF-8 character; widening bytes made
    // it return a Latin-1-mangled lead byte and leave the rest behind.
    let stdout = run_with_stdin(
        &[
            "-c",
            "read -n1 a; read -n1 b; read rest; echo \"[$a][$b][$rest]\"",
        ],
        "中x\n".as_bytes(),
    );
    assert_eq!(stdout, "[中][x][]\n".as_bytes());
}

#[test]
fn read_line_from_process_stdin_preserves_multibyte_text() {
    let stdout = run_with_stdin(
        &["-c", "IFS= read -r x; echo \"[$x]\""],
        "中文 tail\n".as_bytes(),
    );
    assert_eq!(stdout, "[中文 tail]\n".as_bytes());
}

#[test]
fn read_from_process_stdin_roundtrips_invalid_utf8_byte() {
    // A lone 0x80 is not valid UTF-8; GNU bash keeps the raw byte. The
    // raw-byte marker path must re-emit it unchanged.
    let stdout = run_with_stdin(&["-c", "IFS= read -r x; echo \"[$x]\""], b"\x80\n");
    assert_eq!(stdout, b"[\x80]\n");
}

#[test]
fn read_d_delimiter_matches_decoded_char_not_byte() {
    // The delimiter must be compared against the decoded character; a
    // multibyte delimiter used to match (or slice) mid-sequence.
    let stdout = run_with_stdin(
        &[
            "-c",
            "read -d '中' x; echo \"[$x]\"; read rest; echo \"<$rest>\"",
        ],
        "ab中cd\n".as_bytes(),
    );
    assert_eq!(stdout, "[ab]\n<cd>\n".as_bytes());
}
