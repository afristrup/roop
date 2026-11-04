# Functions and their inverses

A reversible function receives places by reference. `&mut` means the function
may change the place. `&` means it may read the place.

```rust
fn transfer(source: &mut i64, destination: &mut i64, amount: &i64) {
    source -= amount;
    destination += amount;
}
```

The compiler knows how to reverse each statement. It therefore creates an
inverse of `transfer` that subtracts from `destination` and adds to `source` in
the opposite order.

```rust
fn undo_transfer(source: &mut i64, destination: &mut i64, amount: &i64) {
    call transfer(source, destination, amount);
    uncall transfer(source, destination, amount);
}
```

`uncall` does not repeat the forward function. It calls its generated inverse.
Arguments remain in the same order because the inverse uses the same input
places.

A `test` runs forward, then Roop runs it backwards. The test fails if the
fixtures do not return to their zero values.

```rust
test transfer_can_be_taken_back {
    source: i64, destination: i64;
    source += 10;
    call transfer(source, destination, 4);
    expect source == 6 && destination == 4;
}
```

The reverse pass returns `source` and `destination` to zero. That gives each
test an extra check for information that the function failed to restore.

[Previous: changing state without losing it](02-updates.md)

[Next: branches and loops](04-control-flow.md)
