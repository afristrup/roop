import Cslib.Foundations.Semantics.LTS.Basic

namespace RoopLean

def forwards (f : State → State) : Cslib.LTS State Unit where
  Tr s _ t := f s = t

end RoopLean
