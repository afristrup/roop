import RoopLean.Certificate.ProgramInverseExec
import RoopLean.Execution

namespace RoopLean

theorem Program.inverse_execution (program : Program State)
    (run : (forwards program.exec).Execution s labels t states) :
    (forwards program.undo).Execution t labels.reverse s states.reverse := by
  exact RoopLean.inverse_execution _ _ (program.certificate).undo_run (program.certificate).run_undo run

end RoopLean
