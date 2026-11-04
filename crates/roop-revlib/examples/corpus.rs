use roop_revlib::{index, load, load_syrec, lookup, validate};

fn main() -> Result<(), roop_revlib::Error> {
    let mut args = std::env::args_os().skip(1);
    let directory = args
        .next()
        .expect("usage: cargo run -p roop-revlib --example corpus -- <revlib-directory>");
    let report = args.any(|argument| argument == "--report");
    let started = std::time::Instant::now();
    let corpus = load(std::path::Path::new(&directory))?;
    let mut rows = 0;
    let mut nodes = 0;
    if report {
        println!("kind,file,inputs,outputs,rows,nodes,module");
    }
    for table in &corpus {
        validate(table)?;
        let indexed = index(table)?;
        nodes += indexed.node_count;
        for (cube, output) in &table.rows {
            rows += 1;
            let input = cube.replace('-', "0");
            assert!(lookup(&indexed, &input)?.contains(&output.as_str()));
        }
        if report {
            println!(
                "pla,{},{},{},{},{},",
                table.file,
                table.inputs,
                table.outputs,
                table.rows.len(),
                indexed.node_count
            );
        }
    }
    let syrec = load_syrec(std::path::Path::new(&directory))?;
    if report {
        for source in &syrec {
            println!("syrec,{},,,,,{}", source.file, source.module);
        }
    }
    println!(
        "validated {} PLA tables, {} rows, {} index nodes, and {} SyReC sources in {:.2?}",
        corpus.len(),
        rows,
        nodes,
        syrec.len(),
        started.elapsed()
    );
    Ok(())
}
