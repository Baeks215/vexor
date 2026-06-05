# Values and Functions

## Values

Bind a value to a name with `val`. Value bindings are top-level declarations — they
live at the top level of a program (or inside a `where` block), not nested inside
expressions:

```vexor
val r = 10
val name = "vexor"
```

## Functions

Define a function with `fn`. The body is a single expression:

```vexor
fn double(x) = x + x
fn area(w, h) = w * h

export Circle(double(5))   -- 10
export Circle(area(2, 3))  -- 6
```

A bound value can also alias an existing function:

```vexor
val also_double = double
```

## Lambdas

Anonymous functions use the `->` arrow:

```vexor
val add = (x, y) -> x + y
val add_one = x -> add(x, 1)
```

## Currying and Partial Application

Functions can be defined with multiple parameter groups, which lets you apply them
one argument at a time:

```vexor
fn scaler(factor)(x) = factor * x

val doubler = scaler(2)   -- partial application

val a = doubler(5)        -- 10
val b = scaler(2)(5)      -- 10
val c = 5 >> scaler(2)    -- 10, using the pipe operator
```

Lambdas can be curried the same way:

```vexor
val scalerL = (factor)(x) -> factor * x
```

## Local Scope with `where`

Attach private helper bindings to a function with a `where` block:

```vexor
fn double_times_factor(x) = times2(x)
where {
  factor = 2
  times2 = (x) -> x * factor
}
```

## Closures

Functions capture values from their surrounding scope:

```vexor
val factor = 3
fn times_factor(x) = x * factor   -- captures `factor`
```

## Lazy Evaluation

Bindings are **lazy** — they are only evaluated when their value is actually used.
Two consequences follow:

- **Order does not matter.** A binding can refer to another that is defined later in
  the file.
- **Unused bindings are never evaluated**, even if they would otherwise be invalid.

```vexor
export Circle(a)

val a = b + 3   -- 10, even though `a` is used before it is defined here
val b = 7

val c = "a" + 1 -- never evaluated, so this invalid expression is harmless
```
