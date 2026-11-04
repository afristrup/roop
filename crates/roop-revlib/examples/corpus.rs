use roop_revlib::{index, load, load_syrec, lookup, validate};
use std::time::{Duration, Instant};

const USAGE: &str = "usage: cargo run -p roop-revlib --example corpus -- <revlib-directory> [--report] [--benchmark]";

fn fastest<T>(
    runs: usize,
    mut run: impl FnMut() -> Result<T, roop_revlib::Error>,
) -> Result<(T, Duration), roop_revlib::Error> {
    let mut best = None;
    for _ in 0..runs {
        let started = Instant::now();
        let value = run()?;
        let elapsed = started.elapsed();
        if best.as_ref().is_none_or(|(_, current)| elapsed < *current) {
            best = Some((value, elapsed));
        }
    }
    Ok(best.expect("at least one benchmark run"))
}

fn main() -> Result<(), roop_revlib::Error> {
    let mut args = std::env::args_os().skip(1);
    let directory = args.next().expect(USAGE);
    let mut report = false;
    let mut benchmark = false;
    for argument in args {
        match argument.to_str() {
            Some("--report") => report = true,
            Some("--benchmark") => {
                benchmark = true;
                report = true;
            }
            _ => panic!("{USAGE}"),
        }
    }
    let directory = std::path::Path::new(&directory);
    let (corpus, parse_time) = fastest(1, || load(directory))?;
    let (syrec, syrec_time) = fastest(1, || load_syrec(directory))?;
    let runs = if benchmark { 3 } else { 1 };
    let mut rows = 0;
    let mut nodes = 0;
    let mut validation_time = Duration::ZERO;
    let mut index_time = Duration::ZERO;
    let mut lookup_time = Duration::ZERO;
    if report {
        println!("kind,file,inputs,outputs,rows,nodes,module,validate_ns,index_ns,lookup_ns");
    }
    for table in &corpus {
        let (_, validation) = fastest(runs, || validate(table))?;
        let (indexed, indexing) = fastest(runs, || index(table))?;
        nodes += indexed.node_count;
        let witnesses: Vec<_> = table
            .rows
            .iter()
            .map(|(cube, output)| (cube.replace('-', "0"), output.as_str()))
            .collect();
        let (_, lookups) = fastest(runs, || {
            let mut matches = 0;
            for (input, output) in &witnesses {
                matches += usize::from(lookup(&indexed, input)?.contains(output));
            }
            std::hint::black_box(matches);
            Ok(())
        })?;
        validation_time += validation;
        index_time += indexing;
        lookup_time += lookups;
        for (input, output) in &witnesses {
            rows += 1;
            assert!(lookup(&indexed, input)?.contains(output));
        }
        if report {
            println!(
                "pla,{},{},{},{},{},,{},{},{}",
                table.file,
                table.inputs,
                table.outputs,
                table.rows.len(),
                indexed.node_count,
                validation.as_nanos(),
                indexing.as_nanos(),
                lookups.as_nanos(),
            );
        }
    }
    if report {
        for source in &syrec {
            println!(
                "syrec,{file},,,,,{module},,,",
                file = source.file,
                module = source.module
            );
        }
    }
    println!(
        "validated {} PLA tables, {} rows, {} index nodes, and {} SyReC sources; parse {:.2?}, SyReC parse {:.2?}, validation {:.2?}, index {:.2?}, lookup {:.2?}",
        corpus.len(),
        rows,
        nodes,
        syrec.len(),
        parse_time,
        syrec_time,
        validation_time,
        index_time,
        lookup_time,
    );
    Ok(())
}
