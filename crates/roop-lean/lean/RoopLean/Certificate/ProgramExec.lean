import RoopLean.Certificate.Program

namespace RoopLean

def Program.exec : Program α → α → α
  | .done => id
  | .step head tail => tail.exec ∘ head.run

end RoopLean
