# Tuples

| Function | Signature | Description |
|----------|-----------|-------------|
| `fst(t)` | `(a, b) -> a` | First element of a 2-tuple. |
| `snd(t)` | `(a, b) -> b` | Second element of a 2-tuple. |

```vexor
val p = (3, 4)
val x = fst(p)   -- 3
val y = snd(p)   -- 4
```

For tuples of other arities, take them apart with a `match` pattern — see
[Tuples](../language/tuples.md) in the language guide.
