use roop_config::FormatConfig;
use roop_fmt::{FmtError, format_source};

fn fmt(src: &str) -> String {
    format_source(src, &FormatConfig::default()).unwrap()
}

fn fmt_width(src: &str, max_width: usize) -> String {
    let config = FormatConfig {
        max_width,
        ..FormatConfig::default()
    };
    format_source(src, &config).unwrap()
}

#[test]
fn indents_and_spaces_a_function() {
    let out = fmt("fn f(x:&mut i64,y:&i64){x+=*y;}"
        .replace("*y", "y")
        .as_str());
    assert_eq!(out, "fn f(x: &mut i64, y: &i64) {\n    x += y;\n}\n");
}

#[test]
fn formatting_twice_changes_nothing() {
    let src = "fn f(a: &mut [i64; 4]) { ancilla i: i64 = 0 { from i == 0 { a[i] += 1; } loop { i += 1; } until i == 3; i -= 3; } }";
    let once = fmt(src);
    assert_eq!(fmt(&once), once);
}

#[test]
fn a_long_call_wraps_one_argument_to_a_line() {
    let src = "fn f(a: &mut i64) { call kernel(aaaaaaaaaaaa, bbbbbbbbbbbb, cccccccccccc, dddddddddddd, eeeeeeeeeeee); }";
    let out = fmt_width(src, 40);
    assert!(
        out.contains("call kernel(\n        aaaaaaaaaaaa,\n"),
        "{out}"
    );
    assert!(out.contains("eeeeeeeeeeee,\n    );"), "{out}");
    assert!(out.lines().all(|l| l.len() <= 40), "{out}");
}

#[test]
fn a_long_signature_wraps_its_parameters_and_keeps_generics_whole() {
    let src = "fn f<N, M>(first: &mut [i64; N], second: &[i64; M], third: &i64) { }";
    let out = fmt_width(src, 50);
    assert!(
        out.starts_with("fn f<N, M>(\n    first: &mut [i64; N],\n"),
        "{out}"
    );
    assert!(out.ends_with("    third: &i64,\n) {}\n"), "{out}");
}

#[test]
fn a_long_expression_breaks_before_an_operator() {
    let src = "fn f(a: &mut i64, b: &i64) { a += b * b * b + b * b * b + b * b * b + b * b * b; }";
    let out = fmt_width(src, 40);
    assert!(out.contains("a += b * b * b\n        + b * b * b"), "{out}");
}

#[test]
fn parentheses_stay_where_the_meaning_needs_them() {
    let src = "fn f(a: &mut i64, b: &i64, c: &i64) { a += (b + c) * c; a += b - (c - b); a += -(b + c); a += b + c + b; }";
    let out = fmt(src);
    assert!(out.contains("a += (b + c) * c;"), "{out}");
    assert!(out.contains("a += b - (c - b);"), "{out}");
    assert!(out.contains("a += -(b + c);"), "{out}");
    assert!(out.contains("a += b + c + b;"), "{out}");
}

#[test]
fn drops_parentheses_that_mean_nothing() {
    let out = fmt("fn f(a: &mut i64, b: &i64) { a += ((b)) + (b * b); }");
    assert!(out.contains("a += b + b * b;"), "{out}");
}

#[test]
fn floats_print_so_they_lex_again() {
    let out = fmt("fn f(a: &mut f64) { a += 0.0000001; a += 2.0; a += 1.5; }");
    assert!(out.contains("a += 0.0000001;"), "{out}");
    assert!(out.contains("a += 2.0;"), "{out}");
}

#[test]
fn an_empty_else_and_an_empty_loop_are_left_out() {
    let src = "fn f(a: &mut i64) { if true { a += 1; } else { } fi true; from a == 0 { a += 1; } loop { } until a == 1; }";
    let out = fmt(src);
    assert!(out.contains("if true { a += 1; } fi true;"), "{out}");
    assert!(
        out.contains("from a == 0 { a += 1; } until a == 1;"),
        "{out}"
    );
}

#[test]
fn a_block_of_one_simple_statement_collapses_when_it_fits() {
    let src = "fn f(a: &mut i64) { if true {\n a += 1;\n } else {\n a -= 1;\n } fi true; }";
    let out = fmt(src);
    assert!(
        out.contains("if true { a += 1; } else { a -= 1; } fi true;"),
        "{out}"
    );
    let narrow = fmt_width(src, 30);
    assert!(
        narrow
            .contains("if true {\n        a += 1;\n    } else {\n        a -= 1;\n    } fi true;"),
        "{narrow}"
    );
}

#[test]
fn a_from_loop_keeps_its_step_on_the_closing_line() {
    let src = "fn f(a: &mut [i64; 9], i: &mut i64) { from i == 0 { a[i] += 1; a[i] += 2; } loop { i += 1; } until i == 8; }";
    let out = fmt(src);
    assert!(
        out.contains("    } loop { i += 1; } until i == 8;"),
        "{out}"
    );
}

