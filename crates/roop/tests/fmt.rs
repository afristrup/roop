mod support;

use std::io::Write;
use std::process::{Command, Stdio};
use support::{project, roop, stderr};

const MESSY: &str = "fn f(a:&mut i64){a+=1;}";
const TIDY: &str = "fn f(a: &mut i64) {\n    a += 1;\n}\n";

fn stdout(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn fmt_rewrites_the_files_of_the_project_in_place() {
    let dir = project("fmt-inplace", "", MESSY);
    let out = roop(&dir, &["fmt"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).contains("prog.roop"), "{}", stdout(&out));
    assert_eq!(
        std::fs::read_to_string(dir.join("prog.roop")).unwrap(),
        TIDY
    );
    let again = roop(&dir, &["fmt"]);
    assert!(stdout(&again).is_empty(), "{}", stdout(&again));
}

#[test]
fn fmt_check_lists_the_files_and_fails_without_touching_them() {
    let dir = project("fmt-check", "", MESSY);
    let out = roop(&dir, &["fmt", "--check"]);
    assert!(!out.status.success());
    assert!(stdout(&out).contains("prog.roop"));
    assert_eq!(
        std::fs::read_to_string(dir.join("prog.roop")).unwrap(),
        MESSY
    );
    std::fs::write(dir.join("prog.roop"), TIDY).unwrap();
    assert!(roop(&dir, &["fmt", "--check"]).status.success());
}

#[test]
fn fmt_takes_the_width_from_roop_toml_and_the_flag_overrides_it() {
    let src = "fn f(a: &mut i64) { call g(aaaaaaaa, bbbbbbbb, cccccccc); }";
    let dir = project("fmt-width", "[format]\nmax_width = 30\n", src);
    assert!(roop(&dir, &["fmt"]).status.success());
    let narrow = std::fs::read_to_string(dir.join("prog.roop")).unwrap();
    assert!(narrow.contains("call g(\n        aaaaaaaa,\n"), "{narrow}");
    assert!(roop(&dir, &["fmt", "--width", "100"]).status.success());
    let wide = std::fs::read_to_string(dir.join("prog.roop")).unwrap();
    assert!(
        wide.contains("call g(aaaaaaaa, bbbbbbbb, cccccccc);"),
        "{wide}"
    );
}

#[test]
fn fmt_stdin_filters_text() {
    let dir = project("fmt-stdin", "", "");
    let mut child = Command::new(env!("CARGO_BIN_EXE_roop"))
        .current_dir(&dir)
        .args(["fmt", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(MESSY.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(stdout(&out), TIDY);
}

#[test]
fn fmt_names_the_file_it_cannot_format() {
    let dir = project("fmt-error", "", "fn (");
    let out = roop(&dir, &["fmt"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("prog.roop"), "{}", stderr(&out));
}

#[test]
fn fmt_with_a_path_leaves_other_files_alone() {
    let dir = project("fmt-path", "", MESSY);
    std::fs::write(dir.join("other.roop"), MESSY).unwrap();
    assert!(roop(&dir, &["fmt", "other.roop"]).status.success());
    assert_eq!(
        std::fs::read_to_string(dir.join("other.roop")).unwrap(),
        TIDY
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("prog.roop")).unwrap(),
        MESSY
    );
}

#[test]
fn fmt_skips_a_repository_checked_out_inside_the_project() {
    let dir = project("fmt-nested", "", TIDY);
    let nested = dir.join("vendored");
    std::fs::create_dir_all(nested.join(".git")).unwrap();
    std::fs::write(nested.join("template.roop"), "fn f(x: &mut [i64; @N@]) {}").unwrap();
    let out = roop(&dir, &["fmt", "--check"]);
    assert!(out.status.success(), "{}", stderr(&out));
}
