# Branches and loops

A forward branch can inspect its condition. A reverse branch needs to know
which arm ran after the branch body has changed the state. Roop puts that fact
after `fi`.

```rust
fn move_if_positive(x: &mut i64, y: &mut i64) {
    if x > 0 {
        x -= 1;
        y += 1;
    } else {
        x += 1;
        y -= 1;
    } fi y > 0;
}
```

The first condition chooses an arm on the forward run. The expression after
`fi` checks the result of that arm. On the reverse run, that expression chooses
the arm and the original condition becomes the check. If either check fails,
the program stops.

Loops use entry and exit assertions for the same reason.

```rust
fn count_to_three(i: &mut i64, total: &mut i64) {
    from i == 0 {
        total += 1;
    } loop {
        i += 1;
    } until i == 2;
    i -= 2;
}
```

The loop begins only when `i == 0`. It repeats until `i == 2`. The final
subtraction restores `i`, so the caller does not observe the loop counter.

Roop can run a proved-disjoint loop in parallel with `#[parallel]`. See the
[reference](../reference.md#parallel-loops) for the aliasing rules and targets.

[Previous: functions and their inverses](03-functions.md)

[Next: ancillas and history](05-ancillas-and-history.md)
