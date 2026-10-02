mod support;

use roop_modules::ModuleError;
use support::{fn_names, load, project};

const TOML: &str = "[modules]\nstd = \"std\"\n";

#[test]
fn imports_from_a_roop_toml_module_and_keeps_only_what_is_used() {
    let dir = project(
        "std",
        &[
            ("Roop.toml", TOML),
            ("std/mod.roop", "mod blas;"),
            (
                "std/blas.roop",
                "pub fn add(a: &mut i64, b: &i64) { a += b; }
                 pub fn unused(a: &mut i64) { a += 1; }",
            ),
            (
                "prog.roop",
                "use std::blas::add;
                 fn main(x: &mut i64, y: &i64) { call add(x, y); }",
            ),
        ],
    );
    let program = load(&dir).unwrap();
    assert_eq!(fn_names(&program), ["std__blas__add", "main"]);
    roop_check::check(&program).unwrap();
}

#[test]
fn private_items_cannot_be_imported() {
    let dir = project(
        "private",
        &[
            ("Roop.toml", TOML),
            ("std/mod.roop", "fn hidden(a: &mut i64) { a += 1; }"),
            (
                "prog.roop",
                "use std::hidden; fn f(x: &mut i64) { call hidden(x); }",
            ),
        ],
    );
    assert!(matches!(load(&dir), Err(ModuleError::Private { .. })));
}

#[test]
fn aliases_resolve_clashes() {
    let dir = project(
        "alias",
        &[
            ("Roop.toml", TOML),
            ("std/mod.roop", "pub fn bump(a: &mut i64) { a += 1; }"),
            (
                "prog.roop",
                "use std::bump as library_bump;
                 fn bump(x: &mut i64) { x += 2; }
                 fn f(x: &mut i64) { call bump(x); call library_bump(x); }",
            ),
        ],
    );
    let program = load(&dir).unwrap();
    assert_eq!(fn_names(&program), ["std__bump", "bump", "f"]);
}

#[test]
fn a_name_defined_and_imported_twice_is_rejected() {
    let dir = project(
        "dup",
        &[
            ("Roop.toml", TOML),
            ("std/mod.roop", "pub fn bump(a: &mut i64) { a += 1; }"),
            (
                "prog.roop",
                "use std::bump; fn bump(x: &mut i64) { x += 2; }",
            ),
        ],
    );
    assert!(matches!(load(&dir), Err(ModuleError::Duplicate { .. })));
}

#[test]
fn mod_declarations_load_sibling_files_and_nested_directories() {
    let dir = project(
        "nested",
        &[
            (
                "prog.roop",
                "mod util; use util::deep::leaf; fn f(x: &mut i64) { call leaf(x); }",
            ),
            ("util/mod.roop", "mod deep;"),
            ("util/deep.roop", "pub fn leaf(a: &mut i64) { a += 1; }"),
        ],
    );
    let program = load(&dir).unwrap();
    assert_eq!(fn_names(&program), ["util__deep__leaf", "f"]);
}

#[test]
fn imported_structs_keep_their_constructors_and_enums_their_variants() {
    let dir = project(
        "types",
        &[
            ("Roop.toml", TOML),
            (
                "std/mod.roop",
                "pub struct Pair { a: i64, b: i64,
                     build(k: &i64) { self.a += k; }
                     unbuild(k: &i64) { self.a -= k; } }
                 pub enum Mode { Fast, Slow }",
            ),
            (
                "prog.roop",
                "use std::Pair; use std::Mode;
                 fn f(m: &Mode, x: &mut i64) {
                     match m { Mode::Fast => { x += 1; } assert x > 0; Mode::Slow => { x += 2; } assert x < 0; }
                 }
                 fn g(k: &i64, x: &mut i64) {
                     ancilla p: Pair = 0 { call Pair_build(p, k); x += p.a; call Pair_unbuild(p, k); }
                 }",
            ),
        ],
    );
    let program = load(&dir).unwrap();
    let text = format!("{program:?}");
    assert!(
        text.contains("std__Pair_build") && text.contains("std__Mode"),
        "{text}"
    );
}

#[test]
fn unknown_modules_and_items_are_reported() {
    let dir = project(
        "unknown",
        &[
            ("Roop.toml", TOML),
            ("std/mod.roop", ""),
            ("prog.roop", "use std::nothing;"),
        ],
    );
    assert!(matches!(load(&dir), Err(ModuleError::UnknownItem { .. })));
    let dir = project("missing", &[("prog.roop", "use nowhere::f;")]);
    assert!(matches!(load(&dir), Err(ModuleError::UnknownModule(_))));
}
