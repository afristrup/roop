# Hybrid quantum programs

`roop hybrid` compiles programs in HQbricks∞, the hybrid while language of
Issa, Chareton and Péchoux, *An Effective Quantum Hoare Logic for Hybrid Quantum
Programs with Unbounded Loops* (arXiv:2607.08548). A program has qubits, bits
and ints, Clifford+R_k gates, measurement, `if`, and unbounded `while`.

```
qubit q; bit c; int x;
do
    H(q); c := measure q;
    x := x + 1
while (¬c)
```

## The language

```
i ::= k | x | i + i | i * i | i ^ i
b ::= tt | ff | 0 | 1 | c | i <= i | i = i | b && b | b || b | !b
p ::= skip | qubit q | bit c | int x
    | X(q) | Z(q) | H(q) | S(q) | T(q) | CNOT(q, q) | R(q, k) | Rk(q)
    | x := i | c := b | c := measure q
    | p; p | if b then p else p end | while b do p done | do p while b
```

`¬`, `∧`, `∨` and `≤` work too, and so do `and`, `or` and `not`. `R(q, k)` is
R_k = RZ(2π/2^k), so `S` is R_2 and `T` is R_3. Measurement is in the
computational basis and keeps the qubit, collapsed.

The checker implements the paper's judgment `s1 ⊢ p : s2`. Memory is declared
before use and only once, both branches of an `if` declare the same names, a
loop declares nothing, and a gate never uses one qubit twice. Expressions are
typed, and the literals 0 and 1 are bits as well as ints.

## The pipeline

1. **Parse and check**, then **lower** to the hybrid IR. A *kernel* is a run of
   gates the device takes in one go. Measurements, assignments, `if` and
   `while` run on the host.
2. **Optimize.** Gates move back into the open kernel past classical
   assignments, which touch no qubit, and past measurements of other qubits.
   This cuts host to device round trips, for example from 10 to 6.67 expected
   kernels in `programs/nested.hyq`. In each kernel, a gate moves back past the
   gates it commutes with until it cancels (X X, H H, CNOT CNOT) or merges
   (phases on one qubit add, exactly, as dyadic fractions of a turn).
   Assignments that are overwritten before anything reads them are dropped.
   `-O0` turns all of this off.
3. **Emit.** `--emit qasm` writes OpenQASM 3 with real-time control: the
   registers `q` and `c`, ints as `int[64] v_<name>`, and `while`/`if` on
   measurement results. `--emit ir` shows the kernels and host code.

## Running it exactly

`--emit report` runs the program on cq-states, the semantics of Feng and Ying,
*Quantum Hoare logic with classical variables* (arXiv:2008.06812). A cq-state
maps each classical store to a partial density operator. A loop is the limit
Σ_n F_¬b (⟦p⟧ ∘ F_b)^n of the paper's Fig. 3, unrolled until the mass left in
the loop is below 1e-12 or `--iterations` rounds have run. The report gives
the probability of termination, the mass still running when unrolling
stopped, the mean of every variable on termination, and the expected number of
kernels, measurements, loop rounds and gates of each kind (T count
included).

```
$ roop hybrid crates/roop-hybrid/programs/synth.hyq --emit report
terminated with probability 1.000000000
...
  E[x] = 1.333333333
```

This is the E[x] = 4/3 that the paper derives with IHPS for repeat-until-success
synthesis. The interpreter is exact up to the truncation it reports. It is
not a proof: the paper's IHPS invariants prove these limits symbolically, and
roop does not produce them. It holds at most 10 qubits, since the density
operator has 4^n entries.
