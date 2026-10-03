use roop_syntax::FnDef;

/// The statement as a line of text, for a trace.
fn statement_text(src: &str, test: &FnDef, index: usize) -> String {
    let span = test.body.stmts[index].span;
    let text: Vec<&str> = src[span.start..span.end].split_whitespace().collect();
    let text = text.join(" ");
    match text.char_indices().nth(46) {
        Some((cut, _)) => format!("{}...", &text[..cut]),
        None => text,
    }
}

fn line_of(src: &str, test: &FnDef, index: usize) -> usize {
    src[..test.body.stmts[index].span.start]
        .matches('\n')
        .count()
        + 1
}

/// The lines of a trace: the state before each statement the driver reached,
/// with the statement, going forward and then backward. `output` is what the
/// driver printed; `failed` says it stopped before it finished.
pub fn trace_report(src: &str, test: &FnDef, output: &str, failed: bool) -> String {
    let rows: Vec<(char, i64, &str)> = output
        .lines()
        .filter_map(|line| {
            let mut parts = line.strip_prefix("@ ")?.splitn(3, ' ');
            let dir = parts.next()?.chars().next()?;
            let idx = parts.next()?.parse().ok()?;
            Some((dir, idx, parts.next().unwrap_or("").trim()))
        })
        .collect();
    let skip = rows.len().saturating_sub(8);
    let mut out = String::from("    state before each statement:\n");
    for (n, (dir, idx, state)) in rows.iter().enumerate().skip(skip) {
        let what = if *idx < 0 {
            if *dir == 'F' {
                "(forward run done)".to_string()
            } else {
                "(backward run done)".to_string()
            }
        } else {
            let text = statement_text(src, test, *idx as usize);
            let line = line_of(src, test, *idx as usize);
            match dir {
                'F' => format!("line {line}: {text}"),
                _ => format!("undo line {line}: {text}"),
            }
        };
        let mark = if failed && n + 1 == rows.len() {
            "   <- stopped here"
        } else {
            ""
        };
        out.push_str(&format!("      {what:<62} {state}{mark}\n"));
    }
    out
}
