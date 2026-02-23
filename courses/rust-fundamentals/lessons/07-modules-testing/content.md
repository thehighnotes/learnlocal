# Modules & Testing

As programs grow, dumping everything into one flat file becomes hard to navigate. You end up with dozens of functions, types, and constants all in the same namespace, and names start to collide.

Rust's **module system** lets you organize code into named groups. Each module creates its own namespace, so `math::add` and `string::add` can coexist without conflict.

In real Rust projects (using Cargo), modules often map to separate files. But the fundamental mechanism is the same: the `mod` keyword defines a module, and everything inside it lives in that module's namespace.

## Defining an Inline Module

The simplest way to create a module is inline -- right in the same file:

```rust
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn subtract(a: i32, b: i32) -> i32 {
        a - b
    }
}

fn main() {
    let sum = math::add(3, 4);
    println!("{}", sum); // 7
}
```

`mod math { ... }` creates a module called `math`. To call functions inside it from outside, you use the **path syntax**: `math::add(3, 4)`.

Notice the `pub` keyword on the functions. Without it, they would be private to the module and inaccessible from `main`.

## Visibility: pub vs Private

By default, everything inside a module is **private**. This is the opposite of many languages where things are public unless you say otherwise.

```rust
mod secrets {
    fn hidden() -> &'static str {
        "you can't see me"
    }

    pub fn visible() -> &'static str {
        hidden() // private items are accessible WITHIN the module
    }
}

fn main() {
    // secrets::hidden();  // ERROR: function `hidden` is private
    println!("{}", secrets::visible()); // OK
}
```

Private items are still accessible to other code *inside the same module*. The `visible` function can call `hidden` because they live in the same module. But code outside (like `main`) can only access `pub` items.

This applies to more than just functions:

- **Functions**: private by default, add `pub` to expose
- **Structs**: the struct itself can be `pub`, but each field is independently private by default
- **Struct fields**: must be individually marked `pub` if you want outside access
- **Constants and statics**: private by default

```rust
mod shapes {
    pub struct Rectangle {
        pub width: f64,
        pub height: f64,
    }

    impl Rectangle {
        pub fn area(&self) -> f64 {
            self.width * self.height
        }
    }
}

fn main() {
    let r = shapes::Rectangle { width: 3.0, height: 4.0 };
    println!("Area: {}", r.area()); // 12
}
```

If `width` or `height` were not marked `pub`, you could not construct the struct from outside the module. You would need a constructor function instead.

## The use Keyword

Typing the full path every time gets tedious. The `use` keyword brings an item into the current scope:

```rust
mod math {
    pub fn add(a: i32, b: i32) -> i32 { a + b }
    pub fn multiply(a: i32, b: i32) -> i32 { a * b }
}

use math::add;
use math::multiply;

fn main() {
    println!("{}", add(2, 3));      // no need for math::add
    println!("{}", multiply(4, 5)); // no need for math::multiply
}
```

You can also bring multiple items from the same module with braces:

```rust
use math::{add, multiply};
```

Or bring everything with a wildcard (called a **glob import**):

```rust
use math::*;
```

Glob imports are convenient but can make it hard to tell where a name comes from. In practice, most Rust code prefers explicit imports.

You can also rename imports with `as`:

```rust
use math::add as sum;

fn main() {
    println!("{}", sum(1, 2)); // calls math::add
}
```

## Nested Modules

Modules can contain other modules, creating a tree structure:

```rust
mod animal {
    pub mod dog {
        pub fn speak() -> &'static str {
            "Woof!"
        }
    }

    pub mod cat {
        pub fn speak() -> &'static str {
            "Meow!"
        }
    }
}

fn main() {
    println!("{}", animal::dog::speak());
    println!("{}", animal::cat::speak());
}
```

Each level of nesting adds another `::` to the path. The `pub` keyword is needed on the inner modules too -- a `pub` function inside a private module is still unreachable from outside.

You can `use` nested paths:

```rust
use animal::dog::speak as dog_speak;
use animal::cat::speak as cat_speak;
```

### Modules in Real Rust Projects

When using Cargo, modules are typically split across files:

```
src/
  main.rs       // mod math;  (declares the module)
  math.rs       // contains the math module's code
  animal/
    mod.rs      // pub mod dog; pub mod cat;
    dog.rs      // pub fn speak() { ... }
    cat.rs      // pub fn speak() { ... }
```

`mod math;` (with a semicolon, no braces) tells Rust to look for the module's code in a separate file. This is how large projects stay organized. But the visibility rules, `use` imports, and path syntax are exactly the same whether modules are inline or in separate files.

In single-file programs compiled with `rustc`, inline modules are the way to go.

## Testing in Rust

Rust has first-class support for testing built right into the language. The conventional approach uses two attributes:

- `#[cfg(test)]` on a module -- tells the compiler to only include this module when running tests
- `#[test]` on a function -- marks it as a test case

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}
```

`use super::*;` imports everything from the parent module (the file's top level) so the tests can access the `add` function.

### Assert Macros

Rust provides three assertion macros for testing:

**assert!** checks that a condition is true. Panics if false:

```rust
let x = 5;
assert!(x > 0);        // passes
assert!(x > 10);       // panics: assertion failed
```

**assert_eq!** checks that two values are equal. Panics if they differ, showing both values in the error message:

```rust
assert_eq!(2 + 2, 4);  // passes
assert_eq!(2 + 2, 5);  // panics: left: 4, right: 5
```

**assert_ne!** checks that two values are NOT equal. Panics if they are the same:

```rust
assert_ne!(2 + 2, 5);  // passes
assert_ne!(2 + 2, 4);  // panics: both are 4
```

All three macros accept an optional custom message as additional arguments:

```rust
assert_eq!(result, 42, "Expected 42 but got {}", result);
```

These macros are not limited to test functions -- you can use them anywhere to verify invariants. If the assertion fails, the program panics with a descriptive error message.

Note: When compiling single files with `rustc`, the test attributes are not
activated -- you would need `rustc --test` to compile in test mode. In these
exercises, we demonstrate testing concepts by running assertions directly
in `main()`, and you will also run a Cargo project's tests with `cargo test`.

## Running Tests with Cargo

`cargo test` is the standard way to run tests. It compiles the `#[cfg(test)]`
module and runs every `#[test]` function:

```bash
cargo test
```

Output looks like:

```
running 2 tests
test tests::test_add ... ok
test tests::test_add_negative ... ok

test result: ok. 2 passed; 0 failed; 0 ignored
```

Each test either passes (returns normally) or fails (panics). Cargo discovers
test functions automatically -- you do not need to register them anywhere.
