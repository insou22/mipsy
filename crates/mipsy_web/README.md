<div align="center">

  <h1>Mipsy Web</h1>

  <p>
    <strong>A browser interface for debugging MIPS programs with mipsy as a backend </strong>
  </p>

</div>

# About

The current existing graphical debuggers that exist for MIPS-32 Assembly are currently aging native applications with outadted UI's. 
Mipsy Web intends to leverage the power of the Mipsy Platform to provide a better debugging experience. 

To upgrade the debugging experience from this:

| ![image](https://user-images.githubusercontent.com/27941681/150674852-2cf9a463-a59b-47e0-a3c8-570a66cfed19.png)| 
|:--:| 
| *An outdated QTSpim interface* |

Into this: 
|![image](https://user-images.githubusercontent.com/27941681/150674986-e61d3acf-fed4-4de5-8659-d46713fe0cd1.png)| 
|:--:| 
| *Mipsy Web's interface* |

a development version can be found at https://shreys.web.cse.unsw.edu.au/mipsy/

Try it with some [test files](https://github.com/insou22/mipsy/tree/main/test_files)!

## How is this built?

`mipsy_web` leverages the power of `mipsy_lib` to provide a browser interface for debugging MIPS code. 
It does this by pulling in the rust `mipsy_lib` crate (amongst others) into an application built with the `yew` crate (a React-like rust library for building web frontends), compiling to wasm and running on the browser. 

### The technology

#### WASM
WASM is not a language, but rather a bytecode format (similar to something like Java ByteCode) that is able to be run on a WASM Virtual Machine (similar to the Java Virtual Machine!).

Similar to how most browsers are able to run JavaSript files using a JavaScript Engine (ie v8) - most browsers (as of recently) also ship with a WASM Virtual Machine. WASM Execution is extremely fast as it is often pre-optimised, and not interpereted.

Rust is able to compile to WASM, and run applications on the browser.

#### Rust

[Rust](https://rust-lang.org) is a programming languaged focused around three pillars: performance, reliability and productivity. It is fast, memory-efficient, has an extensive type system and concept of ownership, allowing us to resolve many common errors at compiletime. 

We chose Rust for this problem, as it is a language that enforces aspects of safety and performance. Additionally, since we want to use `mipsy_lib` for the backend, having `mipsy_web` written in rust allows us to easily use it. Furthermore, Rust has crates that allow us to write modern frontend code (see, Yew) and then compile to WASM.

#### Yew
[Yew](yew.rs/) is a Rust framework for bulding web applications.
It behaves very similar to React, with functional components, lifecycle hooks and a virtual DOM.

#### Web Workers
Web workers are a feature of modern browsers that allow us to run code in the background, without blocking the main thread. For `mipsy_web` - we use web workers to compile the code, and to manage the Runtime. 

The frontend of the application continually communicates messages with a worker thread! Messages includes thing such as _"hey, compile this code!"_ or _"can you run one instruction please!"_. 

For example, when you press the *run* button, a message is sent to run 40 instructions - once the worker thread has run 40 instructions, it sends a message back, either everything went okay, or there was an error! If everything went okay, the frontend says okay - run 40 more! This happens until the program exits, or until there is an error in the code. 

#### Mipsy
See [mipsy](https://github.com/insou22/mipsy/blob/main/README.md) for more information on Mipsy. 

#### Tailwind
[Tailwind CSS](https://tailwindcss.com/) is a CSS framework we use to style the application

## Dependancies
1) Install rust via rustup, by following the instructions at [rustup](https://www.rust-lang.org/tools/install)
2) Instal tailwindcss via npm `npm i -g tailwindcss`.

## Development
1) `scripts/build_dev.sh` will compile the relevant rust code to wasm files, and place into the static directory
2) `scripts/serve.sh` will simply run a http client inside `static/` to serve the relevant files.      

`scripts/hot_reload_build.sh` also exists to compile the rust code to wasm files, but will also watch for changes to the `src` directory

## Production
1) `scripts/deploy.sh` will be useful for figuring out how to compile release, build tailwind for release, and then deploy.

## Debug
If you have no CSS - you will need to produce `tailwind.css` file , `./purge_tailwind.sh` should help here. 

tailwind setup with https://dev.to/arctic_hen7/how-to-set-up-tailwind-css-with-yew-and-trunk-il9

