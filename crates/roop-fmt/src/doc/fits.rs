use crate::{Doc, has_hard};

/// Whether `doc`, laid out flat, and what follows it up to the next place a
/// line can break, stays within `budget` columns. `rest` is the pending work
/// as (breaking, doc) pairs, the next to run first.
pub fn fits<'a>(
    mut budget: isize,
    doc: &'a Doc,
    mut rest: impl Iterator<Item = (bool, &'a Doc)>,
) -> bool {
    let mut work: Vec<(bool, &Doc)> = vec![(false, doc)];
    loop {
        let (breaking, d) = match work.pop().or_else(|| rest.next()) {
            Some(item) => item,
            None => return true,
        };
        match d {
            Doc::Text(s) => budget -= s.chars().count() as isize,
            Doc::Concat(parts) => work.extend(parts.iter().rev().map(|p| (breaking, p))),
            Doc::Nest(inner) => work.push((breaking, inner)),
            Doc::Group(inner) => work.push((has_hard(inner), inner)),
            Doc::Line if breaking => return true,
            Doc::Line => budget -= 1,
            Doc::SoftLine if breaking => return true,
            Doc::SoftLine => {}
            Doc::HardLine => return true,
            Doc::IfBreak(s) if breaking => budget -= s.len() as isize,
            Doc::IfBreak(_) | Doc::Suffix(_) => {}
        }
        if budget < 0 {
            return false;
        }
    }
}
