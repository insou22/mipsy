# Mipsy

A MIPS32 emulator, written in Rust.

Mipsy is entirely intended for educational purposes - it is by no means a complete, correct, or rigorous implementation of the MIPS32 specification. It tries to implement *most* common MIPS32 \[psuedo\]instructions, with correct behaviour, however many features are left out in the interests of simplicity, agility of development, and a keen focus on educational value.

Note that mipsy focuses *specifically* on the education of assembly programming, as opposed to the education of hardware, or how hardware functions. It is suited for introductory systems-programming courses, where students can learn a simple assembly language such as MIPS, in a simulator that attempts to provide helpful feedback, powerful debugging tools, and pre-empt common bugs -- all of which aim to give the student a better learning experience.


## Features

Features you will NOT find include:
- Delay slots
- Big-Endian mode
- Kernel mode
- An extensive trap file
- Conditional Branch Likely Instructions
- **Floating point support** (yet -- planned for future)
- ... more to be included here ...

Features you (hopefully) will be pleased to find in mipsy:
- Helpful and explanatory compilation errors
- Helpful and explanatory runtime errors
- Runtime checks - uninitialized memory, registers, etc.
- A powerful and intuitive debugger with readline support
- Time travel debugging
- Wasm in-browser client (a la QtSpim) -- NOTE: *currently experimental*
- ... more to be included here ...

This project is a work-in-progress, but is in a reasonably usable state -- make sure you understand what mipsy does and does not provide before deciding if it is right for you!


## Installation

1. Install the latest stable rust toolchain with `rustup` at https://www.rust-lang.org/tools/install
2. `git clone https://github.com/insou22/mipsy.git && cd mipsy`
3. `cargo build --package mipsy` will build a binary for your machine into `./target/debug/mipsy`
4. Run mipsy using `./target/debug/mipsy [mips_file]`
5. (Optional): Build an optimized release version with `cargo build --release --package mipsy`. Your binary will be in `./target/release/mipsy`
