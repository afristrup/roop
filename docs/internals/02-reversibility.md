# Reversibility as a program invariant

Every reversible function has a forward form and an inverse form. The compiler
constructs the inverse by reversing statement order and replacing each
operation with its inverse.

```text
x += k;          becomes          x -= k;
call f(...);     becomes          uncall f(...);
push h <- x;     becomes          pop h -> x;
```

The checker rejects an update that reads the place it writes. `x += x` maps
several old values to the same result in fixed-width arithmetic, so the inverse
cannot recover the old value. It also tracks overlapping places across calls and
inside parallel iterations.

Branches and loops carry entry or exit assertions. The inverse needs evidence
of the arm or iteration boundary it is reversing. An `if` records that evidence
as the expression after `fi`. A loop records it at `from` and `until`.

An ancilla is a local proof obligation. The checker knows its initial value and
requires that value when the scope ends. This rule makes local scratch storage
disappear from the caller's state.

`irrev` lifts the information-preservation rule. The compiler does not generate
an inverse for a function that contains irreversible code, and reversible code
cannot call it outside an `irrev` block.

[Previous: the compilation path](01-compilation.md)

[Next: effects, history, and rollback](03-effects.md)
