# Math

All numbers are 64-bit floats. Angles are measured in **radians** unless noted.

## Constants

| Name | Value |
|------|-------|
| `PI` | π (3.14159…) |

## Angles

| Function | Signature | Description |
|----------|-----------|-------------|
| `rad(x)` | `Number -> Number` | Convert degrees to radians. |
| `deg(x)` | `Number -> Number` | Convert radians to degrees. |

## Trigonometry

| Function | Signature | Description |
|----------|-----------|-------------|
| `sin(x)` | `Number -> Number` | Sine. |
| `cos(x)` | `Number -> Number` | Cosine. |
| `tan(x)` | `Number -> Number` | Tangent. |
| `asin(x)` | `Number -> Number` | Inverse sine. |
| `acos(x)` | `Number -> Number` | Inverse cosine. |
| `atan(x)` | `Number -> Number` | Inverse tangent. |
| `atan2(y, x)` | `(Number, Number) -> Number` | Two-argument arctangent. |

## Hyperbolic

| Function | Signature | Description |
|----------|-----------|-------------|
| `sinh(x)` | `Number -> Number` | Hyperbolic sine. |
| `cosh(x)` | `Number -> Number` | Hyperbolic cosine. |
| `tanh(x)` | `Number -> Number` | Hyperbolic tangent. |
| `asinh(x)` | `Number -> Number` | Inverse hyperbolic sine. |
| `acosh(x)` | `Number -> Number` | Inverse hyperbolic cosine. |
| `atanh(x)` | `Number -> Number` | Inverse hyperbolic tangent. |

## Rounding and Sign

| Function | Signature | Description |
|----------|-----------|-------------|
| `round(x)` | `Number -> Number` | Round to the nearest integer. |
| `floor(x)` | `Number -> Number` | Round down. |
| `ceil(x)` | `Number -> Number` | Round up. |
| `abs(x)` | `Number -> Number` | Absolute value. |

## Exponential and Logarithm

| Function | Signature | Description |
|----------|-----------|-------------|
| `log(x)` | `Number -> Number` | Natural logarithm. |
| `exp(x)` | `Number -> Number` | Exponential, e^x. |

## Comparison

| Function | Signature | Description |
|----------|-----------|-------------|
| `max(a, b)` | `(Number, Number) -> Number` | Larger of two numbers. |
| `min(a, b)` | `(Number, Number) -> Number` | Smaller of two numbers. |
| `clamp(x, lo, hi)` | `(Number, Number, Number) -> Number` | Constrain `x` to the range `[lo, hi]`. |

## Vectors

These operate on tuples treated as vectors.

| Function | Signature | Description |
|----------|-----------|-------------|
| `magnitude(v)` | `Tuple -> Number` | Length of the vector (any arity). |
| `normalize(v)` | `Tuple -> Tuple` | Unit vector in the same direction. |
| `dot(a, b)` | `(Tuple, Tuple) -> Number` | Dot product of two equal-length vectors. |
