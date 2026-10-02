use roop_syntax::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum CheckError {
    SelfReferentialUpdate {
        var: String,
        span: Span,
    },
    AncillaNotRestored {
        name: String,
        span: Span,
    },
    AncillaTouchedInControlFlow {
        name: String,
        span: Span,
    },
    NonExhaustiveMatch {
        span: Span,
    },
    UnknownVariant {
        enum_name: String,
        variant: String,
        span: Span,
    },
    DuplicateVariant {
        enum_name: String,
        variant: String,
        span: Span,
    },
    BorrowedPlaceUsed {
        var: String,
        span: Span,
    },
    BorrowIndexModified {
        var: String,
        span: Span,
    },
    ParallelNotLoop {
        span: Span,
    },
    ParallelLoopShape {
        span: Span,
    },
    ParallelBoundModified {
        var: String,
        span: Span,
    },
    ParallelInductionWritten {
        var: String,
        span: Span,
    },
    ParallelWriteNotDisjoint {
        var: String,
        span: Span,
    },
    ParallelCrossIteration {
        var: String,
        span: Span,
    },
    ConcurrentNotBlock {
        span: Span,
    },
    ConcurrentConflict {
        var: String,
        span: Span,
    },
    ChannelOpOutsideTask {
        chan: String,
        span: Span,
    },
    UnknownChannel {
        chan: String,
        span: Span,
    },
    ChannelOpInControlFlow {
        chan: String,
        span: Span,
    },
    ChannelNotPointToPoint {
        chan: String,
        span: Span,
    },
    ChannelDeadlock {
        chan: String,
        span: Span,
    },
    ChannelNotDrained {
        chan: String,
        span: Span,
    },
    IrreversibleOutsideIrrev {
        span: Span,
    },
    CallsIrreversible {
        callee: String,
        span: Span,
    },
    UncallIrreversible {
        callee: String,
        span: Span,
    },
    TryCannotFail {
        span: Span,
    },
    NotUnwindable {
        what: &'static str,
        span: Span,
    },
    TryOutcomeWritten {
        var: String,
        span: Span,
    },
    UnpairedBuild {
        name: String,
        span: Span,
    },
    UnbuildNotInverse {
        name: String,
        span: Span,
    },
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SelfReferentialUpdate { var, span } => write!(
                f,
                "state-destructive update: `{var}` appears on its own right-hand side at {}..{}",
                span.start, span.end
            ),
            Self::AncillaNotRestored { name, span } => write!(
                f,
                "ancilla `{name}` is not provably restored at block exit at {}..{}",
                span.start, span.end
            ),
            Self::AncillaTouchedInControlFlow { name, span } => write!(
                f,
                "ancilla `{name}` is modified inside control flow at {}..{}",
                span.start, span.end
            ),
            Self::BorrowedPlaceUsed { var, span } => write!(
                f,
                "`{var}` is used while borrowed at {}..{}",
                span.start, span.end
            ),
            Self::BorrowIndexModified { var, span } => write!(
                f,
                "`{var}` selects a borrowed place and is modified in the borrow at {}..{}",
                span.start, span.end
            ),
            Self::ParallelNotLoop { span } => write!(
                f,
                "#[parallel] applies only to from loops at {}..{}",
                span.start, span.end
            ),
            Self::ParallelLoopShape { span } => write!(
                f,
                "#[parallel] needs `from v == lo {{..}} loop {{ v += k; }} until v == hi` with k > 0 at {}..{}",
                span.start, span.end
            ),
            Self::ParallelBoundModified { var, span } => write!(
                f,
                "loop bound `{var}` is modified by the parallel body at {}..{}",
                span.start, span.end
            ),
            Self::ParallelInductionWritten { var, span } => write!(
                f,
                "parallel body writes its induction variable `{var}` at {}..{}",
                span.start, span.end
            ),
            Self::ParallelWriteNotDisjoint { var, span } => write!(
                f,
                "write to `{var}` is not indexed by the loop variable, so iterations collide at {}..{}",
                span.start, span.end
            ),
            Self::ParallelCrossIteration { var, span } => write!(
                f,
                "an access to `{var}` overlaps another iteration's write at {}..{}",
                span.start, span.end
            ),
            Self::ConcurrentNotBlock { span } => write!(
                f,
                "#[concurrent] applies only to block statements at {}..{}",
                span.start, span.end
            ),
            Self::ConcurrentConflict { var, span } => write!(
                f,
                "concurrent tasks share `{var}`, and one of them writes it, at {}..{}",
                span.start, span.end
            ),
            Self::ChannelOpOutsideTask { chan, span } => write!(
                f,
                "channel `{chan}` is used outside a #[concurrent] task at {}..{}",
                span.start, span.end
            ),
            Self::UnknownChannel { chan, span } => write!(
                f,
                "channel `{chan}` is not declared by an enclosing chan at {}..{}",
                span.start, span.end
            ),
            Self::ChannelOpInControlFlow { chan, span } => write!(
                f,
                "channel `{chan}` is used under if, match, loop or try inside a task, so its protocol is not static, at {}..{}",
                span.start, span.end
            ),
            Self::ChannelNotPointToPoint { chan, span } => write!(
                f,
                "channel `{chan}` must have one sending task and one receiving task at {}..{}",
                span.start, span.end
            ),
            Self::ChannelDeadlock { chan, span } => write!(
                f,
                "tasks deadlock waiting on channel `{chan}` at {}..{}",
                span.start, span.end
            ),
            Self::ChannelNotDrained { chan, span } => write!(
                f,
                "channel `{chan}` still holds messages when its tasks finish at {}..{}",
                span.start, span.end
            ),
            Self::IrreversibleOutsideIrrev { span } => write!(
                f,
                "irreversible code (overwrite, try, irrev block) needs an irrev fn or irrev block at {}..{}",
                span.start, span.end
            ),
            Self::CallsIrreversible { callee, span } => write!(
                f,
                "`{callee}` is irreversible, so calling it needs irrev code at {}..{}",
                span.start, span.end
            ),
            Self::UncallIrreversible { callee, span } => write!(
                f,
                "`{callee}` is irreversible and has no inverse to uncall at {}..{}",
                span.start, span.end
            ),
            Self::TryCannotFail { span } => write!(
                f,
                "try body has no exit assertion or call that could roll back at {}..{}",
                span.start, span.end
            ),
            Self::NotUnwindable { what, span } => write!(
                f,
                "a try with an outcome cannot contain {what}, since a failure could not be undone, at {}..{}",
                span.start, span.end
            ),
            Self::TryOutcomeWritten { var, span } => write!(
                f,
                "the outcome `{var}` is written by the try body or handler at {}..{}",
                span.start, span.end
            ),
            Self::UnpairedBuild { name, span } => write!(
                f,
                "struct `{name}` must define both build and unbuild at {}..{}",
                span.start, span.end
            ),
            Self::UnbuildNotInverse { name, span } => write!(
                f,
                "unbuild of `{name}` is not provably the inverse of build at {}..{}",
                span.start, span.end
            ),
            Self::UnknownVariant {
                enum_name,
                variant,
                span,
            } => write!(
                f,
                "unknown variant `{enum_name}::{variant}` at {}..{}",
                span.start, span.end
            ),
            Self::DuplicateVariant {
                enum_name,
                variant,
                span,
            } => write!(
                f,
                "enum `{enum_name}` repeats variant `{variant}` at {}..{}",
                span.start, span.end
            ),
            Self::NonExhaustiveMatch { span } => {
                write!(f, "match is not exhaustive at {}..{}", span.start, span.end)
            }
        }
    }
}

impl std::error::Error for CheckError {}
