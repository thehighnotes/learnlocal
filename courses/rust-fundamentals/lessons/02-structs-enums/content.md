# Structs & Enums

So far you have worked with primitive types like integers, floats, and booleans.
Real programs need **custom types** that group related data together. Rust gives
you two powerful tools for this: **structs** and **enums**.

## Defining a Struct

A struct groups named fields into a single type. By convention, struct names use
**PascalCase** and field names use **snake_case**:

```rust
struct Point {
    x: f64,
    y: f64,
}
```

Create an instance by providing values for every field:

```rust
let p = Point { x: 3.0, y: 4.0 };
println!("x = {}, y = {}", p.x, p.y);
```

Fields are accessed with the dot operator, just like in most languages. Unlike
C++, there is no semicolon after the closing brace of the struct definition --
the comma after the last field is optional but conventional.

## Mutability

In Rust, the **entire** variable is mutable or immutable. You cannot make just
one field mutable:

```rust
let mut p = Point { x: 1.0, y: 2.0 };
p.x = 10.0;  // OK -- p is declared mut
```

Without `mut`, any attempt to modify a field is a compile error.

## impl Blocks -- Adding Behavior

A bare struct is just data. To add behavior, you write an **impl block**:

```rust
impl Point {
    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

The `&self` parameter means "a shared reference to the instance this method is
called on." You call it with dot syntax:

```rust
let p = Point { x: 3.0, y: 4.0 };
println!("{}", p.distance_from_origin());  // 5
```

## Method Receivers: &self, &mut self, self

The first parameter of a method determines how it accesses the instance:

| Receiver     | Meaning                                     |
|--------------|---------------------------------------------|
| `&self`      | Borrows the instance immutably (read-only)  |
| `&mut self`  | Borrows the instance mutably (can modify)   |
| `self`       | Takes ownership (consumes the instance)     |

Most methods use `&self`. Use `&mut self` when the method needs to change the
struct's data:

```rust
impl Point {
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

let mut p = Point { x: 1.0, y: 2.0 };
p.translate(3.0, 4.0);
println!("({}, {})", p.x, p.y);  // (4, 6)
```

## Associated Functions (No self)

Functions inside an `impl` block that do **not** take `self` are called
**associated functions**. They are called with `::` syntax, not dot syntax:

```rust
impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn origin() -> Self {
        Point { x: 0.0, y: 0.0 }
    }
}

let p = Point::new(3.0, 4.0);
let o = Point::origin();
```

`Self` (capital S) is an alias for the type the impl block is for -- here,
`Point`. Associated functions are commonly used as constructors. `Point::new()`
is the idiomatic Rust constructor pattern.

## Enums -- One of Several Variants

An **enum** defines a type that can be exactly one of several variants:

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

let d = Direction::North;
```

Variants are namespaced under the enum name: `Direction::North`, not just
`North`.

## Enums with Data

Rust enums are far more powerful than enums in most languages. Each variant can
hold data:

```rust
enum Shape {
    Circle(f64),              // radius
    Rectangle(f64, f64),      // width, height
}

let s = Shape::Circle(5.0);
```

Different variants can hold different types and amounts of data. This is
sometimes called a "tagged union" or "algebraic data type."

## Methods on Enums

Enums can have impl blocks too:

```rust
impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
        }
    }
}
```

The `match` expression is how you inspect which variant an enum value holds.
Every variant must be handled -- the compiler enforces exhaustive matching. We
will cover `match` in depth in the next lesson.

## Option<T> -- Rust's Null Replacement

Rust has no `null`. Instead, the standard library provides `Option<T>`:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`Option<T>` is so common that `Some` and `None` are available without
qualification -- you do not need to write `Option::Some`.

Use `Option` when a value might be absent:

```rust
fn find_first_negative(numbers: &[i32]) -> Option<i32> {
    for &n in numbers {
        if n < 0 {
            return Some(n);
        }
    }
    None
}
```

To extract the value, you can use:

- `unwrap()` -- panics if None (use only when you are certain it is Some)
- `unwrap_or(default)` -- returns default if None
- `match` or `if let` -- safe, pattern-based extraction

```rust
let x: Option<i32> = Some(42);
println!("{}", x.unwrap());           // 42
println!("{}", x.unwrap_or(0));       // 42

let y: Option<i32> = None;
println!("{}", y.unwrap_or(0));       // 0
```

## Derive Macros -- Free Trait Implementations

Rust can auto-generate common trait implementations using **derive macros**.
Place them above a struct or enum definition:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}
```

| Derive        | What it gives you                                |
|---------------|--------------------------------------------------|
| `Debug`       | `{:?}` formatting for println!                  |
| `Clone`       | `.clone()` method to create a deep copy          |
| `PartialEq`   | `==` and `!=` comparison operators              |
| `Copy`        | Implicit copy on assignment (requires Clone)     |

`Debug` is the most commonly derived trait. It lets you inspect any value:

```rust
let p = Point { x: 1.0, y: 2.0 };
println!("{:?}", p);       // Point { x: 1.0, y: 2.0 }
println!("{:#?}", p);      // pretty-printed multi-line version
```

Without `#[derive(Debug)]`, trying to print a struct with `{:?}` is a compile
error.

## Putting It All Together

Here is a complete example combining structs, enums, impl blocks, and derive:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    fn is_grayscale(&self) -> bool {
        self.r == self.g && self.g == self.b
    }
}

fn main() {
    let c1 = Color::new(128, 128, 128);
    let c2 = c1.clone();
    println!("{:?}", c1);                    // Color { r: 128, g: 128, b: 128 }
    println!("grayscale? {}", c1.is_grayscale());  // true
    println!("equal? {}", c1 == c2);         // true
}
```

Custom types are the backbone of Rust programs. Structs hold your data, impl
blocks define behavior, enums model choices, and derive macros save you from
writing boilerplate. In the next lesson, you will learn **pattern matching** --
the primary way to work with enums and destructure data in Rust.
