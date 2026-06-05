# Lists

List functions are curried: they take the function/count argument first and the list
last, which works well with the pipe operator (`xs >> map(f)`).

## Transforming

| Function | Signature | Description |
|----------|-----------|-------------|
| `map(f)` | `(a -> b) -> [a] -> [b]` | Apply `f` to every element. |
| `filter(f)` | `(a -> Bool) -> [a] -> [a]` | Keep elements where `f` is true. |
| `flatMap(f)` | `(a -> [b]) -> [a] -> [b]` | Map then flatten the results. |
| `reverse(xs)` | `[a] -> [a]` | Reverse the list. |

## Selecting

| Function | Signature | Description |
|----------|-----------|-------------|
| `take(n)` | `Number -> [a] -> [a]` | First `n` elements. |
| `drop(n)` | `Number -> [a] -> [a]` | All but the first `n` elements. |
| `takeWhile(f)` | `(a -> Bool) -> [a] -> [a]` | Leading elements while `f` is true. |
| `dropWhile(f)` | `(a -> Bool) -> [a] -> [a]` | Drop leading elements while `f` is true. |
| `find(f)` | `(a -> Bool) -> [a] -> [a]` | First matching element as a singleton list (or empty). |

## Accessing

These primitives return a single element or a sub-list. `nth`, `head`, `tail`, `last`,
and `init` are **partial**: they raise an error on an out-of-range index or an empty
list. Use `take`/`drop` when you want a total (never-erroring) alternative.

| Function | Signature | Description |
|----------|-----------|-------------|
| `nth(i)` | `Number -> [a] -> a` | Element at index `i` (0-based). Errors if out of range. |
| `head(xs)` | `[a] -> a` | First element. Errors if empty. |
| `tail(xs)` | `[a] -> [a]` | All but the first element. Errors if empty. |
| `last(xs)` | `[a] -> a` | Last element. Errors if empty. |
| `init(xs)` | `[a] -> [a]` | All but the last element. Errors if empty. |
| `isEmpty(xs)` | `[a] -> Bool` | Whether the list has no elements. |

## Folding

| Function | Signature | Description |
|----------|-----------|-------------|
| `foldl(f)(init)(xs)` | `((a, b) -> a) -> a -> [b] -> a` | Left fold, starting from `init`. `f` is called as `f(acc, item)`. |
| `foldr(f)(init)(xs)` | `((a, b) -> a) -> a -> [b] -> a` | Right fold. `f` is called as `f(item, acc)`. |

## Combining

| Function | Signature | Description |
|----------|-----------|-------------|
| `zip(xs)(ys)` | `[a] -> [b] -> [(a, b)]` | Pair up elements of two lists. |
| `zipWith(f)(xs)(ys)` | `((a, b) -> c) -> [a] -> [b] -> [c]` | Combine two lists element-wise with `f(x, y)`. |
| `enumerate(xs)` | `[a] -> [(Number, a)]` | Pair each element with its index. |

## Ordering

| Function | Signature | Description |
|----------|-----------|-------------|
| `sort(xs)` | `[Number] -> [Number]` | Sort numbers ascending. |
| `sortBy(f)` | `((a, a) -> Bool) -> [a] -> [a]` | Sort using a comparator. `f(a, b)` returns `true` when `a` should come *before* `b`. |

## Building and Measuring

| Function | Signature | Description |
|----------|-----------|-------------|
| `len(xs)` | `[a] -> Number` | Number of elements. |
| `repeat(n, x)` | `(Number, a) -> [a]` | A list of `n` copies of `x`. |
| `concat(xs, ys)` | `([a], [a]) -> [a]` | Append two lists (`xs` followed by `ys`). |
| `sum(xs)` | `[Number] -> Number` | Sum of all elements (`0` for an empty list). |
| `product(xs)` | `[Number] -> Number` | Product of all elements (`1` for an empty list). |

## Example

```vexor
val total = [1, 2, 3, 4] >>
  filter(n -> n > 2) >>   -- [3, 4]
  map(n -> n * 10) >>     -- [30, 40]
  foldl((a, b) -> a + b)(0)   -- 70
```
