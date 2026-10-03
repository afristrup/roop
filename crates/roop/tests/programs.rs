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
fn files_can_be_written_read_appended_renamed_and_removed() {
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
    ancilla before: i64 = 0;
    ancilla wrote: i64 = 0;
    ancilla appended: i64 = 0;
    ancilla bytes: i64 = 0;
    ancilla text: [u8; 64] = 0;
    ancilla len: i64 = 0;
    ancilla back: [u8; 64] = 0;
    ancilla back_len: i64 = 0;
    ancilla read_ok: i64 = 0;
    ancilla moved: i64 = 0;
    ancilla old_gone: i64 = 0;
    ancilla new_here: i64 = 0;
    ancilla removed: i64 = 0;
    ancilla still: i64 = 0;
    ancilla again: [u8; 8] = 0;
    ancilla again_len: i64 = 0;
    ancilla again_status: i64 = 0;
    ancilla out: [u8; 256] = 0;
    ancilla n: i64 = 0;
    call arg(1, path, path_len);
    call arg(2, other, other_len);

    call exists(path, before);
    call put_int(out, n, before);
    call put(out, n, \" before\\n\");

    call put(text, len, \"hello\");
    call write_file(path, text, len, wrote);
    call put_int(out, n, wrote);
    call put(out, n, \" write\\n\");

    call append_file(path, \" there\", 6, appended);
    call size(path, bytes);
    call put_int(out, n, bytes);
    call put(out, n, \" bytes\\n\");

    call read_file(path, back, back_len, read_ok);
    call put_n(out, n, back, back_len);
    call put(out, n, \"\\n\");

    call rename(path, other, moved);
    call exists(path, old_gone);
    call exists(other, new_here);
    call put_int(out, n, old_gone);
    call put_int(out, n, new_here);
    call put(out, n, \" renamed\\n\");

    call remove(other, removed);
    call exists(other, still);
    call put_int(out, n, still);
    call read_file(other, again, again_len, again_status);
    call put(out, n, \" gone \");
    call put_int(out, n, again_status);
    call put(out, n, \"\\n\");
    call print_buf(out, n);
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
fn directories_are_made_and_removed() {
    let dir = scratch("dirs");
    let src = "
use std::fs::*;
use std::io::*;
use std::process::*;
use std::text::*;

irrev fn main(status: &mut i64) {
    ancilla path: [u8; 512] = 0;
    ancilla n: i64 = 0;
    ancilla made: i64 = 0;
    ancilla is_made: i64 = 0;
    ancilla gone: i64 = 0;
    ancilla is_gone: i64 = 0;
    ancilla out: [u8; 64] = 0;
    ancilla o: i64 = 0;
    call arg(1, path, n);
    call make_dir(path, made);
    call is_dir(path, is_made);
    call put_int(out, o, is_made);
    call remove_dir(path, gone);
    call is_dir(path, is_gone);
    call put_int(out, o, is_gone);
    call put(out, o, \"\\n\");
    call print_buf(out, o);
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
    ancilla unset: [u8; 32] = 0;
    ancilla unset_len: i64 = 0;
    ancilla ms: i64 = 0;
    ancilla out: [u8; 64] = 0;
    ancilla n: i64 = 0;
    call getenv(\"ROOP_TEST_VARIABLE\", value, len);
    call print_buf(value, len);
    call getenv(\"ROOP_TEST_UNSET_VARIABLE\", unset, unset_len);
    call put_int(out, n, unset_len);
    call put(out, n, \"\\n\");
    call print_buf(out, n);
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
fn exit_shows_what_is_pending_and_ends_the_program() {
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
fn reading_a_line_past_the_end_gives_status_one() {
    let src = "
use std::io::*;
use std::text::*;

irrev fn main() {
    ancilla first: [u8; 8] = 0;
    ancilla first_len: i64 = 0;
    ancilla first_status: i64 = 0;
    ancilla second: [u8; 8] = 0;
    ancilla second_len: i64 = 0;
    ancilla second_status: i64 = 0;
    ancilla out: [u8; 32] = 0;
    ancilla n: i64 = 0;
    call read_line(first, first_len, first_status);
    call read_line(second, second_len, second_status);
    call put_int(out, n, first_len);
    call put(out, n, \" \");
    call put_int(out, n, first_status);
    call put(out, n, \" \");
    call put_int(out, n, second_len);
    call put(out, n, \" \");
    call put_int(out, n, second_status);
    call put(out, n, \"\\n\");
    call print_buf(out, n);
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

irrev fn main() {
    ancilla line: [u8; 4] = 0;
    ancilla len: i64 = 0;
    ancilla status: i64 = 0;
    call read_line(line, len, status);
    call print_buf(line, len);
    call print(\"\\n\");
}
";
    let dir = project("longline", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "abcdefgh\n");
    assert_eq!(stdout(&out), "abcd\n", "{}", stderr(&out));
}

#[test]
fn a_failed_try_shows_nothing_of_what_it_wrote() {
    let out = example("undo", &[], "");
    assert_eq!(
        stdout(&out),
        "rolled back: nothing from announce was shown\n",
        "{}",
        stderr(&out)
    );
}

#[test]
fn output_is_shown_when_the_program_ends() {
    let src = "use std::io::*;\nirrev fn main() { call print(\"a\"); call print(\"b\\n\"); }";
    let dir = project("commit-at-end", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "");
    assert_eq!(stdout(&out), "ab\n");
}

#[test]
fn a_prompt_is_shown_before_the_program_waits_for_the_answer() {
    let src = "
use std::io::*;

irrev fn main() {
    ancilla line: [u8; 16] = 0;
    ancilla len: i64 = 0;
    ancilla status: i64 = 0;
    call print(\"name? \");
    call read_line(line, len, status);
    call print(\"hello, \");
    call print_buf(line, len);
    call print(\"\\n\");
}
";
    let dir = project("prompt", &config(), src);
    let mut child = Command::new(env!("CARGO_BIN_EXE_roop"))
        .current_dir(&dir)
        .env("ROOP_RT_LIB", support::runtime_lib())
        .arg("run")
        .arg("prog.roop")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut shown = [0u8; 6];
    std::io::Read::read_exact(child.stdout.as_mut().unwrap(), &mut shown).unwrap();
    assert_eq!(&shown, b"name? ");
    child.stdin.take().unwrap().write_all(b"Ada\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello, Ada\n");
}

#[test]
fn a_line_that_is_read_back_is_read_again() {
    let src = "
use std::io::*;

irrev fn main() {
    ancilla first: [u8; 16] = 0;
    ancilla first_len: i64 = 0;
    ancilla first_status: i64 = 0;
    ancilla again: [u8; 16] = 0;
    ancilla again_len: i64 = 0;
    ancilla again_status: i64 = 0;
    call read_line(first, first_len, first_status);
    uncall read_line(first, first_len, first_status);
    call read_line(again, again_len, again_status);
    call print_buf(again, again_len);
    call print(\"\\n\");
}
";
    let dir = project("unread", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "same line\nnext\n");
    assert_eq!(stdout(&out), "same line\n", "{}", stderr(&out));
}

#[test]
fn output_that_was_shown_cannot_be_taken_back() {
    let src = "
use std::io::*;

irrev fn main() {
    call print(\"shown\\n\");
    call commit();
    uncall print(\"shown\\n\");
}
";
    let dir = project("too-late", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "");
    assert_eq!(stdout(&out), "shown\n");
    assert_eq!(out.status.code(), Some(70));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("already shown"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn a_result_goes_into_a_place_that_is_zero() {
    let src = "
use std::io::*;

irrev fn main() {
    ancilla line: [u8; 4] = 0;
    ancilla len: i64 = 0;
    ancilla status: i64 = 0;
    call read_line(line, len, status);
    call read_line(line, len, status);
}
";
    let dir = project("not-zero", &config(), src);
    let out = run_in(&dir, "prog.roop", &[], "a\nb\n");
    assert_eq!(out.status.code(), Some(70));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("must be zero"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn the_world_tests_leave_the_disk_as_they_found_it() {
    let out = roop(
        &library(),
        &["test", "tests/files.roop", "tests/world.roop"],
    );
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
    let leftovers: Vec<_> = std::fs::read_dir(library())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[test]
fn the_examples_are_formatted() {
    let out = roop(
        &library(),
        &["fmt", "--check", "examples/bin", "tests", "std"],
    );
    assert!(out.status.success(), "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn the_example_programs_need_no_irrev() {
    for entry in std::fs::read_dir(library().join("examples/bin")).unwrap() {
        let path = entry.unwrap().path();
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(
            !source
                .lines()
                .any(|l| !l.trim_start().starts_with("//") && l.contains("irrev")),
            "{} uses irrev",
            path.display()
        );
    }
}

const KEEPER: &str = "use std::process::forget;

fn main() {
    ancilla n: i64 = 0;
    from n == 0 {
        auto ancilla big: [u8; 400] = 0;
        big[399] += 1;
    } loop { n += 1; } until n == 50;
    keep n;
}
";

fn limited(name: &str, limit: Option<u64>, source: &str) -> Output {
    let world = limit.map_or(String::new(), |l| format!("[world]\nhistory_limit = {l}\n"));
    let dir = project(name, &format!("{}{world}", config()), source);
    run_in(&dir, "prog.roop", &[], "")
}

#[test]
fn the_history_is_unlimited_unless_roop_toml_says_otherwise() {
    let out = limited("history-free", None, KEEPER);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn a_keep_over_the_history_limit_stops_the_program() {
    let out = limited("history-limit", Some(2000), KEEPER);
    assert_eq!(out.status.code(), Some(70), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("history of kept values"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn forgetting_each_round_keeps_the_history_under_the_limit() {
    let source = KEEPER.replace(
        "big[399] += 1;",
        "big[399] += 1;\n        irrev { call forget(); }",
    );
    let out = limited("history-forget", Some(2000), &source);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn bank_rolls_a_failed_transfer_back() {
    let out = example("bank", &[], "");
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "not enough money: rolled back\n100 0\n");
}
