use crate::CheckError;
use roop_syntax::Span;

impl CheckError {
    /// Where in the source the error is.
    pub fn span(&self) -> Span {
        match self {
            Self::SelfReferentialUpdate { span, .. }
            | Self::AncillaNotRestored { span, .. }
            | Self::AncillaTouchedInControlFlow { span, .. }
            | Self::NonExhaustiveMatch { span, .. }
            | Self::UnknownVariant { span, .. }
            | Self::DuplicateVariant { span, .. }
            | Self::BorrowedPlaceUsed { span, .. }
            | Self::BorrowIndexModified { span, .. }
            | Self::ParallelNotLoop { span, .. }
            | Self::CallAliasing { span, .. }
            | Self::UncallAfterKeep { span, .. }
            | Self::WorldInParallel { span, .. }
            | Self::ParallelLoopShape { span, .. }
            | Self::ParallelBoundModified { span, .. }
            | Self::ParallelInductionWritten { span, .. }
            | Self::ParallelWriteNotDisjoint { span, .. }
            | Self::ParallelCrossIteration { span, .. }
            | Self::ConcurrentNotBlock { span, .. }
            | Self::ConcurrentConflict { span, .. }
            | Self::ChannelOpOutsideTask { span, .. }
            | Self::UnknownChannel { span, .. }
            | Self::ChannelOpInControlFlow { span, .. }
            | Self::ChannelNotPointToPoint { span, .. }
            | Self::ChannelDeadlock { span, .. }
            | Self::ChannelNotDrained { span, .. }
            | Self::IrreversibleOutsideIrrev { span, .. }
            | Self::CallsIrreversible { span, .. }
            | Self::UncallIrreversible { span, .. }
            | Self::TryCannotFail { span, .. }
            | Self::NotUnwindable { span, .. }
            | Self::TryOutcomeWritten { span, .. }
            | Self::UnpairedBuild { span, .. }
            | Self::UnbuildNotInverse { span, .. }
            | Self::SessionMalformed { span, .. }
            | Self::SessionNotCompliant { span, .. } => *span,
        }
    }
}
