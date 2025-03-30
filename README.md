[![Actions][actions-badge]][actions-url]
[![crates.io version][crates-timeflake-badge]][crates-timeflake]

[actions-badge]: https://github.com/kkent030315/timeflake/workflows/CI/badge.svg?branch=master
[actions-url]: https://github.com/kkent030315/timeflake/actions
[crates-timeflake-badge]: https://img.shields.io/crates/v/timeflake.svg
[crates-timeflake]: https://crates.io/crates/timeflake

# timeflake-rs

Rust port of Timeflake, a 128-bit, roughly-ordered, URL-safe UUID.

Original work can be found at [anthonynsimon/timeflake](https://github.com/anthonynsimon/timeflake).

## Usage

```toml
[dependencies]
timeflake = "0.1.0"
```

or

```toml
cargo add timeflake
```

## Example

```rust
use timeflake::Timeflake;

fn main() {
    let mut rng = rand::rng();
    let flake = Timeflake::new_random(&mut rng);
    println!("{flake}");
}
```

## Features

- `std`: Allow `no_std` environments. This is on by default.
- `uuid`: Allow use of `uuid` crate. This is on by default.

## Benchmark

Run benchmarks by follow command. It is recommended to have gnuplot installed.

```bash
cargo bench
```
