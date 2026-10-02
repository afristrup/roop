import Lean
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

/-- Two runs agree when both fail, whatever the reason, or both end in the same
state. Parallel iterations may fail in any order, so that is all they promise. -/
def agree {σ : Type} : Res σ → Res σ → Prop
  | .ok a, .ok b => a = b
  | .error _, .error _ => True
  | _, _ => False

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

theorem janusGo_no_ancilla {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ)
    (hE : ∀ s, E s ≠ .error .ancilla) (hS : ∀ s, S s ≠ .error .ancilla)
    (hB : ∀ s, B s ≠ .error .ancilla) (hP : ∀ s, P s ≠ .error .ancilla) :
    ∀ (n : Nat) (s : σ), janusGo E S B P n s ≠ .error .ancilla := by
  intro n
  induction n with
  | zero => intro s h; simp [janusGo] at h
  | succ n ih =>
    intro s h
    simp only [janusGo, bind, Except.bind] at h
    rcases hb : B s with e | t
    · simp only [hb, Except.error.injEq] at h; exact hB s (hb ▸ h ▸ rfl)
    · simp only [hb] at h
      rcases hs : S t with e | c
      · simp only [hs, Except.error.injEq] at h; exact hS t (hs ▸ h ▸ rfl)
      · simp only [hs] at h
        cases c
        · simp only [Bool.false_eq_true, if_false] at h
          rcases hp : P t with e | s'
          · simp only [hp, Except.error.injEq] at h; exact hP t (hp ▸ h ▸ rfl)
          · simp only [hp] at h
            rcases he : E s' with e | c
            · simp only [he, check, Except.error.injEq] at h; exact hE s' (he ▸ h ▸ rfl)
            · simp only [he, check] at h
              cases c
              · simp only [Bool.not_false, if_true] at h
                exact ih s' h
              · simp [pure, Except.pure] at h
        · simp [pure, Except.pure] at h

/-- A loop whose entry, exit, body and step never fail on an unrestored ancilla
never does either. -/
theorem janus_no_ancilla {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ)
    (hE : ∀ s, E s ≠ .error .ancilla) (hS : ∀ s, S s ≠ .error .ancilla)
    (hB : ∀ s, B s ≠ .error .ancilla) (hP : ∀ s, P s ≠ .error .ancilla) (s : σ) :
    janus E S B P s ≠ .error .ancilla := by
  intro h
  simp only [janus, bind, Except.bind] at h
  rcases he : E s with e | c
  · simp only [he, Except.error.injEq] at h; exact hE s (he ▸ h ▸ rfl)
  · simp only [he, check] at h
    cases c
    · simp [pure, Except.pure] at h
    · simp only [if_true] at h
      exact janusGo_no_ancilla E S B P hE hS hB hP _ s h

/-- The value a place holds when it is empty: what a push leaves behind. -/
class HasZero (α : Type) where
  zero : α

instance {n : Nat} : HasZero (BitVec n) := ⟨0⟩
instance : HasZero Bool := ⟨false⟩
instance : HasZero Float := ⟨0.0⟩

/-- A history stack of at most `n` values. Slots above `len` hold zero, so
pushing and popping are exact inverses. -/
@[ext] structure Stack (α : Type) (n : Nat) where
  len : Nat
  data : Vector α n
  deriving DecidableEq

def Stack.empty {α : Type} [HasZero α] {n : Nat} : Stack α n :=
  ⟨0, Vector.replicate n HasZero.zero⟩

def Stack.push {α : Type} [HasZero α] [BEq α] {n : Nat} (s : Stack α n) (x : α) :
    Res (Stack α n) :=
  if h : s.len < n then
    if s.data[s.len]'h == HasZero.zero then .ok ⟨s.len + 1, s.data.set s.len x h⟩
    else .error .assertion
  else .error .outOfBounds

def Stack.pop {α : Type} [HasZero α] {n : Nat} (s : Stack α n) : Res (α × Stack α n) :=
  if h : 0 < s.len ∧ s.len - 1 < n then
    .ok (s.data[s.len - 1]'h.2, ⟨s.len - 1, s.data.set (s.len - 1) HasZero.zero h.2⟩)
  else .error .outOfBounds

theorem Stack.pop_push {α : Type} [HasZero α] [BEq α] [LawfulBEq α] {n : Nat}
    {s s' : Stack α n} {x : α} (h : s.pop = .ok (x, s')) : s'.push x = .ok s := by
  unfold Stack.pop at h
  split at h
  · rename_i hh
    simp only [Except.ok.injEq, Prod.mk.injEq] at h
    obtain ⟨hx, hs⟩ := h
    subst hx hs
    unfold Stack.push
    simp only
    have hlt : s.len - 1 < n := hh.2
    simp [hlt]
    have e : s.len - 1 + 1 = s.len := by omega
    rw [e]
  · simp at h


