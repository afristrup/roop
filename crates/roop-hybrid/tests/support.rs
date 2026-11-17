use roop_hybrid::{HybridProgram, Limits, Outcome, compile, execute};

pub fn program(name: &str, optimized: bool) -> HybridProgram {
    let path = format!("{}/programs/{name}.hyq", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).expect("example program");
    compile(&text, optimized).expect("example compiles")
}

pub fn run(name: &str, optimized: bool) -> Outcome {
    execute(&program(name, optimized), &Limits::default()).expect("example runs")
}

pub fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}
