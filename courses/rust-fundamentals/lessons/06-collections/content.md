# Collections & Iterators

## Vec<T> — The Growable Array

A `Vec<T>` (pronounced "vector of T") is Rust's most common collection. It stores a variable number of values of the same type in a contiguous, heap-allocated buffer that grows as needed.

```rust
let mut v: Vec<i32> = Vec::new();
v.push(10);
v.push(20);
v.push(30);
println!("{:?}", v);  // [10, 20, 30]
```

The `vec!` macro is a shorthand for creating a vector with initial values:

```rust
let v = vec![1, 2, 3, 4, 5];
println!("length: {}", v.len());  // length: 5
```

## Accessing Vec Elements

Use indexing with `[]` or the safer `.get()` method:

```rust
let v = vec![10, 20, 30];

println!("{}", v[0]);   // 10
println!("{}", v[2]);   // 30
// v[5] would panic at runtime!

// .get() returns Option<&T> instead of panicking
match v.get(5) {
    Some(val) => println!("{}", val),
    None => println!("out of bounds"),  // prints this
}
```

Indexing with `[]` panics on out-of-bounds access. Use `.get()` when the index might be invalid.

## Modifying a Vec

```rust
let mut v = vec![1, 2, 3];

v.push(4);           // add to end: [1, 2, 3, 4]
let last = v.pop();  // remove from end: returns Some(4)
v[0] = 99;           // overwrite first element: [99, 2, 3]

println!("popped: {:?}", last);  // popped: Some(4)
println!("vec: {:?}", v);        // vec: [99, 2, 3]
```

`pop()` returns `Option<T>` because the vector might be empty.

## Iterating Over a Vec

The `for` loop works naturally with vectors:

```rust
let v = vec![10, 20, 30];

for val in &v {
    println!("{}", val);
}
// v is still usable here because we borrowed it with &
```

Use `&v` (borrow) to iterate without consuming the vector. Use `&mut v` to modify elements in-place. Use `v` directly (without `&`) to consume the vector and take ownership of each element.

## HashMap<K, V> — Key-Value Pairs

`HashMap` lives in `std::collections` and stores key-value pairs with O(1) average lookup time.

```rust
use std::collections::HashMap;

let mut scores = HashMap::new();
scores.insert("Alice", 95);
scores.insert("Bob", 82);
scores.insert("Carol", 91);

println!("{:?}", scores);
```

Note: HashMap does not preserve insertion order. The printed order may vary between runs.

## Reading from a HashMap

`.get()` returns `Option<&V>`:

```rust
use std::collections::HashMap;

let mut m = HashMap::new();
m.insert("apple", 3);
m.insert("banana", 5);

match m.get("apple") {
    Some(count) => println!("apple: {}", count),  // apple: 3
    None => println!("not found"),
}
```

## The Entry API

The `entry` API handles the common pattern of "insert if missing, or modify if present":

```rust
use std::collections::HashMap;

let mut word_count = HashMap::new();
let words = vec!["hello", "world", "hello", "rust"];

for word in &words {
    let count = word_count.entry(word).or_insert(0);
    *count += 1;
}
println!("{:?}", word_count);
// {"hello": 2, "world": 1, "rust": 1}
```

`entry()` returns an `Entry` enum. `or_insert(0)` returns a mutable reference to the value -- inserting 0 first if the key was absent. Then `*count += 1` dereferences and increments.

## String — Growable UTF-8 Text

Rust has two main string types: `&str` (a borrowed string slice) and `String` (an owned, growable buffer). `String` is actually a `Vec<u8>` that guarantees valid UTF-8.

```rust
let mut s = String::from("hello");
s.push_str(" world");
s.push('!');
println!("{}", s);  // hello world!
```

The `format!` macro works like `println!` but returns a `String` instead of printing:

```rust
let name = "Rust";
let version = 2021;
let msg = format!("{} edition {}", name, version);
println!("{}", msg);  // Rust edition 2021
```

## Iterating Over String Characters

Strings are UTF-8, so you cannot index them by byte position (`s[0]` does not compile). Instead, use `.chars()`:

```rust
let s = String::from("hello");
for c in s.chars() {
    println!("{}", c);
}
// Prints: h, e, l, l, o (each on its own line)
```