theorem Stack.push_pop {α : Type} [HasZero α] [BEq α] [LawfulBEq α] {n : Nat}
    {s s' : Stack α n} {x : α} (h : s.push x = .ok s') : s'.pop = .ok (x, s) := by
  unfold Stack.push at h
  split at h
  · rename_i hh
    split at h
    · rename_i hz
      simp only [Except.ok.injEq] at h
      subst h
      unfold Stack.pop
      have hlt : s.len + 1 - 1 < n := by omega
      simp [hlt]
      have hz' : s.data[s.len] = HasZero.zero := beq_iff_eq.mp hz
      have back : s.data.set s.len HasZero.zero hh = s.data := by
        rw [← hz']; exact Vector.set_getElem_self ..
      simp [hh, back]
    · simp at h
  · simp at h


/-- `try body catch handler -> failed`: run the body; if it fails, its effects
are gone (the model is pure, so they never happened), and the handler runs on
the original state. The outcome says which side ran. -/
def tryCatch {σ : Type} (body handler : σ → Res σ) (s : σ) : Res (σ × Bool) :=
  match body s with
  | .ok t => .ok (t, false)
  | .error _ => do let t ← handler s; pure (t, true)

/-- Reversing a `try`: the outcome selects the side to undo. -/
def untry {σ : Type} (body_inv handler_inv : σ → Res σ) (s : σ) (failed : Bool) : Res σ :=
  if failed then handler_inv s else body_inv s

theorem tryCatch_inv {σ : Type} (B H Bi Hi : σ → Res σ)
    (hb : ∀ a b, B a = .ok b → Bi b = .ok a) (hh : ∀ a b, H a = .ok b → Hi b = .ok a)
    (s : σ) (p : σ × Bool) (h : tryCatch B H s = .ok p) : untry Bi Hi p.1 p.2 = .ok s := by
  obtain ⟨r, t⟩ := p
  simp only
  unfold tryCatch at h
  rcases e : B s with err | u
  · simp only [e, bind, Except.bind, pure, Except.pure] at h
    rcases e2 : H s with err2 | v
    · simp [e2] at h
    · simp only [e2] at h
      simp only [Except.ok.injEq, Prod.mk.injEq] at h
      obtain ⟨rfl, rfl⟩ := h
      simp [untry, hh _ _ e2]
  · simp only [e] at h
    simp only [Except.ok.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    simp [untry, hb _ _ e]

theorem tryCatch_no_ancilla {σ : Type} (B H : σ → Res σ)
    (hH : ∀ s, H s ≠ .error .ancilla) (s : σ) :
    tryCatch B H s ≠ .error .ancilla := by
  intro h
  unfold tryCatch at h
  rcases e : B s with err | u
  · simp only [e, bind, Except.bind, pure, Except.pure] at h
    rcases e2 : H s with err2 | v
    · simp only [e2, Except.error.injEq] at h; exact hH s (e2 ▸ h ▸ rfl)
    · simp [e2] at h
  · simp [e] at h

theorem untry_no_ancilla {σ : Type} (Bi Hi : σ → Res σ)
    (hB : ∀ s, Bi s ≠ .error .ancilla) (hH : ∀ s, Hi s ≠ .error .ancilla)
    (s : σ) (t : Bool) : untry Bi Hi s t ≠ .error .ancilla := by
  unfold untry
  split
  · exact hH s
  · exact hB s

open Lean Elab Tactic Meta in
/-- Applies a loop lemma to every hypothesis that mentions the same loop pieces
as the lemma's premise, and keeps the results. -/
elab "roop_loop " l:ident : tactic => withMainContext do
  let needed ← forallTelescope (← getConstInfo l.getId).type fun xs _ => do
    return (← inferType xs.back!).getUsedConstants.filter fun c => c.getRoot == `Fn || (`Roop.Stack).isPrefixOf c
  for decl in (← getLCtx) do
    if decl.isImplementationDetail then continue
    let present := decl.type.getUsedConstants
    unless needed.all present.contains do continue
    try
      let app ← mkAppM l.getId #[decl.toExpr]
      let ty ← instantiateMVars (← inferType app)
      unless (← getLCtx).any (fun d => !d.isImplementationDetail && d.type == ty) ||
          (← withReducible ((← getLCtx).anyM fun d => do
            return !d.isImplementationDetail && (← isDefEq d.type ty))) do
        liftMetaTactic fun g => do
          let (_, g) ← (← g.assert `this ty app).intro1
          return [g]
    catch _ => pure ()

open Lean Elab Tactic Meta in
/-- Case split on the first conditional with a closed condition that no
hypothesis decides yet, in the goal or in a hypothesis, and resolve that
condition everywhere. -/
elab "roop_cases" : tactic => withMainContext do
  let mut types := #[← instantiateMVars (← getMainTarget)]
  let mut known : Array Expr := #[]
  let mut hyps : Array Syntax.Term := #[]
  for decl in (← getLCtx) do
    unless decl.isImplementationDetail do
      let type ← instantiateMVars decl.type
      types := types.push type
      known := known.push type
      if ← isProp decl.type then
        hyps := hyps.push (← Term.exprToSyntax decl.toExpr)
  let decided (c : Expr) : Bool := known.any fun t =>
    t == c || t == mkNot c ||
      match c.eq?, t.eq? with
      | some (_, x, _), some (_, y, _) => x == y
      | _, _ => false
  let found := types.findSome? fun type => type.find? fun e =>
    (e.isAppOfArity ``dite 5 || e.isAppOfArity ``ite 5) && !(e.getArg! 1).hasLooseBVars &&
      !decided (e.getArg! 1)
  let some e := found | throwError "no conditional left"
  let cond ← Term.exprToSyntax (e.getArg! 1)
  let hc := mkIdent `hc
  let mut tac ← `(tactic| by_cases $hc : $cond)
  for h in hyps do
    tac ← `(tactic| ($tac <;> try simp only [$hc:ident, ↓reduceDIte, ↓reduceIte] at $h:term))
  tac ← `(tactic| ($tac <;> try simp only [$hc:ident, ↓reduceDIte, ↓reduceIte]))
  evalTactic tac

end Roop
