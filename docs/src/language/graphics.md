# Graphics

Graphics are the values Vexor renders to SVG. You build them with constructors,
adjust them with transforms, and emit them with `export`.

## Constructors

```vexor
val c = Circle(10)        -- circle of radius 10
val r = Rect(100, 50)     -- 100 x 50 rectangle
val t = Text("hi")        -- text
val g = Group([c, r])     -- combine graphics into one
```

Every constructor produces a **pure** shape positioned at the origin `(0, 0)`. Use
the [transforms](#transforms) to move, scale, or rotate it, and the
[style functions](#style-functions) to color it.

`Group` is useful for treating several graphics as a single component: a transform
applied to a group affects all of its children together, as one unit.

A child's transforms are **relative to its group**. The point `(0, 0)` inside a
group is the group's own origin, which — once the group is moved, scaled, or rotated
— can be somewhere other than the center of the canvas.

```vexor
val pair = Group([Circle(10), Rect(20, 20)])

export pair >> move(50, 50) >> rotate(rad(30))   -- moves and rotates both
```

See the [graphics standard library](../stdlib/graphics.md) for every constructor,
including `Line`, `Curve`, `Path`, and `sample`.

## Transforms

Transforms take a graphic and return a new one, so they chain well with the pipe
operator `>>`:

```vexor
export Circle(20)
  >> move(100, 100)
  >> rotate(rad(45))
```

Available transforms: `move`, `scale`, `rotate`, `mirrorX`, `mirrorY`.

## Style Functions

Style functions set how a graphic is painted. Like transforms, they take a graphic
and return a new one, so they chain with `>>`:

```vexor
export Circle(20)
  >> fill(rgb(255, 100, 0))
  >> stroke(2, rgb(0, 0, 0))
```

Available style functions: `fill`, `stroke`. Colors come from the
[color constructors](../stdlib/color.md).

## Paths

A `Path` is built from path-building steps (`jumpTo`, `lineTo`, `curveTo`, `close`).
Like other constructors, a path is pure and **starts at the origin `(0, 0)`** — so a
leading `jumpTo(0, 0)` is unnecessary; the first `lineTo`/`curveTo` already draws from
the origin. Use `jumpTo` only when you want the path to begin somewhere else.

```vexor
export Path([
  lineTo(50, 0),    -- draws from the origin (0, 0)
  lineTo(50, 50),
  close
])
```

## Joining Paths

The `+` operator joins two paths into one. The right path is **translated so that its
start point lands on the end point of the left path**, then the two are concatenated
into a single continuous path. Only the right path moves — the joined result takes its
position from the **left** path.

```vexor
-- A horizontal segment, then a vertical one
val across = Path([lineTo(50, 0)])   -- from (0,0) to (50, 0)
val down   = Path([lineTo(0, 50)])   -- from (0,0) to (0, 50)

export across + down
-- `down` is shifted by (50, 0) so its start meets `across`'s end,
-- producing an L-shape from (0,0) -> (50,0) -> (50,50)
```

Because the result is itself a path, joins chain left-to-right and combine with
transforms and the pipe operator:

```vexor
val step = Path([lineTo(20, 0), lineTo(20, -20)])

-- Repeat the same step three times, each starting where the last ended
export step + step + step
  >> stroke(2, rgb(0, 0, 0))
```

Building a closed triangle by joining edges:

```vexor
val e1 = Path([lineTo(60, 0)])
val e2 = Path([lineTo(-30, 50)])
val e3 = Path([lineTo(-30, -50)])

export e1 + e2 + e3
  >> close
  >> fill(rgb(80, 160, 255))
```

## Exporting

A program needs at least one `export`. Each `export` produces one graphic in the
output:

```vexor
export Circle(10)
export Rect(20, 20)
```

To export every graphic in a list, use `export each`:

```vexor
val circles = [1..5] >> map(Circle)

export each circles
```
