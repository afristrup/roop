use roop_lean::translate;
use roop_syntax::parse;

fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let program = parse(&src).unwrap();
    roop_check::check(&program).unwrap();
    let t = translate(&program);
    print!("{}", t.lean);
    eprintln!("reversible: {:?}\nforward_only: {:?}\nopen: {:?}\nskipped: {:?}", t.reversible, t.forward_only, t.open, t.skipped);
}
