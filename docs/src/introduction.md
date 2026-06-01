# Introduction

> **Note:** This documentation was generated with the help of AI (Claude).

**Vexor** is a small functional programming language for creating vector graphics.
A Vexor program is an expression — or a set of expressions — that describes shapes,
and the compiler renders them to **SVG**.

```vexor
set canvas(200, 200)

export Circle(50)
  >> fill(rgb(255, 100, 0))
  >> move(100, 100)
```

The language is functional and expression-oriented:

- Numbers are 64-bit floats; there are also booleans, strings, lists, tuples,
  colors, and graphics.
- Functions are first-class, curried, and support closures.
- Control flow is done with `if`/`else` expressions and `match` pattern matching.
- Graphics are built and combined with plain functions (`move`, `scale`, `fill`, …)
  and a pipe operator (`>>`).
- Evaluation is **lazy**: bindings are only evaluated when their value is actually
  needed.

Programs are written in `.vx` files and compiled with the `vexor` CLI. Head to
[Getting Started](./getting-started.md) to install the tool and render your first
shape.
