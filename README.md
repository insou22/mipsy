# RSpim

A MIPS I R2000 emulator, written in Rust.

RSpim is entirely intended for educational purposes - it by no means is a complete, correct, or rigorous implementation of the MIPS specification. It tries to implement *most* common MIPS I \[psuedo\]instructions, with correct behaviour, however many features are left out in the interests of simplicity and agility of development.

Features you will NOT find include:
- Delay slots
- Big-Endian mode
- Kernel mode/segments
- An extensive trap file
- ... more to be included here ...

Features you (hopefully) will be pleased to find in RSpim:
- Helpful and explanatory compilation errors
- Helpful and explanatory runtime errors
- Runtime checks - uninitialized memory, registers, etc.
- A powerful and intuitive debugger
- Time travel debugging
- Wasm in-browser backend
- ... more to be included here ...

This project is a work-in-progress, and is not currently intended to be used for anything other than testing / experimentation.
