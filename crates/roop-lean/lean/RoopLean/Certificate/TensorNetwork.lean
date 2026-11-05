import RoopLean.Certificate.ProgramInverseExecution
import RoopLean.Certificate.TensorAddUpdate

namespace RoopLean.Rev

abbrev Tensor (d n : Nat) := Fin d → BitVec n

def addIntoFirst (f : Tensor d n → Tensor d n) :
    Rev (Tensor d n × Tensor d n) (Tensor d n × Tensor d n) where
  run := fun (x, y) => (fun i => x i + f y i, y)
  undo := fun (x, y) => (fun i => x i - f y i, y)
  undo_run := by
    rintro ⟨x, y⟩
    simp only [Prod.mk.injEq]
    constructor
    · funext i
      simp [BitVec.add_sub_cancel]
    · trivial
  run_undo := by
    rintro ⟨x, y⟩
    simp only [Prod.mk.injEq]
    constructor
    · funext i
      simp [BitVec.sub_add_cancel]
    · trivial

structure CouplingLayer (d n : Nat) where
  intoFirst : Bool
  update : Tensor d n → Tensor d n

def CouplingLayer.certificate (layer : CouplingLayer d n) :
    Rev (Tensor d n × Tensor d n) (Tensor d n × Tensor d n) :=
  if layer.intoFirst then addIntoFirst layer.update else tensorAddUpdate layer.update

def couplingNetwork (layers : List (CouplingLayer d n)) : Program (Tensor d n × Tensor d n) :=
  layers.foldr (fun layer rest => .step layer.certificate rest) .done

theorem couplingNetwork_undo_run (layers : List (CouplingLayer d n)) (state : Tensor d n × Tensor d n) :
    (couplingNetwork layers).undo ((couplingNetwork layers).exec state) = state :=
  (couplingNetwork layers).certificate.undo_run state

theorem couplingNetwork_run_undo (layers : List (CouplingLayer d n)) (state : Tensor d n × Tensor d n) :
    (couplingNetwork layers).exec ((couplingNetwork layers).undo state) = state :=
  (couplingNetwork layers).certificate.run_undo state

theorem couplingNetwork_inverse_execution (layers : List (CouplingLayer d n))
    {s t : Tensor d n × Tensor d n}
    {labels : List Unit} {states : List (Tensor d n × Tensor d n)}
    (run : (forwards (couplingNetwork layers).exec).Execution s labels t states) :
    (forwards (couplingNetwork layers).undo).Execution t labels.reverse s states.reverse :=
  (couplingNetwork layers).inverse_execution run

end RoopLean.Rev
