use roop_revlib::{evaluate, index, load, load_syrec, lookup, parse, parse_syrec, validate};
use std::time::{Duration, Instant};

const USAGE: &str = "usage: cargo run -p roop-revlib --example corpus -- <revlib-directory> [--report] [--benchmark] [--full-evaluate]";

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
    let mut full_evaluate = false;
    for argument in args {
        match argument.to_str() {
            Some("--report") => report = true,
            Some("--benchmark") => {
                benchmark = true;
                report = true;
            }
            Some("--full-evaluate") => {
                benchmark = true;
                report = true;
                full_evaluate = true;
            }
            _ => panic!("{USAGE}"),
        }
    }
    let directory = std::path::Path::new(&directory);
    let (corpus, load_time) = fastest(1, || load(directory))?;
    let (syrec, syrec_load_time) = fastest(1, || load_syrec(directory))?;
    let runs = if benchmark { 3 } else { 1 };
    let mut rows = 0;
    let mut nodes = 0;
    let mut parse_time = Duration::ZERO;
    let mut validation_time = Duration::ZERO;
    let mut evaluate_time = Duration::ZERO;
    let mut full_evaluate_time = Duration::ZERO;
    let mut index_time = Duration::ZERO;
    let mut lookup_time = Duration::ZERO;
    let mut syrec_parse_time = Duration::ZERO;
    if report {
        println!(
            "kind,file,inputs,outputs,rows,nodes,module,parse_ns,validate_ns,evaluate_ns,index_ns,lookup_ns,evaluate_all_ns"
        );
    }
    for table in &corpus {
        let (parsed, parsing) = fastest(runs, || {
            let source = std::fs::read_to_string(&table.file)?;
            parse(&table.file, &source)
        })?;
        assert_eq!(&parsed, table);
        let (_, validation) = fastest(runs, || validate(table))?;
        let witnesses: Vec<_> = table
            .rows
            .iter()
            .map(|(cube, output)| (cube.replace('-', "0"), output.as_str()))
            .collect();
        let (input, output) = witnesses.first().expect("PLA has a row");
        let (_, evaluating) = fastest(runs, || {
            let matches = usize::from(evaluate(table, input)?.contains(output));
            std::hint::black_box(matches);
            Ok(())
        })?;
        evaluate_time += evaluating;
        let evaluating_all = if full_evaluate {
            let (_, elapsed) = fastest(runs, || {
                let mut matches = 0;
                for (input, output) in &witnesses {
                    matches += usize::from(evaluate(table, input)?.contains(output));
                }
                std::hint::black_box(matches);
                Ok(())
            })?;
            full_evaluate_time += elapsed;
            Some(elapsed)
        } else {
            None
        };
        let (indexed, indexing) = fastest(runs, || index(table))?;
        nodes += indexed.node_count;
        let (_, lookups) = fastest(runs, || {
            let mut matches = 0;
            for (input, output) in &witnesses {
                matches += usize::from(lookup(&indexed, input)?.contains(output));
            }
            std::hint::black_box(matches);
            Ok(())
        })?;
        parse_time += parsing;
        validation_time += validation;
        index_time += indexing;
        lookup_time += lookups;
        for (input, output) in &witnesses {
            rows += 1;
            assert!(lookup(&indexed, input)?.contains(output));
        }
        if report {
            println!(
                "pla,{},{},{},{},{},,{},{},{},{},{},{}",
                table.file,
                table.inputs,
                table.outputs,
                table.rows.len(),
                indexed.node_count,
                parsing.as_nanos(),
                validation.as_nanos(),
                evaluating.as_nanos(),
                indexing.as_nanos(),
                lookups.as_nanos(),
                evaluating_all.map_or_else(String::new, |elapsed| elapsed.as_nanos().to_string()),
            );
        }
    }
    for source in &syrec {
        let (parsed, parsing) = fastest(runs, || {
            let text = std::fs::read_to_string(&source.file)?;
            parse_syrec(&source.file, &text)
        })?;
        assert_eq!(&parsed, source);
        syrec_parse_time += parsing;
        if report {
            println!(
                "syrec,{file},,,,,{module},{parse},,,,,",
                file = source.file,
                module = source.module,
                parse = parsing.as_nanos(),
            );
        }
    }
    println!(
        "validated {} PLA tables, {} rows, {} index nodes, and {} SyReC sources; load {:.2?}, SyReC load {:.2?}, per-source parse {:.2?}, SyReC parse {:.2?}, validation {:.2?}, evaluate {:.2?}, index {:.2?}, lookup {:.2?}, full evaluate {:.2?}",
        corpus.len(),
        rows,
        nodes,
        syrec.len(),
        load_time,
        syrec_load_time,
        parse_time,
        syrec_parse_time,
        validation_time,
        evaluate_time,
        index_time,
        lookup_time,
        full_evaluate_time,
    );
    Ok(())
}
