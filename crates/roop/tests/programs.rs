mod support;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use support::{project, roop, stderr};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../roop")
}

fn config() -> String {
    format!("[modules]\nstd = \"{}\"\n", library().join("std").display())
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Runs `roop run <program> <args>` in `dir`, feeding `input` to the program.
fn run_in(dir: &Path, program: &str, args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_roop"))
        .current_dir(dir)
        .env("ROOP_RT_LIB", support::runtime_lib())
        .arg("run")
        .arg(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn example(name: &str, args: &[&str], input: &str) -> Output {
    run_in(
        &library(),
        &format!("examples/bin/{name}.roop"),
        args,
        input,
    )
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("roop-programs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn hello_world() {
    let dir = project(
        "hello",
        &config(),
        "use std::io::println;\nirrev fn main() { call println(\"Hello, world!\"); }",
    );
    let out = run_in(&dir, "prog.roop", &[], "");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "Hello, world!\n");
}

#[test]
fn the_status_main_leaves_is_the_exit_status() {
    let dir = project(
        "status",
        &config(),
        "irrev fn main(status: &mut i64) { status = 3; }",
    );
    let out = run_in(&dir, "prog.roop", &[], "");
    assert_eq!(out.status.code(), Some(3), "{}", stderr(&out));
}

#[test]
fn build_makes_an_executable_when_there_is_a_main() {
    let dir = project(
        "build-main",
        &config(),
        "use std::io::print;\nirrev fn main() { call print(\"built\\n\"); }",
    );
    let out = roop(&dir, &["build", "prog.roop", "-o", "prog"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let ran = Command::new(dir.join("prog")).output().unwrap();
    assert_eq!(stdout(&ran), "built\n");
}

#[test]
fn a_program_without_main_still_builds_an_object() {
    let dir = project("build-object", "", "fn f(a: &mut i64) { a += 1; }");
    let out = roop(&dir, &["build", "prog.roop"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(dir.join("prog.o").exists());
}

#[test]
fn arguments_reach_the_program() {
    let out = example("echo", &["a", "b", "c d"], "");
    assert_eq!(stdout(&out), "a b c d\n", "{}", stderr(&out));
    assert_eq!(stdout(&example("echo", &[], "")), "\n");
}

#[test]
fn cat_copies_standard_input() {
    let out = example("cat", &[], "one\ntwo\nthree\n");
    assert_eq!(stdout(&out), "one\ntwo\nthree\n", "{}", stderr(&out));
}

#[test]
fn cat_copies_files_and_reports_the_one_it_cannot_read() {
    let dir = scratch("cat");
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
    let (a, b, missing) = (
        dir.join("a.txt").display().to_string(),
        dir.join("b.txt").display().to_string(),
        dir.join("none.txt").display().to_string(),
    );
    let out = example("cat", &[&a, &missing, &b], "");
    assert_eq!(stdout(&out), "alpha\nbeta\n");
    assert!(String::from_utf8_lossy(&out.stderr).contains("cat: cannot read"));
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn wc_counts_lines_words_and_bytes() {
    let out = example("wc", &[], "hello world\nfoo  bar baz\n");
    assert_eq!(stdout(&out), "2 5 25\n", "{}", stderr(&out));
    assert_eq!(stdout(&example("wc", &[], "")), "0 0 0\n");
}

#[test]
fn rot13_rotates_and_rotates_back() {
    let out = example("rot13", &[], "Hello, World!\n");
    assert_eq!(stdout(&out), "Uryyb, Jbeyq!\n", "{}", stderr(&out));
    let back = example("rot13", &[], &stdout(&out));
    assert_eq!(stdout(&back), "Hello, World!\n");
}

#[test]
fn fib_prints_fibonacci_numbers() {
    for (n, expected) in [
        ("0", "0"),
        ("1", "1"),
        ("10", "55"),
        ("90", "2880067194370816120"),
    ] {
        let out = example("fib", &[n], "");
        assert_eq!(
            stdout(&out),
            format!("{expected}\n"),
            "fib {n}: {}",
            stderr(&out)
        );
    }
    let bad = example("fib", &["x"], "");
    assert_eq!(bad.status.code(), Some(2));
    assert_eq!(example("fib", &[], "").status.code(), Some(2));
}

#[test]
fn copy_copies_a_file() {
    let dir = scratch("copy");
    let (from, to) = (dir.join("from.bin"), dir.join("to.bin"));
    let data: Vec<u8> = (0..20000u32).map(|i| (i % 251) as u8).collect();
    std::fs::write(&from, &data).unwrap();
    let out = example(
        "copy",
        &[&from.display().to_string(), &to.display().to_string()],
        "",
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(std::fs::read(&to).unwrap(), data);
}

#[test]
fn ls_lists_a_directory_in_order() {
    let dir = scratch("ls");
    for name in ["b.txt", "a.txt", "c.txt"] {
        std::fs::write(dir.join(name), "").unwrap();
    }
    let out = example("ls", &[&dir.display().to_string()], "");
    assert_eq!(stdout(&out), "a.txt\nb.txt\nc.txt\n", "{}", stderr(&out));
    let missing = example("ls", &[&dir.join("nope").display().to_string()], "");
    assert_eq!(missing.status.code(), Some(1));
}

#[test]
fn files_can_be_written_read_appended_listed_renamed_and_removed() {
    let dir = scratch("fs");
    let src = "
use std::fs::*;
use std::io::*;
use std::process::*;
use std::text::*;

irrev fn main(status: &mut i64) {
    ancilla path: [u8; 512] = 0;
    ancilla path_len: i64 = 0;
    ancilla other: [u8; 512] = 0;
    ancilla other_len: i64 = 0;
    ancilla found: i64 = 0;
    ancilla bytes: i64 = 0;
    ancilla failed: i64 = 0;
    ancilla text: [u8; 64] = 0;
    ancilla len: i64 = 0;
    ancilla back: [u8; 64] = 0;
    ancilla back_len: i64 = 0;
    ancilla out: [u8; 256] = 0;
    ancilla n: i64 = 0;
    call arg(1, path, path_len);
    call arg(2, other, other_len);

    call exists(path, found);
    call put_int(out, n, found);
    call put(out, n, \" before\\n\");

    call put(text, len, \"hello\");
    call write_file(path, text, len, failed);
    call put_int(out, n, failed);
    call put(out, n, \" write\\n\");

    call append_file(path, \" there\", 6, failed);
    call size(path, bytes);
    call put_int(out, n, bytes);
    call put(out, n, \" bytes\\n\");

    call read_file(path, back, back_len, failed);
    call put_n(out, n, back, back_len);
    call put(out, n, \"\\n\");

    call rename(path, other, failed);
    call exists(path, found);
    call put_int(out, n, found);
    call exists(other, found);
    call put_int(out, n, found);
    call put(out, n, \" renamed\\n\");

    call remove(other, failed);
    call exists(other, found);
    call put_int(out, n, found);
    call read_file(other, back, back_len, failed);
    call put(out, n, \" gone \");
    call put_int(out, n, failed);
    call put(out, n, \"\\n\");
    call flush(out, n);
}
";
    let project_dir = project("fs-ops", &config(), src);
    let (a, b) = (dir.join("a.txt"), dir.join("b.txt"));
    let out = run_in(
        &project_dir,
        "prog.roop",
        &[&a.display().to_string(), &b.display().to_string()],
        "",
    );
    assert_eq!(
        stdout(&out),
        "0 before\n0 write\n11 bytes\nhello there\n01 renamed\n0 gone -2\n",
        "{}",
        stderr(&out)
    );
    assert!(!a.exists() && !b.exists());
}

#[test]
fn directories_are_made_listed_and_removed() {
    let dir = scratch("dirs");
    let src = "
use std::fs::*;
use std::io::*;
use std::process::*;
use std::text::*;

irrev fn main(status: &mut i64) {
    ancilla path: [u8; 512] = 0;
    ancilla n: i64 = 0;
    ancilla found: i64 = 0;
    ancilla failed: i64 = 0;
    ancilla out: [u8; 64] = 0;
    ancilla o: i64 = 0;
    call arg(1, path, n);
    call make_dir(path, failed);
    call is_dir(path, found);
    call put_int(out, o, found);
    call remove_dir(path, failed);
    call is_dir(path, found);
    call put_int(out, o, found);
    call put(out, o, \"\\n\");
    call flush(out, o);
}
";
    let project_dir = project("fs-dirs", &config(), src);
    let nested = dir.join("x").join("y");
    let out = run_in(
        &project_dir,
        "prog.roop",
        &[&nested.display().to_string()],
        "",
    );
    assert_eq!(stdout(&out), "10\n", "{}", stderr(&out));
    assert!(dir.join("x").exists() && !nested.exists());
}

#[test]
fn environment_and_time_are_available() {
    let src = "
use std::io::*;
use std::process::*;
use std::text::*;

irrev fn main(status: &mut i64) {
    ancilla value: [u8; 32] = 0;
    ancilla len: i64 = 0;
    ancilla ms: i64 = 0;
    ancilla out: [u8; 64] = 0;
    ancilla n: i64 = 0;
    call getenv(\"ROOP_TEST_VARIABLE\", value, len);
    call print_buf(value, len);
    call getenv(\"ROOP_TEST_UNSET_VARIABLE\", value, len);
    call put_int(out, n, len);
    call put(out, n, \"\\n\");
    call flush(out, n);
    call now_ms(ms);
    if ms > 1600000000000 { call print(\"clock ok\\n\"); } fi ms > 1600000000000;
}
";
    let dir = project("env", &config(), src);
    let out = Command::new(env!("CARGO_BIN_EXE_roop"))
        .current_dir(&dir)
        .env("ROOP_RT_LIB", support::runtime_lib())
        .env("ROOP_TEST_VARIABLE", "set")
        .args(["run", "prog.roop"])
        .output()
        .unwrap();
    assert_eq!(stdout(&out), "set-1\nclock ok\n", "{}", stderr(&out));
}

#[test]
fn exit_ends_the_program_at_once() {
    let src = "
use std::io::*;
use std::process::*;

irrev fn main() {
    call print(\"before\\n\");
    call exit(7);
    call print(\"after\\n\");
}
";
    let dir = project("exit", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "");
    assert_eq!(stdout(&out), "before\n");
    assert_eq!(out.status.code(), Some(7));
}

#[test]
fn reading_a_line_past_the_end_sets_eof() {
    let src = "
use std::io::*;
use std::text::*;

irrev fn main() {
    ancilla line: [u8; 8] = 0;
    ancilla len: i64 = 0;
    ancilla eof: i64 = 0;
    ancilla out: [u8; 32] = 0;
    ancilla n: i64 = 0;
    call read_line(line, len, eof);
    call put_int(out, n, len);
    call put(out, n, \" \");
    call put_int(out, n, eof);
    call put(out, n, \" \");
    call read_line(line, len, eof);
    call put_int(out, n, len);
    call put(out, n, \" \");
    call put_int(out, n, eof);
    call put(out, n, \"\\n\");
    call flush(out, n);
}
";
    let dir = project("eof", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "abc\n");
    assert_eq!(stdout(&out), "3 0 0 1\n", "{}", stderr(&out));
}

#[test]
fn a_long_line_is_cut_to_the_buffer() {
    let src = "
use std::io::*;
use std::text::*;

irrev fn main() {
    ancilla line: [u8; 4] = 0;
    ancilla len: i64 = 0;
    ancilla eof: i64 = 0;
    call read_line(line, len, eof);
    call print_buf(line, len);
    call print(\"\\n\");
}
";
    let dir = project("longline", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "abcdefgh\n");
    assert_eq!(stdout(&out), "abcd\n", "{}", stderr(&out));
}

#[test]
fn the_examples_are_formatted() {
    let out = roop(
        &library(),
        &["fmt", "--check", "examples/bin", "tests", "std"],
    );
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
}