#[test]
fn comments_stay_above_and_beside_their_statements() {
    let src = "// header\n\n// about f\nfn f(a: &mut i64) {\n    // first\n    a += 1; // beside\n\n    a += 2;\n    // last\n}\n// tail\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn a_comment_after_the_brace_stays_there() {
    let src = "fn f(a: &mut i64) { // opens\n    a += 1;\n}\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn a_comment_forces_its_block_open() {
    let src = "fn f(a: &mut i64) { if true { a += 1; // why\n } fi true; }";
    let out = fmt(src);
    assert!(
        out.contains("if true {\n        a += 1; // why\n    } fi true;"),
        "{out}"
    );
}

#[test]
fn a_comment_does_not_count_for_the_width() {
    let src = "fn f(a: &mut i64) {\n    a += 1; // a comment far longer than the line is wide, which must not break the code\n}\n";
    assert_eq!(fmt_width(src, 40), src);
}

#[test]
fn comments_in_match_arms_and_at_the_end_of_a_file_are_kept() {
    let src = "fn f(a: &mut i64) {\n    match a {\n        // small\n        0 => { a += 1; } assert true;\n        _ => { a += 2; } assert true;\n    }\n}\n// end\n";
    assert_eq!(fmt(src), src);
}

#[test]
fn a_file_of_only_comments_is_kept() {
    assert_eq!(fmt("// nothing here\n"), "// nothing here\n");
}

#[test]
fn imports_sit_together_and_declarations_are_set_apart() {
    let src = "mod a;\nmod b;\nuse a::x;\nfn f() {}\nfn g() {}";
    assert_eq!(
        fmt(src),
        "mod a;\nmod b;\nuse a::x;\n\nfn f() {}\n\nfn g() {}\n"
    );
}

#[test]
fn use_declarations_keep_their_shape() {
    let src = "pub use a::b::*;use a::{c, d as e};use a::f as g;";
    assert_eq!(
        fmt(src),
        "pub use a::b::*;\nuse a::{c, d as e};\nuse a::f as g;\n"
    );
}

#[test]
fn sessions_enums_and_structs_are_laid_out() {
    let src = "pub enum Color{Red,Green}struct P{x:i64,y:i64,build(x:&i64){}}pub session S{client:checkpoint select{a:end,b:offer{c:end}};}";
    let out = fmt(src);
    assert!(out.contains("pub enum Color { Red, Green }"), "{out}");
    assert!(
        out.contains("struct P {\n    x: i64,\n    y: i64,\n    build(x: &i64) {}\n}"),
        "{out}"
    );
    assert!(
        out.contains("client: checkpoint select { a: end, b: offer { c: end } };"),
        "{out}"
    );
}

#[test]
fn every_statement_form_survives() {
    let src = "
fn f(a: &mut i64, s: &mut Stack<i64, 4>, v: &mut [i64; N]) {
    #[parallel(cpu)]
    from a == 0 { a += 1; } until a == 1;
    swap_a: ;
}";
    assert!(format_source(src, &FormatConfig::default()).is_err());
    let src = "
fn f<N>(a: &mut i64, s: &mut Stack<i64, 4>, v: &mut [i64; N], k: &Kind) {
    #[parallel(cpu)]
    from a == 0 { a += 1; } until a == 1;
    a <=> a;
    a ^= 1;
    a *= 2;
    a /= 2;
    push s <- a;
    pop s -> a;
    borrow r = v[0] { a += r; }
    chan c: i64 { send c <- a; recv c -> a; }
    logged s { a = 1; a %= 2; }
    irrev { a = 3; }
    try { a += 1; } catch_rollback { a -= 1; } -> a;
    #[concurrent]
    { a += 1; }
    call g<N, 3>(a);
    uncall g<N, 3>(a);
    match k { Kind::A => { a += 1; } assert true; _ => { } assert true; }
    ancilla t: [i64; 3] = 0 { t[0] += a; t[0] -= a; }
    a += !true && false || true;
    a += -a;
}";
    let out = fmt(src);
    assert_eq!(fmt(&out), out);
}

#[test]
fn a_comment_inside_a_statement_header_is_refused() {
    let err = format_source(
        "fn f(a: &mut i64) { call g(a, // x\n a); }",
        &FormatConfig::default(),
    );
    assert!(matches!(err, Err(FmtError::Comment(1))), "{err:?}");
}

#[test]
fn a_syntax_error_is_reported() {
    assert!(matches!(
        format_source("fn (", &FormatConfig::default()),
        Err(FmtError::Parse(_))
    ));
}

#[test]
fn the_library_formats_to_itself() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../roop");
    let mut seen = 0;
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "roop") {
                let src = std::fs::read_to_string(&path).unwrap();
                let once = format_source(&src, &FormatConfig::default())
                    .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let twice = format_source(&once, &FormatConfig::default()).unwrap();
                assert_eq!(once, twice, "{} is not stable", path.display());
                assert!(once.lines().all(|l| l.len() <= 88), "{}", path.display());
                seen += 1;
            }
        }
    }
    assert!(seen >= 8, "found only {seen} files");
}
