import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def compose (f : Rev α β) (g : Rev β γ) : Rev α γ where
  run := g.run ∘ f.run
  undo := f.undo ∘ g.undo
  undo_run := by
    intro x
    simp [g.undo_run, f.undo_run]
  run_undo := by
    intro y
    simp [f.run_undo, g.run_undo]

end RoopLean.Rev
