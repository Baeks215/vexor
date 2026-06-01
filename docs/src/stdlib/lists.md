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

## Example

```vexor
val total = [1, 2, 3, 4]
  >> filter(n -> n > 2)   -- [3, 4]
  >> map(n -> n * 10)     -- [30, 40]
  >> foldl((a, b) -> a + b)(0)   -- 70
```
