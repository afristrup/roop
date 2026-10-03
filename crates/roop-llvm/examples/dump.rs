use roop_llvm::{Options, compile_all};
use roop_syntax::parse;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let program = parse(&src).unwrap();
    roop_check::check(&program).unwrap();
    let out = compile_all(&program, &Options::default()).unwrap();
    println!(";; HOST\n{}", out.host);
    if let Some(air) = out.air {
        println!(";; AIR\n{air}");
    }
    if let Some(ptx) = out.ptx {
        println!(";; PTX\n{ptx}");
    }
}
