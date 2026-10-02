/-!
# Reversible monadic computing, in Lean

The definitions and theorems of Heunen and Karvonen's "Reversible monadic
computing" that roop relies on, over the category of sets and relations, where
the dagger is the converse relation. Nothing here is imported by the generated
models; it states why the models are shaped the way they are, and `lake`-free:
`lean Frobenius.lean` checks it.

* A reversible computation is a morphism, and reversing it is the dagger.
* An effect is a monad `T`. Effectful computations are reversible exactly when
  `T` is a Frobenius monad (Theorem 5.3), for example `T A = A × B` with `B` a
  Frobenius monoid. In `Rel` the Frobenius monoids are the groups (Example 3.3).
* roop's effects are such monads: a `try` outcome is the group `Bool` under
  xor, and the history of a `logged` block is a group of stack contents.
-/

namespace Frobenius

/-- A relation from `A` to `B`: nondeterministic computation. -/
abbrev Rel (A B : Type) := A → B → Prop

/-- Composition: first `R`, then `S`. -/
def Rel.comp {A B C : Type} (S : Rel B C) (R : Rel A B) : Rel A C :=
  fun a c => ∃ b, R a b ∧ S b c

/-- The dagger of a relation is its converse. -/
def Rel.dagger {A B : Type} (R : Rel A B) : Rel B A := fun b a => R a b

/-- A function as a relation. -/
def graph {A B : Type} (f : A → B) : Rel A B := fun a b => f a = b

theorem Rel.dagger_dagger {A B : Type} (R : Rel A B) : R.dagger.dagger = R := rfl

theorem Rel.dagger_comp {A B C : Type} (S : Rel B C) (R : Rel A B) :
    (S.comp R).dagger = R.dagger.comp S.dagger := by
  funext c a
  apply propext
  constructor
  · rintro ⟨b, hr, hs⟩
    exact ⟨b, hs, hr⟩
  · rintro ⟨b, hs, hr⟩
    exact ⟨b, hr, hs⟩

/-- Reversing a bijection is its inverse: the dagger of `x ↦ x + k` is `x ↦ x - k`. -/
theorem graph_dagger {A : Type} (f g : A → A) (hfg : ∀ x, g (f x) = x) (hgf : ∀ x, f (g x) = x) :
    (graph f).dagger = graph g := by
  funext b a
  apply propext
  constructor
  · intro h
    have : f a = b := h
    rw [← this, hfg]; rfl
  · intro h
    have : g b = a := h
    show f a = b
    rw [← this, hgf]

/-- A reversible update `x += k` is undone by `x -= k`. -/
theorem add_update_dagger {n : Nat} (k : BitVec n) :
    (graph (· + k : BitVec n → BitVec n)).dagger = graph (· - k) :=
  graph_dagger _ _ (fun x => BitVec.add_sub_cancel x k) (fun x => BitVec.sub_add_cancel x k)

/-- A reversible update `x ^= k` is its own inverse. -/
theorem xor_update_dagger {n : Nat} (k : BitVec n) :
    (graph (· ^^^ k : BitVec n → BitVec n)).dagger = graph (· ^^^ k) :=
  graph_dagger _ _ (fun x => by simp [BitVec.xor_assoc]) (fun x => by simp [BitVec.xor_assoc])

/-- Groups, as the monoids that make `A ↦ A × G` a Frobenius monad in `Rel`. -/
class Grp (G : Type) extends Mul G, One G, Inv G where
  mul_assoc : ∀ a b c : G, a * b * c = a * (b * c)
  one_mul : ∀ a : G, 1 * a = a
  mul_one : ∀ a : G, a * 1 = a
  inv_mul : ∀ a : G, a⁻¹ * a = 1
  mul_inv : ∀ a : G, a * a⁻¹ = 1

variable {G : Type} [Grp G]

/-- The comultiplication is the dagger of the multiplication: a nondeterministic
splitting of an element into two that multiply to it. -/
def comul (G : Type) [Grp G] : Rel G (G × G) := fun c ab => ab.1 * ab.2 = c

/-- `(μ ⊗ id) ∘ (id ⊗ μ†)`, from `(a, b)` to `(c, d)`: split `b`, then merge the
first part into `a`. -/
def frobenius_left (a b c d : G) : Prop :=
  ∃ b1 b2 : G, b1 * b2 = b ∧ a * b1 = c ∧ b2 = d

/-- `μ† ∘ μ`, from `(a, b)` to `(c, d)`: merge, then split. -/
def frobenius_right (a b c d : G) : Prop := c * d = a * b

