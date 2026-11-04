# Effects, history, and rollback

The reversible core cannot overwrite a value. `logged` gives an overwrite a
place to store its old value. The forward code pushes the old value to a
`Stack`. The inverse checks the new value, pops the old value, and restores it.

```rust
logged history {
    balance = amount * 2;
}
```

The history parameter is part of the function state. It is not hidden compiler
storage. A function can reverse an overwrite because its caller gives it the
old information back through the same stack.

`try ... catch_rollback ... -> outcome` has a different effect. If the body
fails, the generated rollback path invokes the inverse of every completed
action. The handler starts from the restored state. `outcome` records whether
the body or the handler ran, so the inverse has a branch selector.

The compiler generates failure-aware variants for functions called from a
reversible `try`. A failure reports to the enclosing rollback path only after
the callee has restored its arguments. This is why a failed transfer does not
leave its debit behind.

The runtime carries a separate history for the external world. It tracks input,
pending output, filesystem changes, and values kept with `keep`. `commit` and
`forget` make parts of that history irreversible.

[Previous: reversibility as a program invariant](02-reversibility.md)

[Next: Frobenius monads](04-frobenius.md)
