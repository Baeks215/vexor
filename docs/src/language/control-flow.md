# Control Flow

## If / Else

`if`/`else` is an expression — it evaluates to one of the two branches:

```vexor
fn classify(n) =
  if n > 10 {
    100
  } else {
    0
  }
```

## Match

`match` compares a value against a series of patterns, top to bottom, and evaluates
the body of the first one that matches:

```vexor
val label = match n {
  0 => "zero",
  1 => "one",
  _ => "many"
}
```

### Patterns

| Pattern | Matches |
|---------|---------|
| `5`, `"hi"`, `true` | A literal value. |
| `x` | Anything, binding it to `x`. |
| `_` | Anything, binding nothing. |
| `[a, b, c]` | A list of exactly that length. |
| `a : rest` | A non-empty list, splitting head and tail. |
| `(x, y)` | A tuple of that arity. |
| `Circle`, `Rect`, `Text`, `Group`, `Path` | A graphic of that kind. |
| `rgb(r, g, b)`, `rgba(...)`, `hsl(...)`, `hsla(...)` | A color, binding its channels. |

```vexor
val first = match reverse([1, 2, 3, 4]) {
  [a, b, c, d] => a,   -- a = 4
  y => 0
}

fn channels(c) =
  match c {
    rgb(128, a, b) => a + b,
    rgba(128, a, b, alpha) => a + b + alpha
  }
```

### Guards

Add an `if` guard to a pattern to match only when a condition also holds:

```vexor
val size = match n {
  x if x > 10 => "big",
  x if x > 5  => "medium",
  _ => "small"
}
```
