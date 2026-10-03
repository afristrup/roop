use crate::{CliError, Fixture, fixture_of};
use roop_syntax::FnDef;

fn show_fixture(j: usize, f: &Fixture) -> String {
    let (cast, format) = match f.kind {
        'f' => ("((const double*)", "%g"),
        'b' | 'u' => ("((const unsigned char*)", "%d"),
        _ => ("((const long long*)", "%lld"),
    };
    let open = if f.count > 1 { "[" } else { "" };
    let close = if f.count > 1 { "]" } else { "" };
    format!(
        "    printf(\" {name}={open}\");\n    \
         for (int k = 0; k < {count}; k++) printf(\"%s{format}\", k ? \",\" : \"\", {cast}f{j})[k]);\n    \
         printf(\"{close}\");\n",
        name = f.name,
        count = f.count,
    )
}

/// The C `main` for a test split into stages: it prints the state before every
/// statement, forward and then backward, flushing as it goes, so what was
/// printed last says where a failing run stopped.
pub fn trace_driver(test: &FnDef, stages: usize) -> Result<String, CliError> {
    let fixtures: Vec<Fixture> = test
        .params
        .iter()
        .map(|p| fixture_of(&test.name, p))
        .collect::<Result<_, _>>()?;
    let pointers = vec!["void*"; fixtures.len()].join(", ");
    let pointers = if pointers.is_empty() {
        "void".into()
    } else {
        pointers
    };
    let mut out = String::from("#include <stdio.h>\n#include <stdint.h>\n");
    for i in 0..stages {
        let stage = roop_opt::stage_name(&test.name, i);
        out.push_str(&format!(
            "void {stage}({pointers});\nvoid {stage}_inv({pointers});\n"
        ));
    }
    let params: Vec<String> = (0..fixtures.len())
        .map(|j| format!("const void *f{j}"))
        .collect();
    let params = if params.is_empty() {
        "void".into()
    } else {
        params.join(", ")
    };
    out.push_str(&format!(
        "static void show(char dir, int idx, {params}) {{\n    printf(\"@ %c %d\", dir, idx);\n"
    ));
    for (j, f) in fixtures.iter().enumerate() {
        out.push_str(&show_fixture(j, f));
    }
    out.push_str("    printf(\"\\n\");\n    fflush(stdout);\n}\n");
    out.push_str("int main(void) {\n");
    for (j, f) in fixtures.iter().enumerate() {
        let words = (f.count * if matches!(f.kind, 'b' | 'u') { 1 } else { 8 }).div_ceil(8);
        out.push_str(&format!("    int64_t f{j}[{words}] = {{0}};\n"));
    }
    let args: Vec<String> = (0..fixtures.len()).map(|j| format!("f{j}")).collect();
    let args = args.join(", ");
    let sep = if args.is_empty() { "" } else { ", " };
    for i in 0..stages {
        out.push_str(&format!("    show('F', {i}{sep}{args});\n"));
        out.push_str(&format!(
            "    {}({args});\n",
            roop_opt::stage_name(&test.name, i)
        ));
    }
    out.push_str(&format!("    show('F', -1{sep}{args});\n"));
    for i in (0..stages).rev() {
        out.push_str(&format!("    show('B', {i}{sep}{args});\n"));
        out.push_str(&format!(
            "    {}_inv({args});\n",
            roop_opt::stage_name(&test.name, i)
        ));
    }
    out.push_str(&format!(
        "    show('B', -1{sep}{args});\n    return 0;\n}}\n"
    ));
    Ok(out)
}
