# Ownership & Borrowing

Ownership is Rust's most distinctive feature. It is the mechanism that lets Rust
guarantee memory safety at compile time, with no garbage collector, no runtime
cost, and no manual malloc/free. Every Rust programmer must understand ownership
because the compiler will not let you ignore it.

This lesson covers the ownership rules, move semantics, cloning, references,
mutable references, borrowing rules, string slices, and a brief introduction
to lifetimes.

## The Three Ownership Rules

Rust's ownership system is built on three rules. They are simple to state and
the compiler enforces them absolutely:

1. Each value in Rust has exactly one variable that is its **owner**.
2. There can only be one owner at a time.
3. When the owner goes out of scope, the value is **dropped** (its memory is freed).

```rust
fn main() {
    let s = String::from("hello");  // s owns the String
    println!("{}", s);
}  // s goes out of scope — the String's memory is freed here
```

There is no garbage collector running in the background. There is no `free()`
call you might forget. The compiler inserts the cleanup code for you, exactly
where the owner's scope ends.

## Move Semantics

When you assign a heap-allocated value to another variable, Rust **moves**
ownership. The original variable is invalidated:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // ownership moves from s1 to s2

    // println!("{}", s1);  // ERROR: s1 is no longer valid
    println!("{}", s2);     // OK: s2 owns the String now
}
```

This is different from languages like Python or Java, where both variables
would point to the same data. Rust's move semantics prevent double-free bugs:
if both `s1` and `s2` tried to free the same memory, your program would crash.

Moves happen in three common situations:

- **Assignment:** `let s2 = s1;`
- **Passing to a function:** `takes_ownership(s1);`
- **Returning from a function:** `let s2 = gives_ownership();`

```rust
fn takes_ownership(s: String) {
    println!("{}", s);
}  // s is dropped here

fn main() {
    let s = String::from("hello");
    takes_ownership(s);
    // s is no longer valid here — it was moved into the function
}
```

### Copy Types

Not everything moves. Simple types that live entirely on the stack implement the
`Copy` trait and are **copied** instead of moved:

```rust
fn main() {
    let x = 5;
    let y = x;  // x is copied, not moved
    println!("x = {}, y = {}", x, y);  // both are valid
}
```

Types that are `Copy` include: integers, floats, booleans, characters, and
tuples that contain only `Copy` types. Anything that allocates heap memory
(like `String` or `Vec`) does **not** implement `Copy`.

## Clone: Explicit Deep Copy

When you want to keep the original and also have a second copy, use `clone()`:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // deep copy — both are valid

    println!("s1 = {}, s2 = {}", s1, s2);
}
```

Calling `clone()` is an explicit signal that you know this operation might be
expensive. It allocates new heap memory and copies all the data. The compiler
never calls `clone()` implicitly — you must opt in.

## References and Borrowing

What if you want to let a function read a value without taking ownership?
Use a **reference**. A reference borrows the value without moving it:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope, but since it does not own the String, nothing is dropped

fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);  // pass a reference
    println!("The length of '{}' is {}.", s1, len);  // s1 is still valid
}
```

The `&` symbol creates a reference. The function signature `s: &String` says
"I am borrowing a String, not taking ownership." This is called **borrowing**.

References are immutable by default. You cannot modify borrowed data:

```rust
fn try_to_change(s: &String) {
    // s.push_str(" world");  // ERROR: cannot borrow as mutable
}
```

## Mutable References

If you need to modify borrowed data, use a **mutable reference** with `&mut`:

```rust
fn add_world(s: &mut String) {
    s.push_str(", world!");
}

fn main() {
    let mut s = String::from("hello");
    add_world(&mut s);
    println!("{}", s);  // prints: hello, world!
}
```

Three things must align for a mutable borrow to work:

1. The variable must be declared `let mut`
2. The reference must be `&mut`
3. The function parameter must accept `&mut`

## The Borrowing Rules

Rust enforces two rules about references at compile time:

1. You can have **either** one mutable reference **or** any number of immutable
   references to a value — but not both at the same time.
2. References must always be valid (no dangling references).

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;      // OK: first immutable reference
    let r2 = &s;      // OK: second immutable reference
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point

    let r3 = &mut s;  // OK: mutable reference (no immutable refs active)
    println!("{}", r3);
}
```

This would fail:

```rust
fn main() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &mut s;  // ERROR: cannot borrow as mutable while immutable ref exists
    println!("{}", r1);
}
```

These rules prevent **data races** at compile time. A data race happens when
two pointers access the same data simultaneously and at least one is writing.
Rust makes it structurally impossible.

## String Slices

A **string slice** is a reference to a portion of a `String`. Its type is `&str`:

```rust
fn main() {
    let s = String::from("hello world");

    let hello = &s[0..5];   // "hello"
    let world = &s[6..11];  // "world"

    println!("{} {}", hello, world);
}
```

Slice syntax uses `[start..end]` where `start` is inclusive and `end` is
exclusive. You can omit the start (defaults to 0) or the end (defaults to
length):

```rust
let s = String::from("hello");
let slice1 = &s[..3];   // "hel" — same as &s[0..3]
let slice2 = &s[2..];   // "llo" — same as &s[2..5]
let slice3 = &s[..];    // "hello" — the entire string
```

String slices are references, so they follow the borrowing rules. You cannot
modify a `String` while a slice to it exists:

```rust
fn main() {
    let mut s = String::from("hello");
    let slice = &s[0..3];
    // s.push_str(" world");  // ERROR: cannot mutate while slice exists
    println!("{}", slice);
}
```

String literals are also slices. When you write `"hello"`, the type is `&str`
— a slice pointing to data baked into the compiled binary.

Functions that accept `&str` are more flexible than those that accept `&String`,
because they can take both string literals and String slices:

```rust
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        if byte == b' ' {
            return &s[..i];
        }
    }
    s
}
```

## Lifetimes: A Brief Introduction

Lifetimes are Rust's way of ensuring that references never outlive the data they
point to. Most of the time, the compiler figures out lifetimes automatically
(this is called **lifetime elision**). But sometimes you need to annotate them.

Here is the problem lifetimes solve:

```rust
// This will NOT compile:
fn dangling() -> &String {
    let s = String::from("hello");
    &s  // ERROR: s is dropped at end of function, reference would dangle
}
```

The function tries to return a reference to local data. When the function
ends, `s` is dropped, so the reference would point to freed memory. Rust
catches this at compile time.

When a function takes two references and returns one, the compiler needs to
know which input the output's lifetime is tied to. You annotate this with
lifetime parameters:

```rust
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s2 }
}
```

The `'a` annotation says: "the returned reference lives at least as long as
both inputs." You will explore lifetimes more deeply in later lessons. For now,
understand that they exist to prevent dangling references, and the compiler
will tell you when you need to add them.

## Summary

Ownership is what makes Rust unique. The rules are strict but consistent:

- One owner per value. When the owner goes out of scope, the value is dropped.
- Assignment of heap data **moves** ownership. Stack-only `Copy` types are copied.
- `clone()` makes explicit deep copies when you need both the original and a copy.
- References (`&T`) borrow without taking ownership. They are immutable by default.
- Mutable references (`&mut T`) allow modification but enforce exclusive access.
- The borrowing rules (one `&mut` or many `&`, never both) prevent data races at compile time.
- String slices (`&str`) reference parts of a `String` and follow the same borrowing rules.
- Lifetimes ensure references never outlive the data they point to.

In the exercises that follow, you will trigger ownership errors, fix them, clone
data, write functions that borrow, and work with slices — building the muscle
memory that makes Rust's ownership system second nature.
