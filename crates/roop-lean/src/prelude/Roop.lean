import Std.Tactic.BVDecide

set_option linter.unusedSimpArgs false
set_option linter.unusedVariables false
set_option linter.unusedTactic false
set_option maxRecDepth 4096

namespace Roop

/-- Why a roop function stopped. `assertion` is a failed exit assertion,
`ancilla` an ancilla that was not restored, `fuel` a loop that did not end. -/
inductive Fail where
  | assertion | outOfBounds | ancilla | fuel
  deriving Repr, DecidableEq

/-- Functions are pure: they return their results or the reason they failed.
This is the functional model of Aeneas: no mutation, mutable parameters come
back as results. -/
abbrev Res (α : Type) := Except Fail α

/-- Machine integers wrap around, exactly like the compiled code. -/
abbrev I64 := BitVec 64

def check (c : Bool) (e : Fail) : Res Unit := if c then .ok () else .error e

def aget {α : Type} {n : Nat} (v : Vector α n) (i : I64) : Res α :=
  if h : i.toNat < n then .ok (v[i.toNat]'h) else .error .outOfBounds

def aset {α : Type} {n : Nat} (v : Vector α n) (i : I64) (x : α) : Res (Vector α n) :=
  if h : i.toNat < n then .ok (v.set i.toNat x h) else .error .outOfBounds

def fuelBound : Nat := 1000000

/-- A reversible loop `from entry { body } loop { step } until stop`:
assert `entry`, then alternate `body` and `step` until `stop` holds, asserting
that `entry` is false again after every `step`. -/
def janusGo {σ : Type} (entry stop : σ → Res Bool) (body step : σ → Res σ) :
    Nat → σ → Res σ
  | 0, _ => .error .fuel
  | n + 1, s => do
      let s1 ← body s
      if (← stop s1) then pure s1 else do
        let s2 ← step s1
        check (!(← entry s2)) .assertion
        janusGo entry stop body step n s2

def janus {σ : Type} (entry stop : σ → Res Bool) (body step : σ → Res σ) (s : σ) : Res σ := do
  check (← entry s) .assertion
  janusGo entry stop body step fuelBound s

end Roop
