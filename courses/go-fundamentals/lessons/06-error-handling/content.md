# Error Handling

Go does not have exceptions. There is no `try`/`catch`, no `throw`, no stack
unwinding. Instead, functions that can fail return an `error` value alongside
their result. The caller checks the error explicitly. This makes error handling
visible, predictable, and impossible to accidentally skip.

This is one of Go's most distinctive design choices. It adds a few lines of
code per call, but you always know exactly where errors are handled.

## The error Interface

In Go, `error` is a built-in interface with a single method:

```go
type error interface {
    Error() string
}
```

Any type that has an `Error() string` method satisfies this interface. When you
see a function signature like `func Open(name string) (*File, error)`, the
second return value is either `nil` (no error) or a value describing what went
wrong.

The standard pattern for checking errors:

```go
result, err := someFunction()
if err != nil {
    // handle the error
    fmt.Println("failed:", err)
    return
}
// use result
```

This `if err != nil` pattern appears everywhere in Go code. It is the
language's primary error-handling idiom.

## Creating Errors

The simplest way to create an error is with `errors.New`:

```go
import "errors"

func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}
```

For formatted error messages, use `fmt.Errorf`:

```go
import "fmt"

func lookup(id int) (string, error) {
    if id < 0 {
        return "", fmt.Errorf("invalid id: %d", id)
    }
    return "found", nil
}
```

Both create values that satisfy the `error` interface. The difference is that
`fmt.Errorf` supports format verbs like `%d`, `%s`, and the special `%w` for
wrapping (covered below).

## Custom Error Types

When you need errors that carry structured data beyond a message string, define
your own type that implements `error`:

```go
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation error: field '%s' - %s", e.Field, e.Message)
}
```

You create and return these like any other value:

```go
func validateEmail(email string) error {
    if !strings.Contains(email, "@") {
        return &ValidationError{Field: "email", Message: "missing @"}
    }
    return nil
}
```

Custom error types are useful when callers need to inspect error details
programmatically — not just read a message string.

## Error Wrapping with %w

When one function calls another and wants to add context to the error, it
wraps the original error:

```go
func readConfig(path string) error {
    data, err := os.ReadFile(path)
    if err != nil {
        return fmt.Errorf("read config: %w", err)
    }
    // use data...
    return nil
}
```

The `%w` verb in `fmt.Errorf` wraps the original error inside a new one. The
resulting error's message includes both the new context and the original
message. Crucially, the original error is still accessible for inspection.

You can wrap multiple layers deep:

```go
original := errors.New("file not found")
wrapped1 := fmt.Errorf("read failed: %w", original)
wrapped2 := fmt.Errorf("parse failed: %w", wrapped1)
fmt.Println(wrapped2)
// Output: parse failed: read failed: file not found
```

Each layer adds context while preserving the full chain.

## errors.Is — Checking for Specific Errors

`errors.Is` checks whether any error in a wrapped chain matches a target
error value:

```go
import "errors"

var ErrNotFound = errors.New("not found")

func findUser(id int) error {
    return fmt.Errorf("findUser: %w", ErrNotFound)
}

err := findUser(42)
if errors.Is(err, ErrNotFound) {
    fmt.Println("user not found")
}
```

`errors.Is` unwraps the chain, checking each error. This is different from
`err == ErrNotFound`, which only checks the outermost error. Always use
`errors.Is` instead of `==` when the error might be wrapped.

## errors.As — Extracting Error Types

`errors.As` finds the first error in the chain that matches a specific type
and extracts it:

```go
type APIError struct {
    Code    int
    Message string
}

func (e *APIError) Error() string {
    return fmt.Sprintf("api error: code=%d, message=%s", e.Code, e.Message)
}

err := fmt.Errorf("request failed: %w", &APIError{Code: 404, Message: "not found"})

var apiErr *APIError
if errors.As(err, &apiErr) {
    fmt.Printf("caught: code=%d, message=%s\n", apiErr.Code, apiErr.Message)
}
```

`errors.As` unwraps the chain looking for an error that can be assigned to the
target variable. If found, it sets the target and returns `true`. This lets you
access the structured fields of a custom error type even when it is wrapped.

The key difference: `errors.Is` checks for a specific error **value** (like a
sentinel). `errors.As` checks for a specific error **type** (like a struct).

## panic and recover

Go has `panic` for situations that should never happen — programming errors,
impossible states, unrecoverable failures:

```go
func mustParse(s string) int {
    n, err := strconv.Atoi(s)
    if err != nil {
        panic(fmt.Sprintf("mustParse: %s", err))
    }
    return n
}
```

`panic` stops normal execution, runs deferred functions, and crashes the
program with a stack trace. It is not for expected errors — use `error` returns
for those.

`recover` catches a panic inside a deferred function:

```go
func safeCall(f func()) {
    defer func() {
        if r := recover(); r != nil {
            fmt.Println("recovered:", r)
        }
    }()
    f()
}
```

`recover` returns the value passed to `panic`, or `nil` if no panic occurred.
It only works inside a `defer` — calling it outside a deferred function always
returns `nil`.

**When to use panic:**
- Failed assertions that indicate a bug (index out of bounds, nil pointer)
- Initialization that must succeed (loading required config, opening required DB)

**When NOT to use panic:**
- File not found, network timeout, invalid user input — these are normal errors,
  not panics. Return an `error` instead.

## Sentinel Errors

Sentinel errors are package-level variables that represent specific, well-known
error conditions:

```go
var (
    ErrNotFound  = errors.New("not found")
    ErrForbidden = errors.New("forbidden")
)
```

Callers check for them with `errors.Is`:

```go
err := fetchResource(id)
if errors.Is(err, ErrNotFound) {
    // handle not found
} else if errors.Is(err, ErrForbidden) {
    // handle forbidden
}
```

The standard library uses this pattern extensively: `io.EOF`, `sql.ErrNoRows`,
`os.ErrNotExist`. The naming convention is `Err` prefix followed by the
condition name.

Sentinel errors work well with wrapping. A function deep in the call stack
returns `ErrNotFound`, intermediate functions wrap it with `fmt.Errorf("...: %w", err)`,
and the top-level handler checks with `errors.Is(err, ErrNotFound)`.

## The Complete Error Handling Toolbox

| Tool | Purpose | Example |
|------|---------|---------|
| `errors.New` | Create simple error | `errors.New("failed")` |
| `fmt.Errorf` | Create formatted error | `fmt.Errorf("bad id: %d", id)` |
| `fmt.Errorf` + `%w` | Wrap with context | `fmt.Errorf("open: %w", err)` |
| `errors.Is` | Check for sentinel | `errors.Is(err, ErrNotFound)` |
| `errors.As` | Extract error type | `errors.As(err, &target)` |
| Custom type | Structured error data | `&ValidationError{...}` |
| `panic` | Unrecoverable failure | `panic("impossible")` |
| `recover` | Catch panic | `defer func() { recover() }()` |

The exercises that follow will give you hands-on practice with each of these
tools, from basic error returns to building wrapped error chains and recovering
from panics.
