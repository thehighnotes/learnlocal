# Hello, Go!

Go is a compiled language created at Google. It compiles fast, runs fast, and
has a deliberately small feature set. There is one way to format code, one way
to handle errors, and one way to structure packages. This simplicity is the
point. You spend less time debating style and more time shipping working
software.

This lesson covers the absolute basics: packages, imports, printing, variables,
types, constants, and the `iota` keyword.

## Your First Go Program

Every Go program follows the same structure:

```go
package main

import "fmt"

func main() {
    fmt.Println("Hello, World!")
}
```

Three things are required:

1. **`package main`** — declares this file as part of the main package, which
   is the entry point for an executable program.
2. **`import "fmt"`** — imports the `fmt` (format) package from the standard
   library. This gives you access to functions like `Println` and `Printf`.
3. **`func main()`** — the main function. Execution starts here. No arguments,
   no return value.

### Printing Output

The `fmt` package provides several print functions:

```go
fmt.Println("Hello, World!")       // prints with a newline at the end
fmt.Print("no newline after this") // prints without a trailing newline
fmt.Printf("Name: %s\n", "Alice") // formatted print, like C's printf
```

`Println` is the most common. It adds a newline automatically and separates
multiple arguments with spaces:

```go
fmt.Println("Name:", "Alice", "Age:", 30)
// Output: Name: Alice Age: 30
```

`Printf` uses format verbs to control the output precisely:

| Verb   | Meaning                    | Example           |
|--------|----------------------------|-------------------|
| `%s`   | String                     | `"hello"`         |
| `%d`   | Integer (decimal)          | `42`              |
| `%f`   | Floating point             | `3.140000`        |
| `%v`   | Default format (any type)  | works for anything|
| `%T`   | Type of a value            | `int`, `string`   |

`Printf` does **not** add a newline — you must include `\n` yourself.

## Variables

Go has two ways to declare variables.

### The `var` keyword

```go
var name string = "Alice"
var age int = 30
var height float64 = 5.7
```

You can also declare without an initial value. The variable gets its **zero
value** (more on that below):

```go
var count int      // count is 0
var label string   // label is ""
var active bool    // active is false
```

Multiple variables can be declared in a block:

```go
var (
    x int    = 10
    y int    = 20
    z string = "hello"
)
```

### Short declarations with `:=`

Inside a function, you can use `:=` to declare and assign in one step. Go
infers the type from the right-hand side:

```go
name := "Alice"    // string
age := 30          // int
height := 5.7      // float64
active := true     // bool
```

This is the most common way to declare variables in Go. The `:=` operator can
only be used inside functions — it does not work at the package level.

**Important:** `:=` declares a **new** variable. If the variable already exists,
use `=` to reassign:

```go
x := 10   // declare and assign
x = 20    // reassign (variable already exists)
x := 30   // ERROR: no new variables on left side of :=
```

## Basic Types

Go has a small set of built-in types:

| Type      | Description          | Example values     |
|-----------|----------------------|--------------------|
| `int`     | Integer              | `0`, `42`, `-7`    |
| `float64` | Floating point       | `3.14`, `0.0`      |
| `string`  | Text                 | `"hello"`, `""`    |
| `bool`    | Boolean              | `true`, `false`    |

There are also sized integer types (`int8`, `int16`, `int32`, `int64`) and
unsigned variants (`uint`, `uint8`, etc.), but `int` and `float64` are the
defaults you will use most often.

Strings in Go are enclosed in double quotes. Single quotes are for single
characters (`rune` type), not strings:

```go
name := "Alice"   // string (double quotes)
letter := 'A'     // rune, which is an alias for int32 (single quotes)
```

## Constants

Constants are declared with `const` and cannot be changed after declaration:

```go
const pi = 3.14159
const greeting = "Hello"
```

Constants can be grouped in a block:

```go
const (
    pi      = 3.14159
    e       = 2.71828
    maxSize = 100
)
```

Constants must be assignable at compile time. You cannot use a function call
as a constant value (except for a few built-in functions like `len`).

Constants in Go are **untyped** by default. The constant `pi = 3.14159` is not
a `float64` until you use it in a context that requires one. This means
constants can be used flexibly across different numeric types without explicit
conversion.

## Zero Values

In Go, every variable has a **zero value** — the default value assigned when no
explicit value is given. There is no concept of "uninitialized" memory in Go.

| Type      | Zero value |
|-----------|-----------|
| `int`     | `0`       |
| `float64` | `0`       |
| `string`  | `""`      |
| `bool`    | `false`   |

```go
var i int
var f float64
var s string
var b bool
fmt.Println(i, f, s, b)
// Output: 0 0  false
```

Notice the empty space between `0` and `false` — that is the empty string `s`
being printed. `Println` separates each argument with a space, so the empty
string appears as an extra gap.

Zero values are one of Go's best features for safety. You never get garbage
data from forgetting to initialize a variable.

## Type Inference

Go infers types from the value on the right-hand side of an assignment:

```go
x := 42      // int (not int64, not int32 — just int)
y := 3.14    // float64 (Go defaults to float64, not float32)
z := "hello" // string
w := true    // bool
```

When you use `:=`, you almost never need to write out the type. The compiler
figures it out. This keeps code concise without sacrificing type safety — every
variable still has a concrete type, it just was not written explicitly.

You can verify the inferred type with `%T` in `Printf`:

```go
x := 42
fmt.Printf("x is %T with value %d\n", x, x)
// Output: x is int with value 42
```

### The `iota` Keyword

`iota` is a special constant generator used inside `const` blocks. It starts at
0 and increments by 1 for each constant in the block:

```go
const (
    Sunday    = iota  // 0
    Monday            // 1
    Tuesday           // 2
    Wednesday         // 3
    Thursday          // 4
    Friday            // 5
    Saturday          // 6
)
```

After the first constant uses `iota`, subsequent constants in the same block
automatically continue the pattern. You do not need to repeat `= iota`.

`iota` resets to 0 at the start of each new `const` block:

```go
const (
    Red   = iota  // 0
    Green         // 1
    Blue          // 2
)

const (
    Small  = iota  // 0 again
    Medium         // 1
    Large          // 2
)
```

You can also use expressions with `iota`:

```go
const (
    _  = iota         // 0 (discarded with blank identifier)
    KB = 1 << (10 * iota)  // 1 << 10 = 1024
    MB                     // 1 << 20 = 1048576
    GB                     // 1 << 30 = 1073741824
)
```

`iota` is Go's way of creating enum-like constants without a separate enum
type. It is clean, expressive, and very commonly used.

## Checking Your Go Installation

The `go` command is your all-in-one tool for compiling, running, testing, and
managing Go code. You can check it is installed:

```bash
go version
```

This prints something like `go1.22.0 linux/amd64`. Knowing your version matters
because newer versions support newer language features.

## Building and Running

There are two ways to run Go code from the command line:

- **`go run`** compiles to a temporary file and runs it immediately. Great for
  development.
- **`go build`** creates a permanent binary you can distribute. The binary is a
  single self-contained executable with no dependencies.

```bash
go run hello.go           # compile and run in one step
go build -o hello hello.go  # create a permanent binary
./hello                     # run the binary
```

The `-o hello` flag names the output binary.

## Formatting with gofmt

Go has an official code formatter called `gofmt`. Unlike other languages where
formatting is debated, Go has exactly one style:

```bash
gofmt messy.go         # prints formatted code to stdout
gofmt -w messy.go      # formats the file in place
gofmt messy.go > clean.go  # saves formatted code to a new file
```

Running `gofmt` is standard practice in Go. Most editors do it automatically on
save.
