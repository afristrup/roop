import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def id (α : Type) : Rev α α where
  run := fun x => x
  undo := fun x => x
  undo_run := fun _ => rfl
  run_undo := fun _ => rfl

end RoopLean.Rev
