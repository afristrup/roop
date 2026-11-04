# Roop by example

Roop is a programming language where ordinary code must be able to run in reverse.
That rule shapes every construct in the language. A function that changes a value
must leave enough information to undo the change. A branch must say how the
reverse run chooses the branch. Temporary values must return to their initial
state before their scope ends.

This guide builds those rules one at a time. Read the chapters in order and run
the examples from a checkout of Roop.

```text
cargo build --release -p roop
./target/release/roop run roop/examples/bin/hello.roop
```

The code blocks use the `.roop` language. The examples are small enough to
type into a file, but the repository examples are the source of truth when you
need a complete program.

## Chapters

1. [Why reversible programs need different rules](01-why-reversible.md)
2. [Changing state without losing it](02-updates.md)
3. [Functions and their inverses](03-functions.md)
4. [Branches and loops](04-control-flow.md)
5. [Ancillas and history](05-ancillas-and-history.md)
6. [Failure and rollback](06-rollback.md)

For the full language, see [the reference](../reference.md). For runnable
programs, see [`roop/examples`](../../roop/examples).
