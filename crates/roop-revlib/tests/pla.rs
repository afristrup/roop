use roop_revlib::{evaluate, index, load, load_syrec, lookup, parse, validate};

#[test]
fn parses_and_validates_revlib_pla_files() {
    let table = parse("toffoli", ".i 3\n.o 3\n000 000\n111 011\n.e\n").unwrap();
    validate(&table).unwrap();
    assert_eq!(evaluate(&table, "111").unwrap(), vec!["011"]);
    assert!(evaluate(&table, "010").unwrap().is_empty());
    let indexed = index(&table).unwrap();
    assert_eq!(lookup(&indexed, "111").unwrap(), vec!["011"]);
    assert_eq!(indexed.node_count, 7);
    let Ok(directory) = std::env::var("ROOP_REVLIB_DIR") else {
        return;
    };
    let corpus = load(directory.as_ref()).unwrap();
    assert_eq!(corpus.len(), 133);
    for table in &corpus {
        validate(table).unwrap();
        let indexed = index(table).unwrap();
        for (cube, output) in &table.rows {
            let input = cube.replace('-', "0");
            assert!(lookup(&indexed, &input).unwrap().contains(&output.as_str()));
        }
    }
    let syrec = load_syrec(directory.as_ref()).unwrap();
    assert_eq!(syrec.len(), 6);
}
