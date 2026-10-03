use crate::{Ctx, Doc};
use roop_syntax::Comment;

/// The lines of a block or of a file, with blank lines kept where the source
/// had them.
pub struct Lines {
    docs: Vec<Doc>,
    prev_end: usize,
}

impl Lines {
    pub fn new(start: usize) -> Self {
        Lines {
            docs: Vec::new(),
            prev_end: start,
        }
    }

    /// Adds a blank line before the next entry at `pos` if the source had
    /// one there, or if `force` asks for it. Never at the start.
    pub fn gap(&mut self, ctx: &Ctx, pos: usize, force: bool) {
        if !self.docs.is_empty() && (force || ctx.blank_between(self.prev_end, pos)) {
            self.docs.push(Doc::nothing());
        }
    }

    pub fn comment(&mut self, ctx: &Ctx, comment: &Comment, force: bool) {
        self.gap(ctx, comment.span.start, force);
        self.docs.push(Doc::text(&comment.text));
        self.prev_end = comment.span.end;
    }

    /// Adds `doc`, which ends at `end`, with the comment that follows it on
    /// its line.
    pub fn entry(&mut self, ctx: &Ctx, doc: Doc, end: usize) {
        match ctx.take_trailing(end) {
            Some(c) => {
                let suffix = Doc::Suffix(format!(" {}", c.text));
                self.docs.push(Doc::concat(vec![doc, suffix]));
                self.prev_end = c.span.end;
            }
            None => {
                self.docs.push(doc);
                self.prev_end = end;
            }
        }
    }

    /// Puts `doc` before everything else.
    pub fn first(&mut self, doc: Doc) {
        self.docs.insert(0, doc);
    }

    pub fn into_docs(self) -> Vec<Doc> {
        self.docs
    }
}