You can also collect characters into a vector or count them:

```rust
let s = String::from("hello");
let char_count = s.chars().count();
println!("{} characters", char_count);  // 5 characters
```

## Iterator Basics

An **iterator** is a value that produces a sequence of elements one at a time. In Rust, iterators are lazy -- they do nothing until consumed.

The three ways to create an iterator from a collection:

| Method         | Yields         | Collection after? |
|----------------|----------------|-------------------|
| `.iter()`      | `&T` (borrows) | Still usable      |
| `.iter_mut()`  | `&mut T`       | Still usable      |
| `.into_iter()` | `T` (owned)    | Consumed           |

```rust
let v = vec![1, 2, 3];

// .iter() borrows each element
for val in v.iter() {
    println!("{}", val);
}
// v is still valid here
```

## Closures

A **closure** is an anonymous function that can capture variables from its surrounding scope. Closures use `|params|` instead of `fn(params)`:

```rust
let add_one = |x: i32| x + 1;
println!("{}", add_one(5));  // 6

let name = String::from("Rust");
let greet = || println!("Hello, {}!", name);  // captures `name`
greet();  // Hello, Rust!
```

Closures can infer parameter and return types from context, so type annotations are often optional. They are the key ingredient for iterator adapters.

## Map, Filter, and Collect

Iterator adapters transform one iterator into another. The most common:

**`.map()`** applies a closure to each element:

```rust
let v = vec![1, 2, 3, 4, 5];
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
println!("{:?}", doubled);  // [2, 4, 6, 8, 10]
```

**`.filter()`** keeps only elements where the closure returns true:

```rust
let v = vec![1, 2, 3, 4, 5, 6];
let evens: Vec<&i32> = v.iter().filter(|x| *x % 2 == 0).collect();
println!("{:?}", evens);  // [2, 4, 6]
```

Note: `.filter()` receives `&&i32` (a reference to the iterator's `&i32` item), so you dereference once with `*x` or use pattern matching `|&&x|`.

**Chaining** adapters is where iterators shine:

```rust
let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let result: Vec<i32> = v.iter()
    .filter(|&&x| x % 2 == 0)
    .map(|&x| x * x)
    .collect();
println!("{:?}", result);  // [4, 16, 36, 64, 100]
```

This reads naturally: take the vector, keep the even numbers, square each one, and collect into a new vector.

## Enumerate and Zip

**`.enumerate()`** pairs each element with its index:

```rust
let fruits = vec!["apple", "banana", "cherry"];
for (i, fruit) in fruits.iter().enumerate() {
    println!("{}: {}", i, fruit);
}
// 0: apple
// 1: banana
// 2: cherry
```

**`.zip()`** pairs elements from two iterators:

```rust
let names = vec!["Alice", "Bob", "Carol"];
let scores = vec![95, 82, 91];

for (name, score) in names.iter().zip(scores.iter()) {
    println!("{}: {}", name, score);
}
// Alice: 95
// Bob: 82
// Carol: 91
```

`zip` stops when either iterator runs out, so mismatched lengths are safe.

## Collecting into Different Types

`.collect()` can produce different collection types depending on the type annotation:

```rust
use std::collections::HashMap;

let pairs = vec![("one", 1), ("two", 2), ("three", 3)];
let map: HashMap<&str, i32> = pairs.into_iter().collect();
println!("{:?}", map);
```

The turbofish syntax `::<Type>` is an alternative to a type annotation on the variable:

```rust
let v = vec![1, 2, 3];
let doubled = v.iter().map(|x| x * 2).collect::<Vec<_>>();
println!("{:?}", doubled);  // [2, 4, 6]
```

The `_` in `Vec<_>` tells Rust to infer the element type.

## Sum, Min, Max

Iterators also have consuming methods that reduce to a single value:

```rust
let v = vec![3, 1, 4, 1, 5, 9];
let total: i32 = v.iter().sum();
let biggest = v.iter().max().unwrap();
let smallest = v.iter().min().unwrap();
println!("sum={}, max={}, min={}", total, biggest, smallest);
// sum=23, max=9, min=1
```

These are **consuming** adapters -- they drive the iterator to completion and return a result.
