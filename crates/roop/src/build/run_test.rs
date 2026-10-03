use crate::{
    BuildArgs, CliError, Emit, TestArgs, Verdict, build, config_for, lean_agreement,
    load_with_tests, run_one_test, run_trace, test_driver, test_files,
};
use roop_syntax::{FnDef, Item};
use std::time::Duration;

/// `roop test`: builds each file that declares tests with a driver and runs
/// every test in a process of its own, forward and then backward.
pub fn run_test(args: &TestArgs) -> Result<(), CliError> {
    let start = args.paths.first().cloned().unwrap_or_else(|| ".".into());
    let config = config_for(&start.join("x"))?;
    let (mut passed, mut failed) = (0, 0);
    for file in test_files(&args.paths, &config.root)? {
        let file_config = config_for(&file)?;
        let program = load_with_tests(&file, &file_config)?;
        let tests: Vec<&FnDef> = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Fn(f) if f.test => Some(f),
                _ => None,
            })
            .collect();
        let selected: Vec<(usize, &FnDef)> = tests
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, t)| args.filter.as_ref().is_none_or(|f| t.name.contains(f)))
            .collect();
        if selected.is_empty() {
            continue;
        }
        let dir = std::env::temp_dir().join(format!("roop-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).map_err(|e| CliError::Io("temp dir".into(), e))?;
        let driver = dir.join("driver.c");
        std::fs::write(&driver, test_driver(&tests)?)
            .map_err(|e| CliError::Io(driver.display().to_string(), e))?;
        let exe = dir.join("tests");
        let built = build(&BuildArgs {
            input: file.clone(),
            output: Some(exe.clone()),
            emit: Some(Emit::Executable),
            link: vec![driver],
            keep_tests: true,
        });
        if let Err(e) = built {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
        println!("running {} tests from {}", selected.len(), file.display());
        let mut native: Vec<(String, bool)> = Vec::new();
        for (index, test) in selected {
            let timeout = Duration::from_secs(args.timeout);
            let verdict = run_one_test(&exe, index, timeout)
                .map_err(|e| CliError::Io(format!("running {}", test.name), e))?;
            match verdict {
                Verdict::Passed(peak) => {
                    native.push((test.name.clone(), true));
                    passed += 1;
                    match peak {
                        Some(bytes) => {
                            println!("test {} ... ok (history peak {bytes} bytes)", test.name)
                        }
                        None => println!("test {} ... ok", test.name),
                    }
                }
                other => {
                    failed += 1;
                    native.push((test.name.clone(), false));
                    let traced = !matches!(other, Verdict::TimedOut);
                    let why = match other {
                        Verdict::NotRestored(what) => format!("after the backward run, {what}"),
                        Verdict::TimedOut => format!("no result after {} s", args.timeout),
                        _ => "an expectation or another check failed".to_string(),
                    };
                    println!("test {} ... FAILED: {why}", test.name);
                    if traced {
                        let src = std::fs::read_to_string(&file)
                            .map_err(|e| CliError::Io(file.display().to_string(), e))?;
                        let timeout = Duration::from_secs(args.timeout);
                        let report = run_trace(&file_config, &program, test, &src, &dir, timeout)?;
                        print!("{report}");
                    }
                }
            }
        }
        if args.lean {
            let agreement = lean_agreement(&program, &native, &dir)?;
            println!(
                "lean model: {} agree, {} differ, {} not modelled",
                agreement.agree,
                agreement.differ.len(),
                agreement.unmodelled.len()
            );
            for name in &agreement.differ {
                println!("  the model and the compiled test differ on {name}");
            }
            failed += agreement.differ.len();
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    println!("{passed} passed, {failed} failed");
    if failed > 0 {
        return Err(CliError::Tool(format!("{failed} tests failed")));
    }
    Ok(())
}
