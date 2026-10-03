use roop_syntax::{Comment, Span};
use std::cell::Cell;

/// The source text and its comments. The printer walks the program in source
/// order and takes each comment as it reaches it.
pub struct Ctx<'a> {
    src: &'a str,
    comments: Vec<Comment>,
    next: Cell<usize>,
    stray: Cell<Option<usize>>,
}

impl<'a> Ctx<'a> {
    pub fn new(src: &'a str, comments: Vec<Comment>) -> Self {
        Ctx {
            src,
            comments,
            next: Cell::new(0),
            stray: Cell::new(None),
        }
    }

    /// How many comments have been taken so far.
    pub fn taken(&self) -> usize {
        self.next.get()
    }

    fn peek(&self) -> Option<&Comment> {
        self.comments.get(self.next.get())
    }

    fn take(&self) -> Comment {
        let comment = self.comments[self.next.get()].clone();
        self.next.set(self.next.get() + 1);
        comment
    }

    /// Takes every comment that starts before `pos`.
    pub fn take_before(&self, pos: usize) -> Vec<Comment> {
        let mut taken = Vec::new();
        while self.peek().is_some_and(|c| c.span.start < pos) {
            taken.push(self.take());
        }
        taken
    }

    /// Takes the comment that follows `end` on the same line with only blanks between.
    pub fn take_trailing(&self, end: usize) -> Option<Comment> {
        let c = self.peek()?;
        let between = self.src.get(end..c.span.start)?;
        (between.trim().is_empty() && !between.contains('\n')).then(|| self.take())
    }

    /// Whether `span` holds a comment that has not been taken.
    pub fn has_comment_in(&self, span: Span) -> bool {
        self.peek()
            .is_some_and(|c| c.span.start >= span.start && c.span.start < span.end)
    }

    /// Whether a blank line separates the two positions.
    pub fn blank_between(&self, from: usize, to: usize) -> bool {
        from < to && self.src[from..to].matches('\n').count() >= 2
    }

    /// Notes a comment that sits inside code that the printer lays out
    /// itself, where it would have to move.
    pub fn check_inside(&self, end: usize) {
        if let Some(c) = self.peek().filter(|c| c.span.start < end) {
            self.stray
                .get()
                .is_none()
                .then(|| self.stray.set(Some(c.span.start)));
        }
    }

    /// The line of the first comment that could not be kept in place.
    pub fn stray_line(&self) -> Option<usize> {
        let at = self
            .stray
            .get()
            .or_else(|| self.peek().map(|c| c.span.start))?;
        Some(self.src[..at].matches('\n').count() + 1)
    }
}
