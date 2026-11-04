# Why reversible programs need different rules

Most programs throw information away. After this assignment, the old value of
`x` has gone:

```rust
x = 0;
```

No reverse program can recover it. Roop therefore rejects that assignment in a
reversible function.

Instead, Roop programs change state with operations that have an inverse. If a
program adds `amount`, its reverse subtracts `amount`.

```rust
fn add_fee(balance: &mut i64, amount: &i64) {
    balance += amount;
}
```

If `balance` begins as `10` and `amount` is `3`, the function leaves `13`. The
inverse starts with `13` and subtracts `3`, returning `10`.

Roop creates that inverse for every reversible `fn`. You call a function in the
usual direction with `call` and in the other direction with `uncall`.

```rust
fn round_trip(balance: &mut i64, amount: &i64) {
    call add_fee(balance, amount);
    uncall add_fee(balance, amount);
}
```

At the end of `round_trip`, `balance` has its original value.

This does not mean a Roop program has no effects. It means that reversible code
must account for the information needed to undo its effects. The next chapter
looks at the updates that make this possible.

[Next: changing state without losing it](02-updates.md)
