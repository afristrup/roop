use roop_revlib::{index, load, load_syrec, lookup, validate};

fn main() -> Result<(), roop_revlib::Error> {
    let directory = std::env::args_os()
        .nth(1)
        .expect("usage: cargo run -p roop-revlib --example corpus -- <revlib-directory>");
    let started = std::time::Instant::now();
    let corpus = load(std::path::Path::new(&directory))?;
    let mut rows = 0;
    for table in &corpus {
        validate(table)?;
        let indexed = index(table)?;
        for (cube, output) in &table.rows {
            rows += 1;
            let input = cube.replace('-', "0");
            assert!(lookup(&indexed, &input)?.contains(&output.as_str()));
        }
    }
    let syrec = load_syrec(std::path::Path::new(&directory))?;
    println!(
        "validated {} PLA tables, {} rows, and {} SyReC sources in {:.2?}",
        corpus.len(),
        rows,
        syrec.len(),
        started.elapsed()
    );
    Ok(())
}
