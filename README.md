# JunoLang

<p align="center">
  <img src="./imgs/logo.png" alt="Juno Logo" width="500"/>
</p>

A modern, statically typed systems programming language built on LLVM.

> **Status:** Early development. Expect breaking changes.

## Features

* LLVM-based compiler
* Written in Rust
* Native module system
* Strong static type system
* Native code generation

## Example

```juno
import ./othermodule.juno;

fn main() -> i32 {
    printf("Hello, World!\n");
    othermodule.say_hello("hi");
    return 0;
}
```

## Building

Clone the repository:

```bash
git clone https://github.com/L0rdCycl0p/JunoLang
cd JunoLang
```

Build the compiler:

```bash
cargo build --release
```

Run:

```bash
cargo run -- examples/hello.juno
```

## Roadmap

* [x] Lexer
* [x] Parser
* [x] AST
* [x] Semantic analysis
* [x] LLVM code generation
* [x] Basic optimizations
* [ ] Structs
* [ ] Enums
* [ ] Generics
* [ ] Trait system
* [ ] Standard library
* [ ] Package manager
* [ ] Language server (LSP)

## Philosophy

Juno aims to be a language that feels simple to read while still providing the power expected from a systems programming language.

The project embraces:

* Explicitness over magic
* Fast compilation
* LLVM as the backend
* Compile-time assets
* Clean and readable syntax

## License

This project is licensed under the MIT License.
