# Graphics

Functions for constructing, transforming, and building up graphics. A *point* is a
2-tuple `(x, y)`.

Every constructor produces a **pure** shape positioned at the origin `(0, 0)`; apply
[transforms](#transforms) to place it. Two paths can be joined with `+` — see
[Joining Paths](../language/graphics.md#joining-paths) in the language guide.

## Constructors

| Function | Signature | Description |
|----------|-----------|-------------|
| `Circle(radius)` | `Number -> Graphic` | A circle. |
| `Rect(width, height)` | `(Number, Number) -> Graphic` | A rectangle. |
| `Text(content)` | `String -> Graphic` | Text. |
| `Group(children)` | `[Graphic] -> Graphic` | Combine graphics into one. |
| `Line(x, y)` | `(Number, Number) -> Graphic` | A line from the origin `(0, 0)` to the point `(x, y)`. |
| `Curve(p1, p2, p3)` | `(Point, Point, Point) -> Graphic` | A cubic Bézier curve drawn from the origin `(0, 0)`. |
| `Path(steps)` | `[Graphic -> Graphic] -> Graphic` | Build a path from a list of path steps. |
| `sample(from, to, steps, f)` | `(Number, Number, Number, Number -> Point) -> Graphic` | Sample `f` over `[from, to]` in `steps` intervals and draw a smooth path through the points. |

## Transforms

Each transform takes a graphic and returns a new one, so they chain with `>>`.

| Function | Signature | Description |
|----------|-----------|-------------|
| `move(x, y)(g)` | `(Number, Number) -> Graphic -> Graphic` | Translate by `(x, y)`. |
| `scale(s)(g)` | `Number -> Graphic -> Graphic` | Scale uniformly. |
| `rotate(angle)(g)` | `Number -> Graphic -> Graphic` | Rotate (angle in radians). |
| `mirrorX(g)` | `Graphic -> Graphic` | Flip vertically (negate y). |
| `mirrorY(g)` | `Graphic -> Graphic` | Flip horizontally (negate x). |

## Style Functions

Like transforms, each style function takes a graphic and returns a new one, so they
chain with `>>`. Colors come from the [color constructors](color.md).

| Function | Signature | Description |
|----------|-----------|-------------|
| `fill(color)(g)` | `Color -> Graphic -> Graphic` | Set the fill color. |
| `strokeColor(color)(g)` | `Color -> Graphic -> Graphic` | Set the stroke color. |
| `strokeWidth(width)(g)` | `Number -> Graphic -> Graphic` | Set the stroke width. |
| `strokeJoin(kind)(g)` | `String -> Graphic -> Graphic` | Set the line join: `"miter"`, `"round"`, or `"bevel"`. |
| `strokeCap(kind)(g)` | `String -> Graphic -> Graphic` | Set the line cap: `"butt"`, `"round"`, or `"square"`. |
| `opacity(n)(g)` | `Number -> Graphic -> Graphic` | Set the opacity (`0`–`1`). |
| `setId(name)(g)` | `String -> Graphic -> Graphic` | Set the SVG `id`. |

## Path Steps

Pass these to `Path([...])` to build up a shape.

| Function | Signature | Description |
|----------|-----------|-------------|
| `jumpTo(x, y)` | `(Number, Number) -> Graphic -> Graphic` | Move the cursor without drawing. |
| `lineTo(x, y)` | `(Number, Number) -> Graphic -> Graphic` | Draw a line segment to `(x, y)`. |
| `curveTo(p1, p2, p3)` | `(Point, Point, Point) -> Graphic -> Graphic` | Draw a cubic Bézier curve. |
| `close(g)` | `Graphic -> Graphic` | Close the path. |

## Sampling Parametric Paths

`sample(from, to, steps, f)` builds a path from a **parametric function**. The
parameter `t` is swept from `from` to `to` in `steps` equal intervals; at each value
`f(t)` is called to produce a point `(x, y)`, and a smooth curve is drawn through all
of the sampled points. `f` must return a 2-tuple and `steps` must be at least 1.

```vexor
set canvas(200, 200)

-- A circle of radius 80, traced by a parametric function of the angle
fn circle(t) = (cos(t) * 80, sin(t) * 80)

export sample(0, 2 * PI, 64, circle) >>
  strokeWidth(2) >> strokeColor(rgb(0, 0, 0))
```

Changing the parametric function gives a different curve. Here a spiral, where the
radius grows with `t`:

```vexor
fn spiral(t) = (cos(t) * t * 5, sin(t) * t * 5)

export sample(0, 6 * PI, 120, spiral) >>
  strokeWidth(2) >> strokeColor(rgb(200, 0, 120))
```

More sample `steps` means a smoother, more accurate curve at the cost of a larger
path.

## Example

```vexor
set canvas(200, 200)

export Path([
  jumpTo(20, 20),
  lineTo(180, 20),
  lineTo(100, 180),
  close
]) >>
  fill(rgb(80, 160, 255)) >>
  strokeWidth(2) >> strokeColor(rgb(0, 0, 0))
```
