# Tuples

A tuple groups a fixed number of values, which may have different types. Tuples can
have any arity:

```vexor
val point = (3, 4)
val triple = (1, 2, 3)
val mixed = ("origin", 0, 0)
```

## Accessing Elements

`fst` and `snd` return the first and second element of a 2-tuple:

```vexor
val x = fst((3, 4))   -- 3
val y = snd((3, 4))   -- 4
```

## Destructuring

Tuples can be taken apart with patterns in a `match`:

```vexor
val sum = match (10, 20) {
  (a, b) => a + b   -- 30
}
```

Tuple patterns also appear when matching over lists of tuples, such as the output of
[`zip`](../stdlib/lists.md) or [`enumerate`](../stdlib/lists.md):

```vexor
val r = match enumerate([10, 20, 30]) {
  [(_a, _x), (i, _y), _z] => i,   -- i = 1
  _ => 0
}
```
