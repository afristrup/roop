import RoopLean.Certificate.Rev

namespace RoopLean.Rev

def inverse (f : Rev α β) : Rev β α where
  run := f.undo
  undo := f.run
  undo_run := f.run_undo
  run_undo := f.undo_run

end RoopLean.Rev
