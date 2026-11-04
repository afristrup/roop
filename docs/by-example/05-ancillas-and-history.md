# Ancillas and history

An ancilla is temporary state. It starts at a declared value and must have that
value again before its scope ends.

```rust
fn add_twice(value: &mut i64, amount: &i64) {
    ancilla twice: i64 = 0;
    twice += amount;
    twice += amount;
    value += twice;
    twice -= amount;
    twice -= amount;
}
```

`twice` begins as zero and ends as zero. Roop accepts the function because no
information remains in its temporary value.

Some work must replace a value. A history stack keeps the old value so the
replacement still has an inverse.

```rust
fn settle_to(balance: &mut i64, amount: &i64, history: &mut Stack<i64, 8>) {
    logged history {
        balance = amount * 2;
    }
}
```

`logged` pushes information that an assignment would otherwise destroy onto
`history`. Running the function backwards restores that information from the
stack.

An ancilla stack must also be empty when its scope ends. A common shape is to
compute a value, copy the useful result to a caller-owned place, then uncall the
work that filled the temporary stack.

```rust
fn quote(balance: &mut i64, amount: &i64, quoted: &mut i64) {
    ancilla history: Stack<i64, 8> = empty;
    call settle_to(balance, amount, history);
    quoted += balance;
    uncall settle_to(balance, amount, history);
}
```

`quoted` keeps the result. The inverse call returns `balance` and `history` to
their initial states.

[Previous: branches and loops](04-control-flow.md)

[Next: failure and rollback](06-rollback.md)
