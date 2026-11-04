use roop_revlib::{load, load_syrec, parse, validate};

#[test]
fn parses_and_validates_revlib_pla_files() {
    let table = parse("toffoli", ".i 3\n.o 3\n000 000\n111 011\n.e\n").unwrap();
    validate(&table).unwrap();
    let Ok(directory) = std::env::var("ROOP_REVLIB_DIR") else {
        return;
    };
    let corpus = load(directory.as_ref()).unwrap();
    assert_eq!(corpus.len(), 133);
    for table in &corpus {
        validate(table).unwrap();
    }
    let syrec = load_syrec(directory.as_ref()).unwrap();
    assert_eq!(syrec.len(), 6);
}
