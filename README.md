# Vexor

A small functional programming language for creating vector graphics. A Vexor
program describes shapes, and the compiler renders them to **SVG**.

> Created as part of a final year BEng project for the Computing course at Imperial
> College London.

```vexor
export Circle(50)
  >> fill(rgb(255, 100, 0))
  >> move(100, 100)
```

- Functional and expression-oriented; first-class, curried functions with closures.
- `if`/`else` and `match` pattern matching for control flow.
- Lazy evaluation.
- Graphics built and combined with plain functions (`move`, `scale`, `fill`, …) and
  a pipe operator (`>>`).

## Documentation

Full language guide and standard library reference:
**<https://baeks215.github.io/vexor/>**

## Installation

### Prebuilt binaries

Download a binary for your platform from the
[releases page](https://github.com/Baeks215/vexor/releases).

### From source

```sh
# Install the `vexor` binary onto your PATH
cargo install --path crates/vexor-cli

# ...or run it directly without installing
cargo run -- compile input.vx output.svg
```

## Usage

Vexor source lives in `.vx` files.

| Command                          | Description                         |
| -------------------------------- | ----------------------------------- |
| `vexor compile <input> <output>` | Compile a `.vx` file to SVG.        |
| `vexor watch <input> <output>`   | Recompile automatically on changes. |
| `vexor gui <input>`              | Open a live preview window.         |

```sh
vexor compile hello.vx hello.svg
```

A program needs at least one `export`. One export writes a single SVG file; multiple
exports write numbered SVGs into the output directory.

## License

Apache-2.0
