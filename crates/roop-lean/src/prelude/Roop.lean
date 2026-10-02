import Std.Tactic.BVDecide

set_option linter.unusedSimpArgs false
set_option linter.unusedVariables false
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


/-- A terminating run of a loop: `k` steps from `s` to the exit state `r`. -/
inductive RunN {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ) : Nat → σ → σ → Prop
  | stop {s t : σ} : B s = .ok t → S t = .ok true → RunN E S B P 0 s t
  | step {k : Nat} {s t s' r : σ} : B s = .ok t → S t = .ok false → P t = .ok s' →
      E s' = .ok false → RunN E S B P k s' r → RunN E S B P (k + 1) s r

theorem runN_of_go {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ) :
    ∀ (n : Nat) (s r : σ), janusGo E S B P n s = .ok r → ∃ k, k < n ∧ RunN E S B P k s r := by
  intro n
  induction n with
  | zero => intro s r h; simp [janusGo] at h
  | succ n ih =>
    intro s r h
    simp only [janusGo, bind, Except.bind] at h
    rcases hb : B s with e | t
    · simp [hb] at h
    · simp only [hb] at h
      rcases hs : S t with e | c
      · simp [hs] at h
      · simp only [hs] at h
        cases c
        · simp only [Bool.false_eq_true, if_false] at h
          rcases hp : P t with e | s'
          · simp [hp] at h
          · simp only [hp] at h
            rcases he : E s' with e | c
            · simp [he, check] at h
            · simp only [he, check] at h
              cases c
              · simp only [Bool.not_false, if_true] at h
                obtain ⟨k, hk, run⟩ := ih s' r h
                exact ⟨k + 1, by omega, RunN.step hb hs hp he run⟩
              · simp at h
        · simp only [if_true] at h
          cases h
          exact ⟨0, by omega, RunN.stop hb hs⟩

/-- What a loop does after running its body to the state `s`: stop, or step on. -/
def janusK {σ : Type} (stop entry : σ → Res Bool) (body step : σ → Res σ) (m : Nat)
    (s : σ) : Res σ := do
  if (← stop s) then pure s else do
    let t ← step s
    check (!(← entry t)) .assertion
    janusGo entry stop body step m t

theorem janusGo_succ {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ) (n : Nat) (s : σ) :
    janusGo E S B P (n + 1) s = (do let s1 ← B s; janusK S E B P n s1) := rfl

theorem runN_back {σ : Type} (E S : σ → Res Bool) (B P Bi Pi : σ → Res σ)
    (hb : ∀ a b, B a = .ok b → Bi b = .ok a) (hp : ∀ a b, P a = .ok b → Pi b = .ok a)
    {k : Nat} {s r : σ} (run : RunN E S B P k s r) :
    ∀ (m : Nat) (z : σ), janusK E S Bi Pi m s = .ok z → janusGo S E Bi Pi (m + k + 1) r = .ok z := by
  induction run with
  | stop hb' hs =>
    intro m z h
    rw [Nat.add_zero, janusGo_succ]
    simp [bind, Except.bind, hb _ _ hb', h]
  | @step k s t s' r hb' hs hp' he run ih =>
    intro m z h
    have : janusK E S Bi Pi (m + 1) s' = .ok z := by
      simp [janusK, bind, Except.bind, he, hp _ _ hp', hs, check]
      rw [janusGo_succ]
      simp [bind, Except.bind, hb _ _ hb', h]
    have := ih (m + 1) z this
    rw [show m + (k + 1) + 1 = m + 1 + k + 1 by omega]
    exact this

theorem runN_stop {σ : Type} {E S : σ → Res Bool} {B P : σ → Res σ} {k : Nat} {s r : σ}
    (run : RunN E S B P k s r) : S r = .ok true := by
  induction run with
  | stop _ hs => exact hs
  | step _ _ _ _ _ ih => exact ih

/-- The inverse of a reversible loop is the loop with entry and exit swapped,
run with the inverse body and step. -/
theorem janus_inv {σ : Type} (E S : σ → Res Bool) (B P Bi Pi : σ → Res σ)
    (hb : ∀ a b, B a = .ok b → Bi b = .ok a) (hp : ∀ a b, P a = .ok b → Pi b = .ok a)
    (s r : σ) (h : janus E S B P s = .ok r) : janus S E Bi Pi r = .ok s := by
  simp only [janus, bind, Except.bind] at h ⊢
  rcases he : E s with e | c
  · simp [he] at h
  · cases c
    · simp [he, check] at h
    · simp only [he, check, if_true] at h
      obtain ⟨k, hk, run⟩ := runN_of_go E S B P _ s r h
      have back := runN_back E S B P Bi Pi hb hp run (fuelBound - k - 1) s
        (by simp [janusK, bind, Except.bind, pure, Except.pure, he])
      rw [show fuelBound - k - 1 + k + 1 = fuelBound by omega] at back
      simp [runN_stop run, check, back]

open Lean Elab Tactic Meta in
/-- Applies a loop lemma to every hypothesis it accepts and keeps the results. -/
elab "roop_loop " l:ident : tactic => withMainContext do
  for decl in (← getLCtx) do
    if decl.isImplementationDetail then continue
    try
      let app ← mkAppM l.getId #[decl.toExpr]
      let ty ← inferType app
      liftMetaTactic fun g => do
        let (_, g) ← (← g.assert `this ty app).intro1
        return [g]
    catch _ => pure ()

end Roop
