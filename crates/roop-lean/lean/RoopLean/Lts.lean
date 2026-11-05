import Cslib.Foundations.Semantics.LTS.Basic

namespace RoopLean

def forwards {α : Type} (f : α → α) : Cslib.LTS α Unit where
  Tr s _ t := f s = t

end RoopLean
