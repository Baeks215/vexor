# Operators

## Arithmetic

| Operator | Meaning |
|----------|---------|
| `+` | Addition |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `//` | Integer (floor) division |
| `%` | Remainder |
| `^` | Exponentiation (right-associative) |
| `-` | Unary negation (prefix) |

## Comparison

`==`, `!=`, `>`, `<`, `>=`, `<=` — compare two **numbers** and return a `Bool`.

## Logic

| Operator | Meaning |
|----------|---------|
| `&&` | Logical AND |
| `\|\|` | Logical OR |
| `!` | Logical NOT (prefix) |

## List Cons

`:` prepends an element to a list (right-associative):

```vexor
val xs = 1 : [2, 3]   -- [1, 2, 3]
```

## Pipe

`>>` feeds a value into a function: `x >> f` is the same as `f(x)`. It reads
left-to-right, which is handy for chaining graphic transforms:

```vexor
val result = 5 >> scaler(2)        -- scaler(2)(5)

export Circle(10)
  >> fill(rgb(255, 0, 0))
  >> move(50, 50)
```

## Precedence

From loosest to tightest binding:

1. `||`
2. `&&`
3. `==`, `!=`, `>`, `<`, `>=`, `<=`
4. `:` (cons)
5. `+`, `-`
6. `*`, `/`, `//`, `%`
7. unary `-`
8. `^` (right-associative)
9. `!`
10. function call `f(x)`
11. `>>` (pipe)
