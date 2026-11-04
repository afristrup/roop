import RoopLean.Certificate.Program

namespace RoopLean

def Program.undo : Program α → α → α
  | .done => id
  | .step head tail => head.undo ∘ tail.undo

end RoopLean
