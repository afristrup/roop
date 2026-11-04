# Frobenius monads

The formal account of Roop's reversible effects is in
[`crates/roop-lean/lean/Frobenius.lean`](../../crates/roop-lean/lean/Frobenius.lean).
It follows Heunen and Karvonen's "Reversible monadic computing" in the category
of sets and relations.

A relation can have more than one output. Reversing a relation swaps its input
and output. In the Lean file, `Rel.dagger` implements that converse relation.
For a bijective function, the dagger is its ordinary inverse. The file proves
that adding a fixed bit-vector has subtraction as its dagger, and xor is its own
dagger.

An effect can preserve reversibility when its monad satisfies the Frobenius law.
For a writer effect `A × G`, the logged value is in a group `G`. The inverse
returns the original result and writes the inverse group element. The Lean
theorem `kleisli_dagger_inverts_the_log` states this directly.

Roop uses this form for the outcome of a reversible `try`. The outcome is a
boolean under xor. `false` is the identity and every boolean is its own inverse.
The theorem `reversing_a_try_keeps_the_outcome` explains why the reverse path
reads the same bit.

A value stack is not a Frobenius monoid under list concatenation. The Lean file
proves a counterexample in `lists_are_not_frobenius`. Roop does not reverse a
whole stack effect as one algebraic operation. Instead, every `pop` reverses its
matching `push`. That distinction matters when changing history semantics.

[Previous: effects, history, and rollback](03-effects.md)

[Next: Lean verification](05-lean.md)
