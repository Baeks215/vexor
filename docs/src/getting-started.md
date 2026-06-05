# Getting Started

## Installation

### Prebuilt binaries

Download a prebuilt `vexor` binary for your platform from the
[releases page](https://github.com/Baeks215/vexor/releases).

### From source

Vexor is a Rust workspace. Build the CLI from source:

```sh
# Install the `vexor` binary onto your PATH
cargo install --path crates/vexor-cli

# ...or run it directly without installing
cargo run -- compile input.vx output.svg
```

## Your First Program

Vexor source lives in files with the `.vx` extension. Create `hello.vx`:

```vexor
export Circle(10)
```

Every program must have at least one `export`. Compile it to SVG:

```sh
vexor compile hello.vx hello.svg
```

## The CLI

| Command | Description |
|---------|-------------|
| `vexor compile <input> <output>` | Compile a `.vx` file to SVG. |
| `vexor watch <input> <output>` | Recompile automatically when the source changes. |
| `vexor gui <input>` | Open a live preview window. |

## The Canvas

Use `set canvas(width, height)` to size the output. Place it at the top of the file:

```vexor
set canvas(400, 300)

export Rect(100, 50)
```

The **origin `(0, 0)` is at the center of the canvas** — positive `x` goes right and
positive `y` goes down. A shape built with no transform sits at the center.

## Precision

Use `set precision(n)` to control how many decimal places coordinates use in the
generated SVG. Every float is rounded to `n` places and trailing zeros are
dropped, so `100.000` becomes `100` and `0.35355…` becomes `0.354`.

**The default precision is `3`** — if you never write a `set precision` statement,
every coordinate is rounded to 3 decimal places.

```vexor
set precision(2)

export Circle(100)
```

Lower precision produces smaller files; higher precision preserves fine detail.
Use `set precision(0)` to round every coordinate to a whole number.

## Output

- **One export** → the output path is written as a single SVG file.
- **Multiple exports** → the output path is treated as a directory, and each export
  is written as a numbered SVG file inside it.

```vexor
export Circle(10)
export Circle(20)
```

```sh
vexor compile shapes.vx out/   # writes out/ with one SVG per export
```

Use `export each <list>` to export every graphic in a list:

```vexor
val circles = [1..5] >> map(Circle)

export each circles
```
