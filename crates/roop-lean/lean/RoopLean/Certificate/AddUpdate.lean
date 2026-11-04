import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def addUpdate (f : α → BitVec n) : Rev (α × BitVec n) (α × BitVec n) where
  run := fun (x, y) => (x, y + f x)
  undo := fun (x, y) => (x, y - f x)
  undo_run := by rintro ⟨x, y⟩; simp [BitVec.add_sub_cancel]
  run_undo := by rintro ⟨x, y⟩; simp [BitVec.sub_add_cancel]

end RoopLean.Rev
