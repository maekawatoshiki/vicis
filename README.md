# Vicis

[![CI](https://github.com/maekawatoshiki/vicis/workflows/Rust/badge.svg)](https://github.com/maekawatoshiki/vicis/actions/workflows/rust.yml)
[![codecov](https://codecov.io/gh/maekawatoshiki/vicis/branch/master/graph/badge.svg)](https://codecov.io/gh/maekawatoshiki/vicis)
[![](http://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Manipulate LLVM IR in pure Rust (without LLVM).

Feel free to create issues and pull requests!

# Requirements

- ~~llvm (== 12.0.0) is used for tests~~
  - You don't need it anymore!

# Examples

- Parse and dump `*.ll` file

```sh
cargo run --example parse FILE.ll
```

- Interpret `*.ll` file

```
cargo run --example interpreter FILE.ll # --release
```

- [Iterate over instructions](./core/examples/iterate.rs)

- [Compile LLVM IR into machine code](./codegen/examples/example_x86_64.rs)
  - The example illustrates the way for x86_64, but it's easy to do the same thing for aarch64 (although aarch64 backend is still heavily under development.)

