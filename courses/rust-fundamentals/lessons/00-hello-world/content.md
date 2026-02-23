# Hello World & Basics

Rust is a systems programming language that gives you fine-grained control over
memory and performance while catching entire categories of bugs at compile time.
This lesson covers the foundation: printing output, declaring variables, and
working with the basic types.

## Your First Program

Here is the simplest Rust program:

```rust
fn main() {
    println!("Hello, World!");
}
```

Two lines of actual code. Run it and you see `Hello, World!` in the terminal.
Rust has no header files to include and no return value to worry about in `main`.

### The main Function

Every Rust program must have a function named `main`. This is the **entry point**
-- the first function that runs when you execute the program.

```rust
fn main() {
    // your code goes here
}
```

- `fn` is the keyword that declares a function.
- `main` is the function name. Rust requires exactly one `main` function.
- `()` is an empty parameter list.
- The curly braces `{ }` contain the function body.

Unlike C or C++, `main` does not return an integer by default. It returns `()`
(the unit type, similar to `void`).

## Printing with println!

`println!` is a **macro** that prints text followed by a newline. The `!` tells
you it is a macro, not a regular function.

```rust
println!("Hello, World!");
```

There is also `print!` which prints without a trailing newline:

```rust
print!("No newline after this");
println!(" but this line ends with one");
```

### Format Placeholders

The real power of `println!` comes from `{}` placeholders:

```rust
let name = "Alice";
let age = 30;
println!("{} is {} years old", name, age);
```

Output: `Alice is 30 years old`

Each `{}` is replaced by the next argument, in order. You can also use positional
indices:

```rust
println!("{0} likes {1}. {0} is learning {1}.", "Alice", "Rust");
```

Output: `Alice likes Rust. Alice is learning Rust.`

For now, the basic `{}` style is all you need.

## Variables with let

Variables in Rust are declared with `let`:

```rust
let x = 5;
let greeting = "hello";
```

By default, variables are **immutable**. Once you assign a value, you cannot
change it:

```rust
let x = 5;
x = 10;  // ERROR: cannot assign twice to immutable variable
```

If you want a variable that can change, use `let mut`:

```rust
let mut count = 0;
count = 1;   // OK — count is mutable
count = 2;   // OK — can change again
println!("count is {}", count);
```

Immutability by default encourages you to think about which values truly need to
change. The compiler will also warn you if you declare `mut` but never mutate.

## Basic Types

Rust is statically typed -- every value has a type known at compile time.

**Integers:** `i8`, `i16`, `i32`, `i64` (signed) and `u8`, `u16`, `u32`, `u64`
(unsigned). `i32` is the default -- when you write `let x = 42;`, Rust infers
`i32`.

**Floats:** `f32` (32-bit) and `f64` (64-bit). `f64` is the default -- when you
write `let pi = 3.14;`, Rust infers `f64`.

```rust
let temperature: f64 = 98.6;
let ratio = 0.75;  // inferred as f64
```

**Booleans:** Only two values, `true` and `false`:

```rust
let is_active: bool = true;
let done = false;  // inferred as bool
```

**Characters:** A `char` is a Unicode scalar value. Uses single quotes:

```rust
let letter: char = 'A';
let crab: char = '🦀';
```

**String types:** Rust has two main string types:

- `&str` (string slice) -- a reference to string data. String literals like
  `"hello"` are `&str`.
- `String` -- an owned, growable string on the heap.

```rust
let greeting: &str = "hello";
let name: String = String::from("Alice");
```

For this lesson, string literals (`&str`) are all you need. The distinction
becomes important when you learn about ownership in the next lesson.

## Shadowing

Rust lets you re-declare a variable with the same name using a new `let`
statement. This is called **shadowing**:

```rust
let x = 5;
let x = x + 1;
let x = x * 2;
println!("x is {}", x);  // prints: x is 12
```

Each `let x` creates a new variable that shadows the previous one. Shadowing
is different from `mut` -- you can even change the type:

```rust
let spaces = "   ";         // &str
let spaces = spaces.len();  // now it's usize (an integer)
```

This would not work with `mut` because you cannot change a variable's type.

## Type Inference

Rust can figure out the type of most variables from context:

```rust
let x = 42;          // Rust infers i32
let pi = 3.14;       // Rust infers f64
let active = true;   // Rust infers bool
let name = "Alice";  // Rust infers &str
```

You **can** annotate types explicitly when you want to be clear or when Rust
cannot infer:

```rust
let x: i64 = 42;             // explicit: use i64 instead of i32
let temperature: f32 = 98.6; // explicit: use f32 instead of f64
```

Type annotations use a colon: `let name: Type = value;`. When the compiler gives
a "type annotations needed" error, that is your cue to add one.

### Comments

Comments are notes for humans. The compiler ignores them completely.

**Single-line comments** start with `//`:

```rust
// This is a comment
let x = 5;  // This is also a comment
```

**Multi-line comments** are wrapped in `/* */`:

```rust
/* This comment
   spans multiple
   lines */
```

Rust also has **doc comments** (`///` and `//!`) for generating documentation,
but those come later. For now, `//` is all you need.

Use comments to explain **why** you did something, not **what** the code does.

## Checking Your Toolchain

Rust has two key tools: the compiler (`rustc`) and the build system (`cargo`).
Both are installed together via `rustup`. You can check they are available:

```bash
rustc --version
cargo --version
```

This prints version information for each tool. Knowing which version you have
tells you which Rust features and editions are available.

## Using Cargo

For real projects, you use **Cargo** -- Rust's build system and package manager.
Cargo handles compilation, dependencies, and project structure:

```bash
cargo new greeting     # creates a project directory with src/main.rs
cd greeting
cargo run              # compiles and runs in one step
```

`cargo new` generates a standard project layout:

```
greeting/
  Cargo.toml     # project metadata and dependencies
  src/
    main.rs      # your code (starts with a Hello World)
```

`cargo run` compiles and executes in one step. You will use `rustc` for quick
single-file exercises and `cargo` for projects with multiple files or
dependencies.

## Compiling with rustc

For single-file programs, `rustc` compiles directly:

```bash
rustc -o hello hello.rs
./hello
```

- `-o hello` names the output executable
- `hello.rs` is your source file

If there are errors, the compiler prints detailed messages with suggestions.
Rust's compiler errors are famously helpful -- read them carefully.
