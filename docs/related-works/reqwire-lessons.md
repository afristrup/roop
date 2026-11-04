# Lessons from ReQWIRE

Source: Robert Rand, Jennifer Paykin, Dong-Ho Lee, and Steve Zdancewic,
[*ReQWIRE: Reasoning about Reversible Quantum Circuits*](https://doi.org/10.4204/EPTCS.287.17),
QPL 2018, pp. 299-312. This note uses its quantum-circuit work as a design
analogy for Roop.

## The problem

ReQWIRE studies scratch qubits. A program initialises a qubit, uses it, then
claims that it has returned to a known state before disposal. A false claim can
make an optimisation unsound. The paper compares conservative and optimised
behaviour. An assertion is valid when the two agree (§3, pp. 302-304).

Roop has the same problem with temporary values. The compiler can remove a
temporary only when the inverse can restore its earlier value.

## Paper results

- ReQWIRE defines conservative and optimised behaviour separately. Its proof
  accepts an assertion when their denotations agree (§3, pp. 302-304).
- Its source-symmetry rule proves common scratch-wire programs valid without a
  semantic proof (§4, pp. 304-306).
- A programmer can supply a semantic proof when the structural rule does not
  apply (§1, p. 301).
- Its Boolean compiler computes into an ancilla, uses the result, reruns the
  computation to clear the ancilla, then asserts the initial value (§5,
  pp. 306-308).
- The proof, rather than a separate assumption, permits scratch-space reuse
  (§7, pp. 310-311).

## Current implementation

`roop-check` exposes `clean_temporaries`. It returns a `CleanTemporary` record
only when an ancilla starts at zero, the matching calls write only that
ancilla, neither call has other effects, and no intervening statement changes
an input that the call reads.

With `clear_ancillas = true`, `roop-llvm` uses that record to replace the
matching inverse call with a store of the zero value. It does this for scalar
and array ancillas. With the option off, the compiler emits the inverse call.

The Lean translator models the source program. Its restoration theorem checks
that the temporary returns to its entry value at the end of the scope. It does
not yet prove that LLVM implements the optimisation.

The LLVM tests check both IR forms and run LLVM's verifier. The Lean tests
prove restoration for a temporary built by a call and cleared by its inverse.

## Further work

The optimisation does not remove `logged` history, rollback records, or Weave
activations. A pass that removes those values needs its own restoration
condition. Its tests must cover calls, control flow, loop iterations, and
rollback.

## Limits

ReQWIRE's structural condition is sufficient, not necessary. Roop should not
reject a valid program only because a fast checker cannot prove it (§3, p. 304;
§4, pp. 304-305). Several lemmas and the source-symmetry conjecture remained
unproved in the published development (§7, p. 311). The paper's compiler also
handles Boolean expressions only. Its space-saving result shows a useful
pattern, not a cost model for general Roop programs (§5, pp. 306-308; §7,
pp. 311-312).
