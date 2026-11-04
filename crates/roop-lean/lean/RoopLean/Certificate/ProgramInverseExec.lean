import RoopLean.Certificate.ProgramExec
import RoopLean.Certificate.ProgramInverse

namespace RoopLean

def Program.certificate (program : Program α) : Rev α α where
  run := program.exec
  undo := program.undo
  undo_run := by
    intro x
    induction program generalizing x with
    | done => rfl
    | step head tail ih =>
      simp [Program.exec, Program.undo, ih, head.undo_run]
  run_undo := by
    intro x
    induction program generalizing x with
    | done => rfl
    | step head tail ih =>
      simp [Program.exec, Program.undo, ih, head.run_undo]

end RoopLean
