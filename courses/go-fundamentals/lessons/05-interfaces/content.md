# Interfaces

In the previous lesson you learned how to attach methods to structs. Interfaces
take this a step further: they let you define **behavior** without specifying
how that behavior is implemented. This is Go's primary tool for abstraction and
polymorphism.

If you have used interfaces in Java or C#, Go's version will feel familiar but
with one crucial difference: implementation is **implicit**. There is no
`implements` keyword. A type satisfies an interface simply by having the right
methods.

## Defining an Interface

An interface is a type that specifies a set of method signatures:

```go
type Shape interface {
    Area() float64
    Perimeter() float64
}
```

This says: "any type that has both an `Area() float64` method and a
`Perimeter() float64` method is a `Shape`." The interface does not care what
the type is called or how it is structured internally.

## Implicit Implementation

In Go, you never write `type Circle implements Shape`. You just define the
methods:

```go
type Circle struct {
    Radius float64
}

func (c Circle) Area() float64 {
    return math.Pi * c.Radius * c.Radius
}

func (c Circle) Perimeter() float64 {
    return 2 * math.Pi * c.Radius
}
```

Because `Circle` has both `Area() float64` and `Perimeter() float64`, it
automatically satisfies the `Shape` interface. No registration, no annotation,
no boilerplate.

This design has a powerful consequence: you can define an interface in one
package and satisfy it with a type from a completely different package, even one
you did not write.

## Using Interfaces

Once you have an interface, you can write functions that accept **any** type
satisfying it:

```go
func printArea(s Shape) {
    fmt.Printf("area: %.2f\n", s.Area())
}

func main() {
    c := Circle{Radius: 5}
    printArea(c)    // works — Circle satisfies Shape
}
```

The function `printArea` does not know or care whether it receives a `Circle`,
a `Rectangle`, or something else entirely. It only cares that the value has an
`Area()` method.

## The Empty Interface

The empty interface `interface{}` has zero methods, which means every type
satisfies it:

```go
func printAnything(v interface{}) {
    fmt.Println(v)
}

printAnything(42)
printAnything("hello")
printAnything(true)
```

Since Go 1.18, you can also write `any` as an alias for `interface{}`:

```go
func printAnything(v any) {
    fmt.Println(v)
}
```

The empty interface is useful when you need a function that accepts values of
unknown type, but you lose type safety. Use it sparingly.

## Type Assertions

When you have a value of an interface type, you sometimes need to get the
concrete value back. A **type assertion** does this:

```go
var i interface{} = "hello"

s := i.(string)     // s is now the string "hello"
fmt.Println(s)
```

If the assertion is wrong, the program panics:

```go
n := i.(int)        // panic: interface conversion: interface {} is string, not int
```

To avoid panics, use the **comma-ok** pattern:

```go
s, ok := i.(string)
if ok {
    fmt.Println("it's a string:", s)
}

n, ok := i.(int)
if !ok {
    fmt.Println("not an int")
}
```

The second return value `ok` is a `bool` that tells you whether the assertion
succeeded.

## Type Switches

When you need to check multiple types, a **type switch** is cleaner than a
chain of comma-ok assertions:

```go
func describe(i interface{}) string {
    switch v := i.(type) {
    case int:
        return fmt.Sprintf("integer: %d", v)
    case string:
        return fmt.Sprintf("text: %s", v)
    case bool:
        return fmt.Sprintf("boolean: %v", v)
    default:
        return "unknown"
    }
}
```

Notice the special syntax `i.(type)` — this only works inside a `switch`
statement. The variable `v` is automatically typed to the matched case, so
inside `case int:` you can use `v` as an `int` directly.

## The Stringer Interface

The `fmt` package defines a widely-used interface called `Stringer`:

```go
type Stringer interface {
    String() string
}
```

If your type implements `String() string`, then `fmt.Println`, `fmt.Printf`
with `%v`, and other formatting functions will use your method to represent the
value:

```go
type Color struct {
    R, G, B uint8
}

func (c Color) String() string {
    return fmt.Sprintf("rgb(%d, %d, %d)", c.R, c.G, c.B)
}

func main() {
    c := Color{255, 128, 0}
    fmt.Println(c)    // prints: rgb(255, 128, 0)
}
```

The `Stringer` interface is a great example of Go's interface philosophy: it is
small (one method), widely applicable, and you opt in by simply defining the
method.

## Multiple Interfaces

A type can satisfy any number of interfaces simultaneously. Go encourages
small, focused interfaces:

```go
type Reader interface {
    Read() string
}

type Writer interface {
    Write(s string)
}
```

A single type can implement both:

```go
type Document struct {
    content string
}

func (d Document) Read() string {
    return d.content
}

func (d *Document) Write(s string) {
    d.content = s
}
```

Now `Document` can be passed to any function that expects a `Reader` and
`*Document` can be passed to any function that expects a `Writer`. You can also
compose interfaces:

```go
type ReadWriter interface {
    Reader
    Writer
}
```

This **embedding** says: a `ReadWriter` is anything that is both a `Reader` and
a `Writer`. Interface embedding is how the standard library builds up complex
interfaces from simple ones (e.g., `io.ReadWriter` embeds `io.Reader` and
`io.Writer`).

## Interface Design Philosophy

Go's standard library favors interfaces with one or two methods:

| Interface        | Method(s)          | Package |
|------------------|--------------------|---------|
| `fmt.Stringer`   | `String() string`  | fmt     |
| `error`          | `Error() string`   | builtin |
| `io.Reader`      | `Read([]byte) (int, error)` | io |
| `io.Writer`      | `Write([]byte) (int, error)` | io |
| `sort.Interface` | `Len`, `Less`, `Swap` | sort |

The smaller the interface, the more types can satisfy it. This is the opposite
of the Java/C# approach where interfaces often have many methods. In Go, the
advice is: **accept interfaces, return structs**.

In the exercises that follow, you will define interfaces, implement them
implicitly, work with the empty interface, use type assertions and type
switches, implement `fmt.Stringer`, and build types that satisfy multiple
interfaces.
