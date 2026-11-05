import RoopLean.Certificate.ProgramInverseExec
import RoopLean.Execution

namespace RoopLean

theorem Program.inverse_execution {α : Type} (program : Program α)
    {s t : α} {labels : List Unit} {states : List α}
    (run : (forwards program.exec).Execution s labels t states) :
    (forwards program.undo).Execution t labels.reverse s states.reverse := by
  exact RoopLean.inverse_execution _ _ (program.certificate).undo_run (program.certificate).run_undo run

end RoopLean
