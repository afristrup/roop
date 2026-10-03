use crate::{Doc, fits, has_hard};

/// Lays `doc` out in lines of at most `width` columns, where it can.
pub fn render(doc: &Doc, width: usize, indent: usize) -> String {
    let mut out = String::new();
    let mut column = 0;
    let mut suffix = String::new();
    let mut stack: Vec<(usize, bool, &Doc)> = vec![(0, true, doc)];
    while let Some((level, breaking, d)) = stack.pop() {
        match d {
            Doc::Text(s) => {
                out.push_str(s);
                column += s.chars().count();
            }
            Doc::Concat(parts) => stack.extend(parts.iter().rev().map(|p| (level, breaking, p))),
            Doc::Nest(inner) => stack.push((level + indent, breaking, inner)),
            Doc::Group(inner) => {
                let breaks = breaking
                    && (has_hard(inner) || {
                        let rest = stack.iter().rev().map(|(_, b, d)| (*b, *d));
                        !fits(width as isize - column as isize, inner, rest)
                    });
                stack.push((level, breaks, inner));
            }
            Doc::Line if !breaking => {
                out.push(' ');
                column += 1;
            }
            Doc::SoftLine if !breaking => {}
            Doc::Line | Doc::SoftLine | Doc::HardLine => {
                out.push_str(&std::mem::take(&mut suffix));
                out.truncate(out.trim_end_matches(' ').len());
                out.push('\n');
                out.push_str(&" ".repeat(level));
                column = level;
            }
            Doc::IfBreak(s) if breaking => {
                out.push_str(s);
                column += s.len();
            }
            Doc::IfBreak(_) => {}
            Doc::Suffix(s) => suffix.push_str(s),
        }
    }
    out.push_str(&suffix);
    out
}
