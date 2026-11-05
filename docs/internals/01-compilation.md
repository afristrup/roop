# The compilation path

The `roop` binary dispatches `build`, `run`, `test`, `lean`, `fmt`, `weave`, and
`lsp`. Its entry point is [the CLI main module](../../crates/roop/src/main.rs).

For a build, the compiler follows this order:

```text
Roop.toml and module roots
        |
module loader
        |
lexer and parser
        |
AST and semantic checks
        |
generic specialisation and optimisation
        |
LLVM IR and native linker
```

`roop-modules` finds `mod.roop` files and resolves imports. `roop-syntax` owns
tokens, parsing, the AST, spans, and visitors. `roop-check` rejects programs
that cannot be reversed safely. `roop-opt` specialises length parameters,
renames generated items, handles Bennett transformations, and applies selected
optimisations. `roop-llvm` emits LLVM IR and target-specific parallel code. For an
NVPTX IR build, `roop build --emit ir` also writes `program.ptx.ll` and
`program.cuda-contract.json`. The JSON file names each emitted kernel and records
its direction, induction variable, and device buffers with their byte size,
alignment, and copyback mode. It is generated from the compiler's kernel
representation and checked against PTX entry names in a CLI regression. It does not
prove that PTX refines the Roop or Lean semantics.

The Lean path shares loading, parsing, and checking. `roop-lean` translates the
checked program into a pure Lean model. `roop weave` uses `roop-weave` to read
model JSON and write Roop source for the ordinary pipeline.

The checker runs early. Code generation assumes that a mutable place does not
alias an incompatible read or write, that ancillas can be restored, and that a
reverse control-flow path exists.

[Next: reversibility as a program invariant](02-reversibility.md)
