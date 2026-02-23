# Pattern Matching

Pattern matching is one of Rust's most distinctive features. Where other languages
use chains of if/else or switch statements, Rust gives you `match` -- a construct
that compares a value against patterns and runs the code for the first one that fits.
The key difference from a C-style switch: the compiler guarantees you handle every
possible case. Forget one, and your code will not compile.

## The match Expression

A `match` expression takes a value and compares it against patterns, top to bottom:

```rust
let coin = "quarter";

match coin {
    "penny"   => println!("1 cent"),
    "nickel"  => println!("5 cents"),
    "dime"    => println!("10 cents"),
    "quarter" => println!("25 cents"),
    _         => println!("unknown coin"),
}
```

Each line inside the braces is a **match arm**: a pattern, a fat arrow `=>`, and
an expression to evaluate. The underscore `_` is the wildcard pattern -- it matches
anything and is typically used as the final catch-all.

`match` is an expression, so it can return a value:

```rust
let cents = match coin {
    "penny"   => 1,
    "nickel"  => 5,
    "dime"    => 10,
    "quarter" => 25,
    _         => 0,
};
```

When used as an expression, every arm must return the same type.

## Exhaustive Matching

The compiler insists that every possible value is covered. For integers, that
means a wildcard `_` is usually required as a catch-all:

```rust
let n: i32 = 42;

match n {
    1 => println!("one"),
    2 => println!("two"),
    _ => println!("something else"),
}
```

Remove the `_` arm and the compiler refuses to build. This exhaustiveness check
is one of Rust's strongest safety nets. When you add a new variant to an enum,
the compiler tells you every match that needs updating.

## Matching Enums

Enums are where match truly shines. Each variant becomes a pattern, and if the
variant carries data, the pattern can extract it:

```rust
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

let s = Shape::Circle(5.0);

match s {
    Shape::Circle(r) => println!("Circle with radius {}", r),
    Shape::Rectangle(w, h) => println!("Rectangle {}x{}", w, h),
}
```

The variables `r`, `w`, `h` are **bindings** -- they capture the data inside the
variant. Because both variants are listed, no wildcard is needed.

The standard library's `Option<T>` is just an enum with two variants:

```rust
let maybe: Option<i32> = Some(42);

match maybe {
    Some(x) => println!("Got {}", x),
    None    => println!("Nothing"),
}
```

## if let: Single-Pattern Shorthand

When you only care about one pattern and want to ignore the rest, a full `match`
is verbose. `if let` handles this neatly:

```rust
let config_value: Option<i32> = Some(100);

if let Some(val) = config_value {
    println!("Config is set to {}", val);
}
```

You can add an `else` block for the non-matching case:

```rust
if let Some(val) = config_value {
    println!("Found: {}", val);
} else {
    println!("Not set");
}
```

`if let` does not require exhaustiveness. Use `match` when you need to handle
every case explicitly.

## while let: Loop on a Pattern

`while let` repeats a block as long as a pattern matches. The classic use case is
draining a `Vec` with `pop()`, which returns `Option<T>`:

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

This prints `3`, `2`, `1`. The loop runs until `pop()` returns `None`, at which
point the pattern fails and the loop exits.

## Destructuring Structs and Tuples

Patterns can pull apart compound types. For a struct:

```rust
struct Point {
    x: i32,
    y: i32,
}

let p = Point { x: 10, y: 20 };
let Point { x, y } = p;
println!("x={}, y={}", x, y);
```

The `let Point { x, y } = p;` line creates two variables from the struct's
fields. Tuples work the same way:

```rust
let rgb = (255, 128, 0);
let (r, g, b) = rgb;
println!("Red={}, Green={}, Blue={}", r, g, b);
```

Destructuring works inside `match` arms too:

```rust
match p {
    Point { x: 0, y: 0 } => println!("At the origin"),
    Point { x, y: 0 }    => println!("On the x-axis at {}", x),
    Point { x: 0, y }    => println!("On the y-axis at {}", y),
    Point { x, y }       => println!("At ({}, {})", x, y),
}
```

Literal values in patterns match exactly. Bare names match anything and bind.

## Match Guards

A match guard adds an `if` condition after a pattern. The arm only matches if
both the pattern and the guard are satisfied:

```rust
let n = 4;

match n {
    x if x < 0  => println!("{} is negative", x),
    x if x == 0 => println!("zero"),
    x if x % 2 == 0 => println!("{} is positive and even", x),
    x => println!("{} is positive and odd", x),
}
```

Guards are useful when the logic cannot be expressed by the pattern alone. The
final arm `x` (without a guard) acts as the catch-all. The variable `x` is
available in both the guard and the arm's body.

## @ Bindings

Sometimes you want to test a value against a pattern **and** capture it in a
variable at the same time. The `@` operator does this:

```rust
let n = 7;

match n {
    val @ 1..=10 => println!("{} is between 1 and 10", val),
    val @ 11..=20 => println!("{} is between 11 and 20", val),
    other => println!("{} is out of range", other),
}
```

Without `@`, the range `1..=10` matches but gives you no name for the value.
With `val @ 1..=10`, you get both the range check and a binding called `val`.

Without `@`, you could use a match guard (`x if x >= 1 && x <= 10`), but the
`@` syntax is more concise when working with range patterns.

## Choosing the Right Tool

- Use `match` when you have multiple cases to handle, especially with enums.
- Use `if let` when you care about exactly one pattern and want to ignore the rest.
- Use `while let` to loop until a pattern stops matching.
- Use destructuring to pull apart structs and tuples into named variables.
- Use match guards when the pattern syntax alone cannot express the condition.
- Use `@` bindings when you need to both test and name a value.
