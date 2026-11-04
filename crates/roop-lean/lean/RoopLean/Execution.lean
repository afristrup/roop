import Cslib.Foundations.Semantics.LTS.Reverse
import RoopLean.Reverse

namespace RoopLean

theorem inverse_execution (f g : State → State)
    (g_after_f : ∀ s, g (f s) = s) (f_after_g : ∀ s, f (g s) = s)
    (run : (forwards f).Execution s labels t states) :
    (forwards g).Execution t labels.reverse s states.reverse := by
  rw [← reverse_forwards f g g_after_f f_after_g]
  exact run.reverse

end RoopLean
