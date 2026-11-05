namespace RoopLean

def Refines (representation : Machine → Source → Prop)
    (run : Machine → Machine) (semantics : Source → Source) : Prop :=
  ∀ machine source, representation machine source → representation (run machine) (semantics source)

theorem inverse_refines {Machine Source : Type}
    (representation : Machine → Source → Prop)
    (forward inverse : Source → Source)
    (run_forward run_inverse : Machine → Machine)
    (source_inverse : ∀ source, inverse (forward source) = source)
    (forward_refines : Refines representation run_forward forward)
    (inverse_refines : Refines representation run_inverse inverse)
    {machine : Machine} {source : Source}
    (represented : representation machine source) :
    representation (run_inverse (run_forward machine)) source := by
  rw [← source_inverse source]
  exact inverse_refines _ _ (forward_refines _ _ represented)

end RoopLean
