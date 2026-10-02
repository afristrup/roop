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

/-- A guarded multiplication `x *= e` runs when the factor is nonzero and the
product does not overflow; its inverse `x /= e` when the factor is nonzero and
the division is exact. Under those guards each undoes the other. -/
def mulOk (x e : BitVec 64) : Bool := !(e == 0) && !(BitVec.smulOverflow x e)
def divOk (x e : BitVec 64) : Bool :=
  !(e == 0) && (BitVec.srem x e == 0) && !(x == BitVec.intMin 64 && e == -1)

theorem toInt_ne_zero {e : BitVec 64} (h : e ≠ 0) : e.toInt ≠ 0 := by
  intro hz
  apply h
  apply BitVec.eq_of_toInt_eq
  simpa using hz

theorem toInt_bounds (x : BitVec 64) : -9223372036854775808 ≤ x.toInt ∧ x.toInt < 9223372036854775808 := by
  have h1 := BitVec.le_toInt (x := x)
  have h2 := BitVec.toInt_lt (x := x)
  simp at h1 h2
  omega


theorem toInt_zero64 : (0 : BitVec 64).toInt = 0 := by simp

theorem scale_unscale (x e : BitVec 64) (h : mulOk x e = true) :
    divOk (x * e) e = true ∧ BitVec.sdiv (x * e) e = x := by
  simp only [mulOk, Bool.and_eq_true, Bool.not_eq_true', beq_eq_false_iff_ne, ne_eq] at h
  obtain ⟨he, hs⟩ := h
  have hs' : ¬ BitVec.smulOverflow x e = true := by simp [hs]
  have hm := BitVec.toInt_mul_of_not_smulOverflow hs'
  have hb := toInt_ne_zero he
  have hx := toInt_bounds x
  have hsd : BitVec.sdiv (x * e) e = x := by
    apply BitVec.eq_of_toInt_eq
    rw [BitVec.toInt_sdiv, hm, Int.mul_tdiv_cancel _ hb]
    exact Int.bmod_eq_of_le (by omega) (by omega)
  refine ⟨?_, hsd⟩
  simp only [divOk, Bool.and_eq_true, Bool.not_eq_true', beq_eq_false_iff_ne, ne_eq, Bool.not_eq_true,
    beq_iff_eq, Bool.and_eq_false_imp]
  refine ⟨⟨he, ?_⟩, ?_⟩
  · apply BitVec.eq_of_toInt_eq
    rw [BitVec.toInt_srem, hm, Int.mul_tmod_left, toInt_zero64]
  · intro hmin hneg
    have h1 : (x * e).toInt = (BitVec.intMin 64).toInt := by rw [hmin]
    have h2 : e.toInt = -1 := by rw [hneg]; simp
    rw [hm, h2, BitVec.toInt_intMin] at h1
    simp at h1
    omega

theorem unscale_scale (x e : BitVec 64) (h : divOk x e = true) :
    mulOk (BitVec.sdiv x e) e = true ∧ BitVec.sdiv x e * e = x := by
  simp only [divOk, Bool.and_eq_true, Bool.not_eq_true', beq_eq_false_iff_ne, ne_eq,
    beq_iff_eq, Bool.and_eq_false_imp] at h
  obtain ⟨⟨he, hr⟩, hn⟩ := h
  have hb := toInt_ne_zero he
  have hx := toInt_bounds x
  have hr' : x.toInt.tmod e.toInt = 0 := by
    have := congrArg BitVec.toInt hr
    rwa [BitVec.toInt_srem, toInt_zero64] at this
  have hdvd : e.toInt ∣ x.toInt := Int.dvd_of_tmod_eq_zero hr'
  have hc : x.toInt.tdiv e.toInt * e.toInt = x.toInt := Int.tdiv_mul_cancel hdvd
  obtain ⟨c, hcdef⟩ : ∃ c, c = x.toInt.tdiv e.toInt := ⟨_, rfl⟩
  rw [← hcdef] at hc
  have habs : c.natAbs ≤ x.toInt.natAbs := by
    have h1 : x.toInt.natAbs = c.natAbs * e.toInt.natAbs := by
      rw [← hc, Int.natAbs_mul]
    have h2 : 0 < e.toInt.natAbs := Int.natAbs_pos.mpr hb
    rw [h1]
    exact Nat.le_mul_of_pos_right _ h2
  have hnotmin : ¬ (c = 9223372036854775808) := by
    intro hc2
    have h1 : x.toInt.natAbs = c.natAbs * e.toInt.natAbs := by
      rw [← hc, Int.natAbs_mul]
    rw [hc2] at habs h1
    have h3 : x.toInt = -9223372036854775808 := by omega
    have h4 : e.toInt.natAbs = 1 := by
      rw [h3] at h1
      simp at h1
      omega
    have h5 : e.toInt = -1 ∨ e.toInt = 1 := by omega
    rcases h5 with h5 | h5
    · apply hn
      · apply BitVec.eq_of_toInt_eq
        rw [h3, BitVec.toInt_intMin]; simp
      · apply BitVec.eq_of_toInt_eq
        rw [h5]; simp
    · rw [hc2, h5] at hc
      omega
  have hcr : -9223372036854775808 ≤ c ∧ c < 9223372036854775808 := by omega
  have hq : (BitVec.sdiv x e).toInt = c := by
    rw [BitVec.toInt_sdiv, ← hcdef]
    exact Int.bmod_eq_of_le (by omega) (by omega)
  have hnov : ¬ BitVec.smulOverflow (BitVec.sdiv x e) e = true := by
    simp only [BitVec.smulOverflow, hq, hc, Bool.or_eq_true, decide_eq_true_eq, not_or]
    omega
  refine ⟨?_, ?_⟩
  · simp only [mulOk, Bool.and_eq_true, Bool.not_eq_true', beq_eq_false_iff_ne, ne_eq]
    exact ⟨he, by simpa using hnov⟩
  · apply BitVec.eq_of_toInt_eq
    rw [BitVec.toInt_mul_of_not_smulOverflow hnov, hq, hc]


theorem divOk_mul {x e : BitVec 64} (h : mulOk x e = true) : divOk (x * e) e = true :=
  (scale_unscale x e h).1

theorem sdiv_mul {x e : BitVec 64} (h : mulOk x e = true) : BitVec.sdiv (x * e) e = x :=
  (scale_unscale x e h).2

theorem mulOk_div {x e : BitVec 64} (h : divOk x e = true) : mulOk (BitVec.sdiv x e) e = true :=
  (unscale_scale x e h).1

theorem mul_sdiv {x e : BitVec 64} (h : divOk x e = true) : BitVec.sdiv x e * e = x :=
  (unscale_scale x e h).2

/-- A strict signed comparison separates its operands, also as natural numbers,
which is what array indices are. -/
theorem slt_ne {a b : BitVec 64} (h : BitVec.slt a b = true) : a.toNat ≠ b.toNat := by
  intro heq
  have := BitVec.eq_of_toNat_eq heq
  subst this
  simp [BitVec.slt] at h

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

/-- A loop only ends once its exit test holds, so a counter loop leaves its
counter at the upper bound. -/
theorem janus_stop {σ : Type} (E S : σ → Res Bool) (B P : σ → Res σ)
    (s r : σ) (h : janus E S B P s = .ok r) : S r = .ok true := by
  simp only [janus, bind, Except.bind] at h
  rcases he : E s with e | c
  · simp [he] at h
  · cases c
    · simp [he, check] at h
    · simp only [he, check, if_true] at h
      obtain ⟨k, hk, run⟩ := runN_of_go E S B P _ s r h
      exact runN_stop run

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


theorem Stack.push_ne_ancilla {α : Type} [HasZero α] [BEq α] {n : Nat} {s : Stack α n} {x : α} :
    s.push x ≠ .error .ancilla := by
  unfold Stack.push
  split <;> (try split) <;> simp

theorem Stack.pop_ne_ancilla {α : Type} [HasZero α] {n : Nat} {s : Stack α n} :
    s.pop ≠ .error .ancilla := by
  unfold Stack.pop
  split <;> simp

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
/-- Rewrites the goal with a proof of `a = v` where `v` also occurs in `a`, and
looks inside conjunctions. -/
partial def unfoldCyclic (p : Expr) : TacticM Unit := do
  let ty ← whnfR (← instantiateMVars (← inferType p))
  if ty.isAppOfArity ``And 2 then
    unfoldCyclic (mkApp3 (mkConst ``And.left) ty.appFn!.appArg! ty.appArg! p)
    unfoldCyclic (mkApp3 (mkConst ``And.right) ty.appFn!.appArg! ty.appArg! p)
  else if let some (_, lhs, rhs) := ty.eq? then
    let element := rhs.isApp && rhs.getAppFn.isConstOf ``GetElem.getElem
    if (rhs.isFVar && lhs.containsFVar rhs.fvarId!) || element then
      try
        let stx ← Term.exprToSyntax p
        evalTactic (← `(tactic| rw [← $stx]))
      catch _ => pure ()

open Lean Elab Tactic Meta in
/-- Uses an equation `a = v` where `v` also occurs in `a`, which cannot be
substituted, to rewrite the goal: every `v` in the goal becomes `a`. -/
elab "roop_unfold_cyclic" : tactic => withMainContext do
  for decl in (← getLCtx) do
    unless decl.isImplementationDetail do
      if ← isProp decl.type then unfoldCyclic decl.toExpr

open Lean Elab Tactic Meta in
/-- Opens the first hypothesis that is a conjunction into its parts. -/
elab "roop_ands" : tactic => withMainContext do
  for decl in (← getLCtx) do
    if decl.isImplementationDetail then continue
    let ty ← whnfR (← instantiateMVars decl.type)
    unless ty.isAppOfArity ``And 2 do continue
    try
      let goals ← (← getMainGoal).cases decl.fvarId
      replaceMainGoal (goals.toList.map (·.mvarId))
      return
    catch _ => pure ()
  throwError "no conjunction to open"

open Lean Elab Tactic Meta in
/-- Opens the first local variable that is a pair into its components, so that
facts about its parts (`v.2 = 7`) turn into substitutions. -/
elab "roop_pairs" : tactic => withMainContext do
  for decl in (← getLCtx) do
    if decl.isImplementationDetail then continue
    let ty ← whnfR (← instantiateMVars decl.type)
    unless ty.isAppOfArity ``Prod 2 do continue
    try
      let goals ← (← getMainGoal).cases decl.fvarId
      replaceMainGoal (goals.toList.map (·.mvarId))
      return
    catch _ => pure ()
  throwError "no pair to open"

open Lean Elab Tactic Meta in
partial def vectorFacts (p : Expr) : TacticM (Array Expr) := do
  let ty ← whnfR (← instantiateMVars (← inferType p))
  if ty.isAppOfArity ``And 2 then
    return (← vectorFacts (mkApp3 (mkConst ``And.left) ty.appFn!.appArg! ty.appArg! p)) ++
      (← vectorFacts (mkApp3 (mkConst ``And.right) ty.appFn!.appArg! ty.appArg! p))
  else if let some (t, _, _) := ty.eq? then
    if (← whnfR t).isAppOfArity ``Vector 2 then return #[p] else return #[]
  else return #[]

open Lean Elab Tactic Meta in
/-- Proves an equation of vectors one index at a time, with every vector
equation among the hypotheses read at that index. Works where the hypotheses
are cyclic and cannot be substituted. -/
elab "roop_pointwise" : tactic => withMainContext do
  let some (t, _, _) := (← instantiateMVars (← getMainTarget)).eq? | throwError "not an equation"
  unless (← whnfR t).isAppOfArity ``Vector 2 do throwError "not a vector equation"
  let mut facts : Array Expr := #[]
  for decl in (← getLCtx) do
    unless decl.isImplementationDetail do
      if ← isProp decl.type then facts := facts ++ (← vectorFacts decl.toExpr)
  evalTactic (← `(tactic| (apply Vector.ext; intro $(mkIdent `j) $(mkIdent `hj))))
  for p in facts do
    let stx ← Term.exprToSyntax p
    evalTactic (← `(tactic| have := congrArg (fun v => v[$(mkIdent `j)]'$(mkIdent `hj)) $stx))

open Lean Elab Tactic Meta in
/-- For every hypothesis `a < b` between machine integers, adds that `a` and
`b` differ as natural numbers, in the form simplification produces. -/
elab "roop_slt" : tactic => withMainContext do
  for decl in (← getLCtx) do
    if decl.isImplementationDetail then continue
    let some (_, lhs, rhs) := (← instantiateMVars decl.type).eq? | continue
    unless lhs.isAppOf ``BitVec.slt && rhs.isConstOf ``Bool.true do continue
    try
      let stx ← Term.exprToSyntax decl.toExpr
      evalTactic (← `(tactic| (have hne := Roop.slt_ne $stx; have hne2 := hne.symm; simp at hne hne2)))
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

open Lean Elab Tactic Meta in
/-- Splits an equation between a variable and a term that contains it, such as
`(v.1, w) = v`. Used as a rewrite rule it would never stop; as its components
it is harmless. -/
elab "roop_uncycle" : tactic => withMainContext do
  for decl in (← getLCtx) do
    unless decl.isImplementationDetail do
      let some (_, a, b) := (← instantiateMVars decl.type).eq? | continue
      let cyclic := (b.isFVar && a.containsFVar b.fvarId!) || (a.isFVar && b.containsFVar a.fvarId!)
      if cyclic then
        let h ← Term.exprToSyntax decl.toExpr
        evalTactic (← `(tactic| simp only [Prod.ext_iff] at $h:term))

open Lean Elab Tactic in
/-- Runs a tactic, turning running out of recursion depth, which `first` does
not recover from, into an ordinary failure. -/
elab "roop_catch " t:tactic : tactic =>
  tryCatchRuntimeEx (evalTactic t) fun _ => throwError "recursion limit"

end Roop