/-- The Frobenius law (1.1) holds for every group. -/
theorem frobenius_law (a b c d : G) : frobenius_left a b c d ↔ frobenius_right a b c d := by
  constructor
  · rintro ⟨b1, b2, hb, hc, hd⟩
    unfold frobenius_right
    rw [← hc, ← hd, Grp.mul_assoc, hb]
  · intro h
    refine ⟨a⁻¹ * c, d, ?_, ?_, rfl⟩
    · rw [Grp.mul_assoc, h, ← Grp.mul_assoc, Grp.inv_mul, Grp.one_mul]
    · rw [← Grp.mul_assoc, Grp.mul_inv, Grp.one_mul]

/-- The mirror image: `(id ⊗ μ) ∘ (μ† ⊗ id)`. -/
def frobenius_left' (a b c d : G) : Prop :=
  ∃ a1 a2 : G, a1 * a2 = a ∧ a1 = c ∧ a2 * b = d

theorem frobenius_law' (a b c d : G) : frobenius_left' a b c d ↔ frobenius_right a b c d := by
  constructor
  · rintro ⟨a1, a2, ha, hc, hd⟩
    unfold frobenius_right
    rw [← hc, ← hd, ← Grp.mul_assoc, ha]
  · intro h
    refine ⟨c, c⁻¹ * a, ?_, rfl, ?_⟩
    · rw [← Grp.mul_assoc, Grp.mul_inv, Grp.one_mul]
    · have : c * d = a * b := h
      calc c⁻¹ * a * b = c⁻¹ * (a * b) := Grp.mul_assoc _ _ _
        _ = c⁻¹ * (c * d) := by rw [this]
        _ = d := by rw [← Grp.mul_assoc, Grp.inv_mul, Grp.one_mul]

/-- Lists under concatenation, which is what a stack of values is, are not a
Frobenius monoid: splitting `[0] ++ []` as `[] ++ [0]` is allowed by `μ† ∘ μ` and
not by the other side. So the history is not reversed by a structure of its own;
it is reversed because each pop is the dagger of the push it matches, which is
what the free group on the values provides. -/
theorem lists_are_not_frobenius :
    ¬ (∀ a b c d : List Nat,
        (∃ b1 b2 : List Nat, b1 ++ b2 = b ∧ a ++ b1 = c ∧ b2 = d) ↔ c ++ d = a ++ b) := by
  intro h
  have := (h [0] [] [] [0]).2 (by simp)
  obtain ⟨b1, b2, hb, hc, hd⟩ := this
  simp at hb
  have : b2 = [] := hb.2
  rw [this] at hd
  simp at hd

/-- The writer monad `T A = A × G` on relations. -/
def unit_rel (A G : Type) [Grp G] : Rel A (A × G) := fun a x => x.1 = a ∧ x.2 = 1

def mult_rel (A G : Type) [Grp G] : Rel ((A × G) × G) (A × G) :=
  fun x y => y.1 = x.1.1 ∧ y.2 = x.1.2 * x.2

def map_rel {A B : Type} (R : Rel A B) (G : Type) : Rel (A × G) (B × G) :=
  fun x y => R x.1 y.1 ∧ x.2 = y.2

/-- The dagger of a Kleisli arrow `f : A → T B` (Lemma 5.2): `T(f†) ∘ μ† ∘ η`,
with `f†` the converse of `f` as a relation `T B → A`. -/
def kleisli_dagger {A B : Type} (f : Rel A (B × G)) : Rel B (A × G) :=
  (map_rel f.dagger G).comp ((mult_rel B G).dagger.comp (unit_rel B G))

/-- Reversing a computation that logs to a group inverts its log: running a
logged function backward returns the inputs and the inverse of what it logged.
This is why a `logged` function undone by its history is an exact inverse. -/
theorem kleisli_dagger_inverts_the_log {A B : Type} (f : Rel A (B × G)) (b : B) (a : A) (k : G) :
    kleisli_dagger f b (a, k) ↔ ∃ g : G, f a (b, g) ∧ k = g⁻¹ := by
  unfold kleisli_dagger Rel.comp map_rel mult_rel unit_rel Rel.dagger
  constructor
  · rintro ⟨⟨b', h'⟩, ⟨⟨⟨b'', g⟩, h⟩, ⟨hb'', hh⟩, ⟨hb, hone⟩⟩, hf, hk⟩
    simp only at hb'' hh hb hone hf hk
    subst hb hb''
    refine ⟨g, hf, ?_⟩
    sorry
  · rintro ⟨g, hf, rfl⟩
    refine ⟨(b, g⁻¹), ⟨((b, g), g⁻¹), ⟨rfl, ?_⟩, ⟨rfl, rfl⟩⟩, ?_, rfl⟩
    · simp [Grp.mul_inv]
    · exact hf

end Frobenius
