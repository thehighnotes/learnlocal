# Control Flow

In the previous lesson every line ran top to bottom. Control flow statements let you
make decisions, repeat work, and clean up resources — the three things that turn a
script into a real program.

Go's control flow is deliberately simple. There is one loop keyword, conditions
never need parentheses, and switches don't fall through by accident.

## if / else

Go's `if` looks like other languages with one important difference: **no parentheses
around the condition**, but **braces are always required**.

```go
x := 10

if x > 0 {
    fmt.Println("positive")
} else if x < 0 {
    fmt.Println("negative")
} else {
    fmt.Println("zero")
}
```

The opening brace `{` must be on the same line as the `if` — this is not a style
preference, it is enforced by the compiler. Go uses automatic semicolon insertion,
and a newline before `{` would insert a semicolon in the wrong place.

## Short Variable Declarations in if

Go lets you declare a variable inside the `if` statement itself. The variable is
scoped to the entire if/else chain and does not leak outside:

```go
if length := len("hello"); length > 3 {
    fmt.Println("long word:", length)
} else {
    fmt.Println("short word:", length)
}
// length is not accessible here
```

The pattern is `if initialization; condition { ... }`. The semicolon separates the
initialization from the condition. This is idiomatic Go — you will see it constantly
in error handling:

```go
if err := doSomething(); err != nil {
    fmt.Println("error:", err)
}
```

## for — Go's Only Loop

Go has a single loop keyword: `for`. It replaces `while`, `do-while`, and C-style
`for` from other languages.

**C-style for loop** (init; condition; post):

```go
for i := 0; i < 5; i++ {
    fmt.Println(i)
}
```

**While-style** (condition only):

```go
n := 1
for n <= 100 {
    n *= 2
}
fmt.Println(n) // 128
```

**Infinite loop** (no condition):

```go
for {
    // runs forever until break or return
}
```

Like `if`, the `for` loop needs no parentheses around its clauses, and braces are
mandatory. The init and post statements are optional — leaving them off gives you
a while loop. Leaving everything off gives you an infinite loop.

## switch

Go's `switch` is cleaner than in C, C++, or Java because **cases do not fall through
by default**. Each case is independent — no `break` required.

```go
day := 3

switch day {
case 1:
    fmt.Println("Monday")
case 2:
    fmt.Println("Tuesday")
case 3:
    fmt.Println("Wednesday")
default:
    fmt.Println("Other day")
}
```

You can list multiple values in a single case:

```go
switch day {
case 6, 7:
    fmt.Println("Weekend")
default:
    fmt.Println("Weekday")
}
```

If you *want* fallthrough (rare), you must say so explicitly with the `fallthrough`
keyword. This is the opposite of C/C++ where you must remember `break`.

Switch cases can also use expressions and comparisons when you switch on no value:

```go
switch {
case score >= 90:
    fmt.Println("A")
case score >= 80:
    fmt.Println("B")
default:
    fmt.Println("C or below")
}
```

This "expression switch" is a clean replacement for long if/else chains.

## break and continue

**`break`** exits the innermost `for` loop immediately:

```go
for i := 1; i <= 10; i++ {
    if i == 6 {
        break
    }
    fmt.Println(i)
}
// Prints 1 through 5
```

**`continue`** skips the rest of the current iteration and moves to the next:

```go
for i := 1; i <= 5; i++ {
    if i == 3 {
        continue
    }
    fmt.Println(i)
}
// Prints 1, 2, 4, 5
```

These work the same as in C-family languages. Where Go differs is with labeled
loops.

## Labeled Loops

When you have nested loops, `break` and `continue` affect only the innermost loop.
Labels let you target an outer loop:

```go
outer:
    for i := 0; i < 3; i++ {
        for j := 0; j < 3; j++ {
            if i == 1 && j == 1 {
                break outer
            }
            fmt.Printf("(%d,%d) ", i, j)
        }
    }
```

The label `outer:` is placed directly before the `for` statement. `break outer`
exits the outer loop entirely, not just the inner one. Without the label, the
break would only exit the inner loop.

## defer

The `defer` keyword schedules a function call to run **after the enclosing function
returns**. Deferred calls are stacked — they execute in last-in, first-out (LIFO)
order.

```go
func main() {
    fmt.Println("start")
    defer fmt.Println("deferred-1")
    defer fmt.Println("deferred-2")
    defer fmt.Println("deferred-3")
    fmt.Println("end")
}
```

Output:
```
start
end
deferred-3
deferred-2
deferred-1
```

`"start"` and `"end"` print immediately. The three deferred calls execute after
`main()` finishes, in reverse order.

Common uses for `defer`:
- Closing files: `defer file.Close()`
- Unlocking mutexes: `defer mu.Unlock()`
- Cleaning up resources

The key rule: **a deferred call's arguments are evaluated immediately**, but the
call itself is not executed until the surrounding function returns. This means:

```go
x := 10
defer fmt.Println(x) // will print 10, even if x changes later
x = 20
```

## Choosing the Right Tool

- Use `if/else` for branching on conditions.
- Use `switch` for clean multi-way branching — prefer it over long if/else chains.
- Use `for` with init/condition/post for counted loops.
- Use `for` with condition only as a while loop.
- Use `break` and `continue` for early exits and skipping iterations.
- Use labels when you need to break out of nested loops.
- Use `defer` to guarantee cleanup code runs when the function exits.
- Use short declarations in `if` to keep temporary variables tightly scoped.
