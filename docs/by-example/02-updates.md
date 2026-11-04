# Changing state without losing it

An update needs enough information for its inverse. Roop has several updates
with that property.

```rust
fn update(x: &mut i64, y: &mut i64, amount: &i64) {
    x += amount;
    y -= amount;
    x ^= y;
    x <=> y;
}
```

`+=` and `-=` undo one another. `^=` is undone by the same xor operation. `<=>`
swaps two places, and a second swap restores both values.

An update may read a value that it does not write. It cannot read the same place
that it changes:

```rust
// Rejected. The old value of x cannot be recovered from the result.
// x += x;
```

For the same reason, a mutable argument cannot overlap another argument to the
same call. The compiler rejects `call f(x, x)` when the first parameter can
change `x`.

Multiplication and division also have rules. Integer multiplication checks that
the factor is non-zero and that the result does not overflow. Integer division
checks that the divisor is non-zero and divides exactly. Those checks make the
two operations inverse operations.

```rust
fn scale(x: &mut i64, factor: &i64) {
    x *= factor;
    x /= factor;
}
```

`scale` finishes with the value it started with when both checks pass.

[Previous: why reversible programs need different rules](01-why-reversible.md)

[Next: functions and their inverses](03-functions.md)
