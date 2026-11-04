module

import Mathlib.Algebra.Group.Defs
import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def addUpdate [AddGroup β] (f : α → β) : Rev (α × β) (α × β) where
  run := fun (x, y) => (x, y + f x)
  undo := fun (x, y) => (x, y - f x)
  undo_run := by rintro ⟨x, y⟩; simp
  run_undo := by rintro ⟨x, y⟩; simp

end RoopLean.Rev
