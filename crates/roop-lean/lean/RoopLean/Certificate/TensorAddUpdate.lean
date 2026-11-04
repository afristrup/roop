import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def tensorAddUpdate (f : α → Fin d → BitVec n) :
    Rev (α × (Fin d → BitVec n)) (α × (Fin d → BitVec n)) where
  run := fun (x, y) => (x, fun i => y i + f x i)
  undo := fun (x, y) => (x, fun i => y i - f x i)
  undo_run := by
    rintro ⟨x, y⟩
    simp only [Prod.mk.injEq, true_and]
    funext i
    simp [BitVec.add_sub_cancel]
  run_undo := by
    rintro ⟨x, y⟩
    simp only [Prod.mk.injEq, true_and]
    funext i
    simp [BitVec.sub_add_cancel]

end RoopLean.Rev
