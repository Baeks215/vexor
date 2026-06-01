# Types

Vexor has a small set of built-in types. Types are inferred at runtime — there are
**no type annotations** in the language.

| Type | Example | Notes |
|------|---------|-------|
| Number | `10`, `3.14`, `-2` | 64-bit floating point. |
| Bool | `true`, `false` | |
| String | `"hello"` | Double-quoted. |
| List | `[1, 2, 3]`, `[1..10]` | Homogeneous sequences. See [Lists](./lists.md). |
| Tuple | `(x, y)`, `(1, 2, 3)` | Fixed-size, any arity. See [Tuples](./tuples.md). |
| Color | `rgb(255, 0, 0)` | Built with color constructors. |
| Graphic | `Circle(10)` | Shapes that can be exported. See [Graphics](./graphics.md). |

## Numbers

All numbers are 64-bit floats, so there is no separate integer type:

```vexor
val r = 10
val half = r / 2
```

## Comments

Line comments start with `--` and run to the end of the line:

```vexor
-- This is a comment
val x = 5   -- so is this
```
