# Error Handling

Most languages handle errors with exceptions. Rust takes a different approach:
errors are values. There is no `try`/`catch`. Instead, functions that can fail
return a `Result` type, and the caller decides what to do with it. This makes
error handling explicit, visible in function signatures, and impossible to
accidentally ignore.

## Result<T, E>

The `Result` enum is defined in the standard library:

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`T` is the success type. `E` is the error type. A function that parses a string
into a number might return `Result<i32, String>` — either an `Ok(i32)` on
success or an `Err(String)` describing what went wrong:

```rust
fn parse_age(input: &str) -> Result<i32, String> {
    match input.trim().parse::<i32>() {
        Ok(age) if age >= 0 => Ok(age),
        Ok(age) => Err(format!("age cannot be negative: {}", age)),
        Err(e) => Err(format!("not a number: {}", e)),
    }
}
```

The caller then decides how to handle the result:

```rust
match parse_age("25") {
    Ok(age) => println!("Age: {}", age),
    Err(msg) => println!("Error: {}", msg),
}
```

Because `Result` is a regular enum, the compiler forces you to handle both
cases. You cannot accidentally ignore an error.

## The ? Operator

Writing `match` for every fallible call gets verbose. The `?` operator is
syntactic sugar that propagates errors automatically:

```rust
fn read_age(input: &str) -> Result<i32, String> {
    let age = input.trim().parse::<i32>()
        .map_err(|e| format!("not a number: {}", e))?;
    if age < 0 {
        return Err(format!("age cannot be negative: {}", age));
    }
    Ok(age)
}
```

When you put `?` after a `Result`, two things happen:
- If the value is `Ok(v)`, the `?` extracts `v` and the expression evaluates to it.
- If the value is `Err(e)`, the `?` returns early from the function with that error.

This means `?` can only be used inside functions that return `Result` (or
`Option`). The error type must be compatible with the function's return type.

You can chain multiple `?` calls to write clean, linear error-handling code:

```rust
fn process(a: &str, b: &str) -> Result<i32, String> {
    let x = a.parse::<i32>().map_err(|e| e.to_string())?;
    let y = b.parse::<i32>().map_err(|e| e.to_string())?;
    Ok(x + y)
}
```

## unwrap() and expect()

Sometimes you know a `Result` will be `Ok`, or you are writing a quick prototype
where you do not want to handle errors yet. Two methods help:

```rust
let n: i32 = "42".parse().unwrap();       // panics if Err
let n: i32 = "42".parse().expect("bad");  // panics with "bad" if Err
```

`unwrap()` extracts the `Ok` value. If the `Result` is `Err`, it panics —
crashing the program with the error's debug representation.

`expect("message")` does the same, but the panic message includes your custom
text. This makes debugging easier: when you see the panic output, you know
which `expect` failed and why.

**Rule of thumb:** Use `unwrap`/`expect` in tests, examples, and early
prototypes. In production code, propagate errors with `?` or handle them
with `match`.

## Custom Error Types

For real applications, using `String` as your error type is too vague. Define
an enum that lists your specific failure modes:

```rust
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    InvalidInput(String),
    ParseFailed(ParseIntError),
    OutOfRange(i32),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "invalid input: {}", msg),
            AppError::ParseFailed(e) => write!(f, "parse error: {}", e),
            AppError::OutOfRange(n) => write!(f, "out of range: {}", n),
        }
    }
}

impl std::error::Error for AppError {}
```

A proper Rust error type needs three things:
1. `#[derive(Debug)]` — so it can be printed in debug format
2. `impl Display` — so it can be printed in user-friendly format
3. `impl std::error::Error` — so it integrates with the standard error trait

## The From Trait for Error Conversion

When your function calls code that returns a different error type, you need to
convert between them. The `From` trait makes this automatic:

```rust
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::ParseFailed(e)
    }
}
```

Now the `?` operator can convert `ParseIntError` into `AppError` automatically:

```rust
fn parse_config(input: &str) -> Result<i32, AppError> {
    let value = input.parse::<i32>()?;  // ParseIntError -> AppError via From
    if value < 0 || value > 100 {
        return Err(AppError::OutOfRange(value));
    }
    Ok(value)
}
```

Without the `From` impl, you would need `.map_err(AppError::ParseFailed)?`
every time. With it, `?` handles the conversion invisibly.

You can implement `From` for as many source error types as you need:

```rust
impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::InvalidInput(e.to_string())
    }
}
```

## Error Propagation Patterns

In real Rust code, errors flow upward through the call stack. Each function
either handles an error or propagates it to its caller:

```rust
fn validate(input: &str) -> Result<i32, AppError> {
    let n = input.parse::<i32>()?;       // propagate parse errors
    if n < 1 || n > 100 {
        return Err(AppError::OutOfRange(n));  // create new error
    }
    Ok(n)
}

fn run(input: &str) -> Result<String, AppError> {
    let value = validate(input)?;        // propagate validation errors
    Ok(format!("Valid: {}", value))
}
```

The function at the top of the chain (often `main`) is where errors are finally
handled and reported to the user:

```rust
fn main() {
    match run("42") {
        Ok(msg) => println!("{}", msg),
        Err(e) => println!("Error: {}", e),
    }
}
```

This pattern keeps intermediate functions clean — they just use `?` to pass
errors along — while the top-level function decides how to present errors.

## Matching on Error Variants

Since `Result` and your custom error enums are regular enums, you can `match`
on them to handle different error cases differently:

```rust
match parse_config("abc") {
    Ok(value) => println!("Config: {}", value),
    Err(AppError::ParseFailed(e)) => println!("Bad number: {}", e),
    Err(AppError::OutOfRange(n)) => println!("{} is out of range", n),
    Err(AppError::InvalidInput(msg)) => println!("Bad input: {}", msg),
}
```

This is how you provide specific error messages or recovery logic for each
failure mode. The compiler ensures you handle every variant if you do not
include a wildcard `_` arm.

You can also match on standard library errors. For example, `parse::<i32>()`
returns `Result<i32, ParseIntError>`, and you can match on it directly:

```rust
match "abc".parse::<i32>() {
    Ok(n) => println!("Got: {}", n),
    Err(e) => println!("Parse failed: {}", e),
}
```

## Choosing Your Approach

- Use `match` when you need to handle each error variant differently.
- Use `?` when you want to propagate errors without handling them locally.
- Use `unwrap()`/`expect()` only in tests, examples, or when failure is truly impossible.
- Use `String` as the error type for small scripts and prototypes.
- Use custom error enums for libraries and applications with multiple failure modes.
- Implement `From` to make `?` work seamlessly across error types.

The exercises that follow will give you hands-on practice with each of these
patterns, from basic `Result` handling to building your own error types with
automatic conversion.
