namespace RoopLean

structure Rev (α β : Type) where
  run : α → β
  undo : β → α
  undo_run : ∀ x, undo (run x) = x
  run_undo : ∀ y, run (undo y) = y

end RoopLean
