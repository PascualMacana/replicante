# replicante

A small Rust program that copies itself. The binary carries its own source and can write a child Cargo project that, once compiled, can do the same thing.

It is not a language model and it does not spread by itself. You point it at a folder; it only writes there.

```
generation 0  ──spawn──►  generation 1  ──spawn──►  generation 2
lineage    0               lineage    0.1            lineage    0.1.1
```

The child is the same program, one generation later. Lineage counts daughters, not the generation number: the first child of `0` is `0.1`, its first child is `0.1.1`, and a second child of `0` is `0.2`. It does not get smarter; it just inherits the genome and bumps the counter.

![A parent cell budding off a daughter](cell.svg)

Watch it happen in the terminal. The parent stays; a daughter pinches off. That is `spawn`, drawn as a cell.

```bash
cargo run -- dish
```

## Run it

You need [Rust](https://rustup.rs/).

```bash
cargo build --release
./target/release/replicante identity
./target/release/replicante spawn ./hijo --build
./hijo/target/debug/replicante identity
./hijo/target/debug/replicante spawn ./nieto --build
./nieto/target/debug/replicante identity
```

`identity` prints generation, lineage, and the embedded files.  
`spawn ./hijo --build` writes a child crate into `./hijo` and compiles it.

## Commands

```
replicante              help
replicante identity     generation, lineage, genome files
replicante genome       print the embedded sources
replicante dish         animate a cell budding daughters
                 --gens N  how many buds (default 8)
                 --delay MS  ms per frame (default 80)
replicante spawn <dir>  write a child Cargo project
                 --build   compile that child
                 --force   overwrite a previous child
```

## How it works

At compile time, `include_str!` embeds every project file inside the binary:

```rust
const GENOME: &[(&str, &str)] = &[
    ("Cargo.toml", include_str!("../Cargo.toml")),
    ("src/main.rs", include_str!("main.rs")),
    // ...
];
```

`spawn` writes those files to disk and updates two constants in `src/main.rs`:

```rust
const GENERATION: u32 = 0;
const LINEAGE: &str = "0";
```

The child gets `GENERATION = 1` and `LINEAGE = "0.1"`. When you compile it, its own `include_str!` captures that new source. What is copied is the genome, not the executable.

The child still needs `rustc` / `cargo` to become runnable.

## Safety

- One child per run. No background loops, no network.
- It will not write over your home directory, `/`, `/usr`, `/etc`, or the directory you are standing in.
- `--force` only deletes a folder that already looks like a `replicante` project.

## Related

[mejorante](https://github.com/PascualMacana/mejorante) is a sibling that copies itself and also tries to improve.  
[demostrante](https://github.com/PascualMacana/demostrante) is a sibling that only writes a claimed improvement when a checkable proof says so.
