# Roop internals

This guide describes how the repository turns Roop source into a native program,
a Lean model, and generated Weave code. It explains the rules that matter when
changing those parts. It does not replace the language reference.

## Chapters

1. [The compilation path](01-compilation.md)
2. [Reversibility as a program invariant](02-reversibility.md)
3. [Effects, history, and rollback](03-effects.md)
4. [Frobenius monads](04-frobenius.md)
5. [Lean verification](05-lean.md)
6. [The runtime and external world](06-runtime.md)
7. [Weave code generation](07-weave.md)

Each chapter links to the code it describes. The public language rules are in
[the reference](../reference.md).
