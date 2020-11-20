# Mipsy

A MIPS I R2000 emulator, written in Rust.

Mipsy is entirely intended for educational purposes - it by no means is a complete, correct, or rigorous implementation of the MIPS specification. It tries to implement *most* common MIPS I \[psuedo\]instructions, with correct behaviour, however many features are left out in the interests of simplicity, agility of development, and focus on education.

Features you will NOT find include:
- Delay slots
- Big-Endian mode
- Kernel mode
- An extensive trap file
- ... more to be included here ...

Features you (hopefully) will be pleased to find in Mipsy:
- Helpful and explanatory compilation errors
- Helpful and explanatory runtime errors
- Runtime checks - uninitialized memory, registers, etc.
- A powerful and intuitive debugger
- Time travel debugging
- Wasm in-browser backend (todo)
- ... more to be included here ...

This project is a work-in-progress, and is not currently intended to be used for anything other than testing / experimentation.


## Installation

1. Install the latest stable rust with rustup at https://www.rust-lang.org/tools/install
2. `git clone https://github.com/insou22/mipsy.git && cd mipsy`
3. `./extras/build.sh` will build a binary for your machine into `./target/debug/mipsy`
4. Run Mipsy using `./target/debug/mipsy {mips_file}`
5. (Optional): Build an optimized release version with `./extras/build.sh --release`, your binary will be in `./target/release/mipsy`