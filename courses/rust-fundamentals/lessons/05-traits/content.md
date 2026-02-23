# Traits & Generics

Traits are Rust's way of defining shared behavior. If you have used interfaces
in Java or abstract classes in C++, traits fill a similar role --- they describe
*what* a type can do without specifying *how* it does it. Combined with
generics, traits let you write flexible code that works across many types while
still being checked at compile time.

## Defining a Trait

A trait is a collection of method signatures (and optionally, default
implementations). You define one with the `trait` keyword:

```rust
trait Describable {
    fn describe(&self) -> String;
}
```

This says: "Any type that is `Describable` must provide a `describe` method
that takes a reference to self and returns a `String`." The trait does not say
*how* to describe --- that is up to each type that implements it.

A trait can require multiple methods:

```rust
trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}
```

## Implementing a Trait

To make a type satisfy a trait, you write an `impl Trait for Type` block:

```rust
struct Dog {
    name: String,
    breed: String,
}

impl Describable for Dog {
    fn describe(&self) -> String {
        format!("{} is a {}", self.name, self.breed)
    }
}
```

Now you can call `.describe()` on any `Dog`:

```rust
let rex = Dog { name: String::from("Rex"), breed: String::from("Labrador") };
println!("{}", rex.describe());
// Rex is a Labrador
```

Multiple types can implement the same trait, each in their own way:

```rust
struct Cat {
    name: String,
    indoor: bool,
}

impl Describable for Cat {
    fn describe(&self) -> String {
        let location = if self.indoor { "indoor" } else { "outdoor" };
        format!("{} is an {} cat", self.name, location)
    }
}
```

## Default Method Implementations

Sometimes a trait can provide a reasonable default for a method. Types that
implement the trait can use the default or override it:

```rust
trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}
```

Here, `summarize` has a default implementation that calls `summarize_author`.
A type only *must* implement `summarize_author`; it gets `summarize` for free:

```rust
struct Article {
    author: String,
    title: String,
}

impl Summary for Article {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }
    // summarize() uses the default: "(Read more from <author>...)"
}
```

If you want different behavior, override the default:

```rust
struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }

    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}
```

## Trait Bounds

Traits become truly powerful when combined with generics. A **trait bound**
constrains a generic type parameter so only types implementing a specific trait
are accepted:

```rust
fn print_description<T: Describable>(item: &T) {
    println!("{}", item.describe());
}
```

This function works with *any* type that implements `Describable` --- `Dog`,
`Cat`, or any future type. If you try to pass a type that does not implement
the trait, the compiler rejects it at compile time.

You can require multiple traits with `+`:

```rust
fn log_item<T: Describable + std::fmt::Display>(item: &T) {
    println!("Description: {}", item.describe());
    println!("Display: {}", item);
}
```

For longer bounds, the `where` clause is cleaner:

```rust
fn process<T>(item: &T)
where
    T: Describable + Clone + std::fmt::Debug,
{
    println!("{:?}", item);
}
```

## Generic Functions

Generics let you write one function that works with many types. Without trait
bounds, you can only do things that work for *all* types (which is very
little). With trait bounds, you unlock the methods those traits provide:

```rust
fn largest<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

fn main() {
    println!("{}", largest(10, 20));       // 20
    println!("{}", largest(3.5, 2.1));     // 3.5
    println!("{}", largest("apple", "banana")); // banana
}
```

`PartialOrd` is a standard library trait that enables comparison with `>=`.
Because `i32`, `f64`, and `&str` all implement `PartialOrd`, the same
`largest` function works for all three.

Generic types can also be used in structs:

```rust
struct Pair<T> {
    first: T,
    second: T,
}

impl<T: std::fmt::Display> Pair<T> {
    fn show(&self) {
        println!("({}, {})", self.first, self.second);
    }
}
```

## The Display Trait

`std::fmt::Display` controls how a type appears when you use `{}` in
`println!`. Rust does not auto-generate Display --- you must implement it
yourself:

```rust
use std::fmt;

struct Point {
    x: f64,
    y: f64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 };
    println!("{}", p);  // (3.0, 4.0)
}
```

The `fmt` method receives a `Formatter` and uses the `write!` macro to produce
output. The return type `fmt::Result` is `Result<(), fmt::Error>` --- return
`Ok(())` on success (which `write!` handles for you).

Implementing `Display` also gives you `.to_string()` for free, since the
standard library provides a blanket implementation of `ToString` for any type
that implements `Display`.

## The Debug Trait

`std::fmt::Debug` controls how a type appears with `{:?}` (debug formatting).
The easiest way to get it is with a derive macro:

```rust
#[derive(Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

fn main() {
    let c = Color { r: 255, g: 128, b: 0 };
    println!("{:?}", c);   // Color { r: 255, g: 128, b: 0 }
    println!("{:#?}", c);  // pretty-printed version
}
```

`#[derive(Debug)]` auto-generates the implementation by printing the struct
name and all fields. This is great for development and logging.

You can also implement Debug manually for custom output:

```rust
use std::fmt;

struct Secret {
    value: String,
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Secret {{ value: [REDACTED] }}")
    }
}
```

This is useful when you want debug output that hides sensitive data or
presents information differently than the default derivation.

## Summary

| Concept            | Key Point                                           |
|--------------------|-----------------------------------------------------|
| Trait definition   | `trait Name { fn method(&self) -> Type; }`          |
| Trait implementation | `impl Trait for Type { fn method(&self) ... }`    |
| Default methods    | Trait provides body; implementors can override       |
| Trait bounds       | `fn foo<T: Trait>(x: &T)` constrains generic params |
| Generic functions  | Work with any type satisfying the bounds             |
| Display            | Implement `fmt::Display` for `{}` formatting         |
| Debug              | `#[derive(Debug)]` or manual impl for `{:?}`        |

Traits and generics are the foundation of Rust's polymorphism. They let you
write code that is both flexible and type-safe, with zero runtime cost ---
the compiler generates specialized code for each concrete type at compile time.
