import RoopLean.Certificate.Rev

namespace RoopLean

inductive Program (α : Type) where
  | done
  | step (head : Rev α α) (tail : Program α)

end RoopLean
