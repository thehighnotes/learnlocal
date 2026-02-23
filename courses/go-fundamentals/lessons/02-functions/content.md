# Functions

Functions are the building blocks of Go programs. Every Go program starts with `func main()`, but real programs break logic into many small, focused functions. Go's function design is simple but powerful — multiple return values, first-class functions, and closures are all built in.

## Basic Functions

You define a function with the `func` keyword, a name, parameters in parentheses, and optionally a return type:

```go
func greet() {
    fmt.Println("Hello!")
}

func add(a int, b int) int {
    return a + b
}
```

Call a function by writing its name followed by arguments:

```go
greet()              // prints: Hello!
result := add(3, 4)  // result is 7
```

When consecutive parameters share the same type, you can write the type just once:

```go
func add(a, b int) int {    // same as add(a int, b int) int
    return a + b
}
```

This shorthand is idiomatic Go. You will see it everywhere.

## Multiple Return Values

Go functions can return multiple values. This is not a workaround or a library feature — it is built into the language. The most common pattern is returning a value and an error:

```go
func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, fmt.Errorf("division by zero")
    }
    return a / b, nil
}
```

The caller receives both values:

```go
result, err := divide(10, 3)
if err != nil {
    fmt.Println("Error:", err)
} else {
    fmt.Println(result)
}
```

The `(value, error)` pattern is the idiomatic way to handle errors in Go. You will see it in the standard library, in third-party packages, and in virtually every Go codebase. The convention is that the error is always the last return value.

If you do not need one of the return values, use the blank identifier `_`:

```go
_, err := divide(10, 0)  // only care about the error
```

## Named Return Values

You can give names to return values. Named returns act as variables declared at the top of the function, initialized to their zero values:

```go
func rectangleArea(width, height float64) (area float64) {
    area = width * height
    return  // "bare return" — returns the named value
}
```

A bare `return` sends back whatever the named return variables currently hold. This is useful for short functions but can hurt readability in longer ones — use it judiciously.

Named returns also serve as documentation. The function signature tells the caller what each return value means:

```go
func divide(a, b float64) (result float64, err error) {
    if b == 0 {
        err = fmt.Errorf("division by zero")
        return
    }
    result = a / b
    return
}
```

Here, `result` and `err` are both zero-initialized (`0.0` and `nil`). The function assigns the ones it needs and uses a bare `return`.

## Variadic Functions

A variadic function accepts any number of arguments of a given type. You declare the parameter with `...` before the type:

```go
func sum(nums ...int) int {
    total := 0
    for _, n := range nums {
        total += n
    }
    return total
}
```

Call it with any number of arguments — including zero:

```go
fmt.Println(sum(1, 2, 3))  // 6
fmt.Println(sum(10, 20))   // 30
fmt.Println(sum())          // 0
```

Inside the function, `nums` is a slice (`[]int`). You can also pass an existing slice using `...`:

```go
numbers := []int{1, 2, 3, 4, 5}
fmt.Println(sum(numbers...))  // 15
```

`fmt.Println` itself is a variadic function — that is why it can accept any number of arguments.

The variadic parameter must be the last parameter in the function signature. You can have regular parameters before it:

```go
func printAll(prefix string, values ...int) {
    for _, v := range values {
        fmt.Printf("%s: %d\n", prefix, v)
    }
}
```

## First-Class Functions

In Go, functions are first-class values. You can assign them to variables, pass them as arguments, and return them from other functions:

```go
func apply(f func(int, int) int, a, b int) int {
    return f(a, b)
}

func add(a, b int) int { return a + b }
func multiply(a, b int) int { return a * b }

fmt.Println(apply(add, 3, 4))      // 7
fmt.Println(apply(multiply, 3, 4)) // 12
```

The type `func(int, int) int` means "a function that takes two ints and returns an int." You pass function names without parentheses — `add`, not `add()`.

You can also use anonymous functions (function literals) inline:

```go
result := apply(func(a, b int) int { return a - b }, 10, 3)
fmt.Println(result)  // 7
```

This pattern is common in Go for callbacks, sorting comparators, and HTTP handlers.

## Closures

A closure is a function that captures variables from its enclosing scope. The function "remembers" those variables even after the outer function has returned:

```go
func makeCounter() func() int {
    count := 0
    return func() int {
        count++
        return count
    }
}

counter := makeCounter()
fmt.Println(counter())  // 1
fmt.Println(counter())  // 2
fmt.Println(counter())  // 3
```

Each call to `makeCounter()` creates a new, independent `count` variable. The returned function holds a reference to it, so `count` persists between calls.

Closures are useful for:
- **Encapsulation**: hiding state inside a function without exposing it globally.
- **Factories**: creating specialized functions from a template.
- **Callbacks**: passing behavior along with the data it needs.

```go
func multiplier(factor int) func(int) int {
    return func(x int) int {
        return x * factor
    }
}

double := multiplier(2)
triple := multiplier(3)
fmt.Println(double(5))  // 10
fmt.Println(triple(5))  // 15
```

## Recursion

A recursive function calls itself to solve a problem by breaking it into smaller subproblems. Every recursive function needs:

1. **Base case**: a condition that stops the recursion.
2. **Recursive case**: the function calls itself with a smaller input.

The classic example is factorial (n! = n * (n-1) * ... * 1):

```go
func factorial(n int) int {
    if n == 0 {
        return 1  // base case
    }
    return n * factorial(n-1)  // recursive case
}

fmt.Println(factorial(5))  // 120
```

The call chain for `factorial(5)`:

```
factorial(5) = 5 * factorial(4)
factorial(4) = 4 * factorial(3)
factorial(3) = 3 * factorial(2)
factorial(2) = 2 * factorial(1)
factorial(1) = 1 * factorial(0)
factorial(0) = 1 (base case)
```

The results multiply back: 1 * 1 * 2 * 3 * 4 * 5 = 120.

Without a base case, the function calls itself forever until Go panics with a stack overflow. Always make sure the recursive call moves toward the base case.

Go does not optimize tail calls, so deep recursion can overflow the stack. For large inputs, an iterative loop is usually better. But for naturally recursive problems (trees, nested structures, mathematical sequences), recursion keeps the code clean and expressive.

## Putting It All Together

Here is an example combining several concepts from this lesson:

```go
func applyToEach(f func(int) int, nums ...int) []int {
    results := make([]int, len(nums))
    for i, n := range nums {
        results[i] = f(n)
    }
    return results
}

double := func(x int) int { return x * 2 }
squared := applyToEach(func(x int) int { return x * x }, 1, 2, 3, 4)
doubled := applyToEach(double, 10, 20, 30)

fmt.Println(squared)  // [1 4 9 16]
fmt.Println(doubled)  // [20 40 60]
```

This combines:
- A **variadic parameter** (`nums ...int`) to accept any number of values.
- A **function parameter** (`f func(int) int`) to accept any transformation.
- An **anonymous function** passed inline for squaring.
- A **closure** stored in a variable for doubling.
- **Multiple return mechanics** — the function builds and returns a slice.

Functions are how you structure every Go program. The exercises that follow will help you practice each concept individually.
