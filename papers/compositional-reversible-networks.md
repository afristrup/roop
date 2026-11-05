# Compositional certificates for reversible neural networks

## Abstract

Roop programs use fixed-width machine integers and generate inverse code. This paper gives a small Lean certificate calculus for a class of reversible neural networks. A layer preserves one tensor and adds a function of it into the other tensor. The function is opaque to the reversibility proof. A network is an arbitrary list of certified layers. Lean proves both round trips for every list length and every fixed tensor width.

The result avoids proving reversibility by expanding a whole network. It does not prove that the current Roop compiler recognizes generated Weave code as this calculus. It also does not prove that a CUDA kernel implements the Lean function.

## Machine arithmetic

Let \(B_n\) be `BitVec n`. Addition and subtraction are modulo \(2^n\), so for all \(a,b \in B_n\),

\[
(a+b)-b=a, \qquad (a-b)+b=a.
\]

For a tensor \(T_{d,n}=\operatorname{Fin}(d)\to B_n\), these equations hold pointwise. Lean proves tensor equality with function extensionality. The proof has no case split over the \(d\) indices.

## Coupling layers

A layer has a direction and a function \(f:T_{d,n}\to T_{d,n}\). Its two forms are

\[
U_f(q,p)=(q,p+f(q)),
\]

and

\[
V_f(q,p)=(q+f(p),p).
\]

Their inverses replace addition with subtraction. Neither theorem requires \(f\) to be linear, injective, differentiable, or itself reversible. The untouched tensor supplies the input needed to recompute \(f\) during reversal.

The implementation is in [TensorNetwork.lean](../crates/roop-lean/lean/RoopLean/Certificate/TensorNetwork.lean). `CouplingLayer.certificate` selects \(U_f\) or \(V_f\). Its state type is a pair of fixed-width tensors.

## Arbitrary networks

For a list of layers \(L=[\ell_1,\ldots,\ell_k]\), `couplingNetwork L` builds a `Program` whose forward execution is the layers in list order and whose reverse execution is their inverses in reverse order. The generic `Program.certificate` theorem gives

\[
\operatorname{undo}_L(\operatorname{exec}_L(s))=s,
\qquad
\operatorname{exec}_L(\operatorname{undo}_L(s))=s.
\]

These theorems quantify over \(d\), \(n\), the list length, every layer direction, and every update function. A proof checks the program structure and the selected layer certificates. It need not normalize the tensor computation inside any update.

The regression test `arbitrary_coupling_networks_have_a_checked_roundtrip_certificate` runs Lean on this module. It protects the theorem as part of the Rust test suite.

## Scope

This theorem applies to networks assembled from additive coupling updates over modular bitvectors. It does not establish numerical accuracy, gradient correctness, resource bounds, alias safety in emitted source, or compiler and backend refinement. Those claims require separate theorems.

## Next proof boundary

The compiler should emit a `CouplingLayer` certificate when it lowers a Weave layer whose source update preserves one tensor and performs pointwise modular addition on the other. The correspondence proof must show that the generated Einsum computation reads only the preserved tensor and that its destination is the tensor named by the certificate. Once that boundary is proved, network depth changes only the length of a checked list.
