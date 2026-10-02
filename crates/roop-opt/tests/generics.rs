use roop_check::check;
use roop_opt::{GenericError, monomorphize};
use roop_syntax::{Item, parse};

const AXPY: &str = "
    fn axpy<N>(a: &mut [i64; N], b: &[i64; N], i: &mut i64, k: &i64) {
        #[parallel]
        from i == 0 { a[i] += b[i] * k; } loop { i += 1; } until i == N - 1;
    }
";

fn mono(src: &str) -> Result<roop_syntax::Program, GenericError> {
    monomorphize(&parse(src).unwrap())
}

fn names(program: &roop_syntax::Program) -> Vec<String> {
    program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f.name.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn instantiates_a_generic_function_per_length_and_the_result_checks() {
    let src = format!(
        "{AXPY}
         fn main(a: &mut [i64; 8], b: &[i64; 8], c: &mut [i64; 3], d: &[i64; 3],
                 i: &mut i64, j: &mut i64, k: &i64) {{
             call axpy<8>(a, b, i, k);
             call axpy<3>(c, d, j, k);
             call axpy<8>(a, b, i, k);
         }}"
    );
    let program = mono(&src).unwrap();
    let mut got = names(&program);
    got.sort();
    assert_eq!(got, ["axpy__3", "axpy__8", "main"]);
    check(&program).unwrap();
}

#[test]
fn lengths_pass_through_generic_callers() {
    let program = mono(&format!(
        "{AXPY}
         fn twice<M>(a: &mut [i64; M], b: &[i64; M], i: &mut i64, k: &i64) {{
             call axpy<M>(a, b, i, k);
         }}
         fn main(a: &mut [i64; 4], b: &[i64; 4], i: &mut i64, k: &i64) {{
             call twice<4>(a, b, i, k);
         }}"
    ))
    .unwrap();
    let mut got = names(&program);
    got.sort();
    assert_eq!(got, ["axpy__4", "main", "twice__4"]);
    check(&program).unwrap();
}

#[test]
fn reports_arity_and_constness_errors() {
    let main = |call: &str| {
        format!("{AXPY} fn main(a: &mut [i64; 4], b: &[i64; 4], i: &mut i64, k: &i64) {{ {call} }}")
    };
    assert!(matches!(
        mono(&main("call axpy<4, 4>(a, b, i, k);")),
        Err(GenericError::Arity { .. })
    ));
    assert!(matches!(
        mono(&main("call axpy(a, b, i, k);")),
        Err(GenericError::Arity { .. })
    ));
    assert!(matches!(
        mono(&main("call axpy<k>(a, b, i, k);")),
        Err(GenericError::NotConstant(_))
    ));
    assert!(matches!(
        mono("fn f(x: &mut i64) { x += 1; } fn main(x: &mut i64) { call f<3>(x); }"),
        Err(GenericError::NotGeneric(_))
    ));
}

#[test]
fn an_unbound_length_is_an_error() {
    assert!(matches!(
        mono("fn f(a: &mut [i64; N]) { } fn main(a: &mut [i64; 2]) { call f(a); }"),
        Err(GenericError::Unbound(_))
    ));
}

#[test]
fn generic_stacks_instantiate_too() {
    let program = mono(
        "fn log<C>(h: &mut Stack<i64, C>, x: &mut i64) { push h <- x; }
         fn main(h: &mut Stack<i64, 4>, x: &mut i64) { call log<4>(h, x); }",
    )
    .unwrap();
    check(&program).unwrap();
}
