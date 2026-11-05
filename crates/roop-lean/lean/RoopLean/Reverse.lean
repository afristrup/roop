import Cslib.Foundations.Semantics.LTS.Reverse
import RoopLean.Lts

namespace RoopLean

theorem reverse_forwards {α : Type} (f g : α → α)
    (g_after_f : ∀ s, g (f s) = s) (f_after_g : ∀ s, f (g s) = s) :
    (forwards f).reverse = forwards g := by
  ext s _ t
  simp only [Cslib.LTS.reverse_tr, forwards]
  constructor
  · intro h
    rw [← h, g_after_f]
  · intro h
    rw [← h, f_after_g]

end RoopLean
