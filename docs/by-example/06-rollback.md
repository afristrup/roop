# Failure and rollback

`expect` stops a program when its condition is false. Inside `try`, a failed
expect causes Roop to reverse the work already completed in the try body.

```rust
fn transfer(
    payer: &mut i64,
    payee: &mut i64,
    amount: &i64,
    refusals: &mut i64,
    refused: &mut bool,
) {
    try {
        payer -= amount;
        payee += amount;
        expect payer >= 0;
    } catch_rollback {
        refusals += 1;
    } -> refused;
}
```

When `payer` has insufficient funds, the failed `expect` makes Roop reverse the
two updates. `payer` and `payee` have their previous values before the handler
runs. The handler increments `refusals`.

`refused` records the path that ran. It must begin as `false`. The reverse run
uses it to decide whether to undo the try body or the handler. That stored bit
makes the complete statement reversible.

The body and handler may not access `refused`. A reversible `try` also cannot
contain irreversible code, parallel loops, tasks, or channels. Roop must be
able to reverse every completed action when failure occurs.

For a complete program, read
[`roop/examples/bin/bank.roop`](../../roop/examples/bin/bank.roop). The
[reference](../reference.md#history-rollback-and-monads) covers stacks, rollback,
and irreversible escape hatches.

[Previous: ancillas and history](05-ancillas-and-history.md)

[Back to the contents](README.md)
