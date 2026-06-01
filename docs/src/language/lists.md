# Lists

A list is an ordered sequence of values.

## Literals

```vexor
val xs = [1, 2, 3, 4]
```

The empty list is written `[]` or `Nil`:

```vexor
val empty = []
val also_empty = Nil
```

## Ranges

Build a list from a range of numbers (inclusive):

```vexor
[1..10]      -- 1, 2, 3, ..., 10
[1, 3..10]   -- 1, 3, 5, 7, 9  (the first two values set the step)
```

## Cons

The `:` operator prepends an element:

```vexor
val xs = 1 : [2, 3]   -- [1, 2, 3]
```

It is also used in patterns to split a list into head and tail:

```vexor
val head = match [1, 2, 3] {
  x : rest => x,   -- x = 1
  [] => 0
}
```

## Pipelines

Lists combine naturally with the standard [list functions](../stdlib/lists.md) and
the pipe operator:

```vexor
val doubled = [1..5] >> map(x -> x * 2)   -- [2, 4, 6, 8, 10]
```

See the [list standard library](../stdlib/lists.md) for the full set of operations.
