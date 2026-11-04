import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def addUpdateAt (i : Fin n) (f : α → BitVec w) :
    Rev (α × Vector (BitVec w) n) (α × Vector (BitVec w) n) where
  run := fun (x, y) => (x, y.set i.val (y[i.val] + f x) i.isLt)
  undo := fun (x, y) => (x, y.set i.val (y[i.val] - f x) i.isLt)
  undo_run := by
    rintro ⟨x, y⟩
    simp [Vector.getElem_set_self, BitVec.add_sub_cancel]
  run_undo := by
    rintro ⟨x, y⟩
    simp [Vector.getElem_set_self, BitVec.sub_add_cancel]

end RoopLean.Rev
