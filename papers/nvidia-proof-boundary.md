# The proof boundary for NVIDIA execution

## Abstract

Roop can generate and launch PTX on NVIDIA hardware. A runtime test passed on an NVIDIA GeForce RTX 4090 with driver 595.91.07. The test launched generated PTX and checked writable-buffer copyback. This is execution evidence. It is not a proof that CUDA execution refines Roop\x27s Lean semantics.

This paper states the boundary precisely. A useful verification claim has three parts: a source theorem, a lowering theorem, and a backend theorem. Roop currently has evidence for the first part on supported source programs and a hardware test for a narrow part of the third. The middle theorem and a CUDA refinement theorem remain open.

## The claims

Let \(P\) be a well-formed Roop program, \(\llbracket P\rrbracket\) its Lean state transition, and \(K(P)\) the kernel emitted by the compiler.

The source claim is

\[
\llbracket \operatorname{inv}(P)\rrbracket(\llbracket P\rrbracket(s))=s
\]

where the language model covers the program\x27s guards, stack state, and supported effects.

The compiler claim is a correspondence relation

\[
K(P) \sqsubseteq \llbracket P\rrbracket.
\]

The backend claim relates a PTX launch and its device buffers to the kernel semantics:

\[
\operatorname{launch}_{\mathrm{PTX}}(K(P),m)=m\x27
\implies
m\x27 \sim \llbracket P\rrbracket(m).
\]

Together they permit a statement about the running GPU program. Without the latter two claims, a Lean theorem describes the model rather than the emitted executable.

## Current evidence

`cargo test -p roop-rt --test cuda -- --nocapture` passed on the RTX 4090 configuration recorded in [benchmarks.md](../docs/benchmarks.md). The test covers PTX launch and host-visible writable-buffer copyback. It does not range over all kernels, block shapes, alias patterns, memory ordering, failures, or floating-point behavior.

The same benchmark record does not report CUDA throughput. It would be incorrect to call the backend competitive before measurements compare end-to-end Roop kernels with a stated workload, data-transfer policy, precision, warm-up, and baseline.

## A narrow route to a theorem

Start with one integer additive kernel. Give the kernel a functional specification over `BitVec 64` buffers. Prove that the Roop lowering emits that kernel only when its alias and index conditions hold. Then validate or prove a PTX-level refinement for that restricted launch configuration.

The tensor coupling certificate in [the companion paper](compositional-reversible-networks.md) supplies an appropriate source specification. For an update \(p\leftarrow p+f(q)\), the CUDA result must equal the pointwise modular sum and preserve \(q\). Its inverse kernel must compute pointwise subtraction from the same preserved \(q\).

This order keeps trusted assumptions visible. Hardware execution tests remain valuable regression tests, but the theorem boundary stays explicit.

## Measurements

The first NVIDIA performance report should separate allocation and transfer time from kernel time. It should report device, driver, CUDA toolchain, tensor shape, element type, operation count, launch configuration, repetitions, warm-up policy, and variance. It should compare the same arithmetic and data-residency model against a named CUDA baseline. Reversibility adds a second measure: the cost of forward execution followed by inverse execution, including any retained history.
