use crate::{CliError, Fixture, fixture_of};
use roop_syntax::FnDef;

fn bytes(f: &Fixture) -> usize {
    f.count * if matches!(f.kind, 'b' | 'u') { 1 } else { 8 }
}

/// The C `main` that runs one test, chosen by index: it makes the fixtures
/// zero, runs the test forward, then backward, and checks that every fixture
/// is zero again and that no output is still pending. Exit 0 passes, 3 means the
/// backward run did not restore.
pub fn test_driver(tests: &[&FnDef]) -> Result<String, CliError> {
    let mut out = String::from(
        "#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n\
         void roop_pending(int64_t *len);\n\
         static int zero(const void *p, size_t n) {\n    const unsigned char *b = p;\n    \
         for (size_t i = 0; i < n; i++) if (b[i]) return 0;\n    return 1;\n}\n",
    );
    let mut cases = String::new();
    for (k, test) in tests.iter().enumerate() {
        let fixtures: Vec<Fixture> = test
            .params
            .iter()
            .map(|p| fixture_of(&test.name, p))
            .collect::<Result<_, _>>()?;
        let pointers = vec!["void*"; fixtures.len()].join(", ");
        let pointers = if pointers.is_empty() {
            "void".to_string()
        } else {
            pointers
        };
        out.push_str(&format!(
            "void {n}({pointers});\nvoid {n}_inv({pointers});\n",
            n = test.name
        ));
        let args: Vec<String> = (0..fixtures.len()).map(|j| format!("f{j}")).collect();
        let mut body = String::new();
        for (j, f) in fixtures.iter().enumerate() {
            let words = bytes(f).div_ceil(8);
            body.push_str(&format!("        int64_t f{j}[{words}] = {{0}};\n"));
        }
        body.push_str(&format!("        {}({});\n", test.name, args.join(", ")));
        body.push_str(&format!(
            "        {}_inv({});\n",
            test.name,
            args.join(", ")
        ));
        for (j, f) in fixtures.iter().enumerate() {
            body.push_str(&format!(
                "        if (!zero(f{j}, {})) {{ printf(\"fixture {} is not zero\\n\"); return 3; }}\n",
                bytes(f),
                f.name
            ));
        }
        body.push_str(
            "        { int64_t pending = 0; roop_pending(&pending); \
             if (pending) { printf(\"output is still pending\\n\"); return 3; } }\n",
        );
        cases.push_str(&format!(
            "    case {k}: {{\n{body}        return 0;\n    }}\n"
        ));
    }
    out.push_str(&format!(
        "int main(int argc, char **argv) {{\n    if (argc < 2) return 2;\n    switch (atoi(argv[1])) {{\n{cases}    }}\n    return 2;\n}}\n"
    ));
    Ok(out)
}
