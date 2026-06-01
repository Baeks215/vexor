# Color

Colors are built with these constructors.

## Expected ranges

| Argument | Range |
|----------|-------|
| `r`, `g`, `b` (RGB channels) | 0–255 |
| `h` (hue) | 0–360 |
| `s`, `l` (saturation, lightness) | 0–100 |
| `a` (alpha) | 0–1 |

## Constructors

| Function | Signature | Description |
|----------|-----------|-------------|
| `rgb(r, g, b)` | `(Number, Number, Number) -> Color` | RGB color. |
| `rgba(r, g, b, a)` | `(Number, Number, Number, Number) -> Color` | RGB color with alpha. |
| `hsl(h, s, l)` | `(Number, Number, Number) -> Color` | HSL color. |
| `hsla(h, s, l, a)` | `(Number, Number, Number, Number) -> Color` | HSL color with alpha. |

```vexor
val red = rgb(255, 0, 0)
val glass = rgba(0, 128, 255, 0.5)

export Circle(20) >> fill(red)
```

Colors can also be destructured in a `match` — see [Control Flow](../language/control-flow.md).
