# Structs & Methods

So far you have worked with built-in types like `int`, `string`, and `float64`, and
you have organized data in slices and maps. Real programs need **custom types** that
group related data into a meaningful unit. In Go, the primary tool for this is
the **struct**.

## Defining a Struct

A struct groups named fields into a single type. By convention, exported struct
names and exported field names use **PascalCase**:

```go
type Point struct {
    X float64
    Y float64
}
```

This defines a new type called `Point`. It does not create a variable -- it defines
a blueprint.

## Creating Struct Values

Create a struct value using a **struct literal**. You can name the fields
explicitly or provide values in declaration order:

```go
p1 := Point{X: 3.0, Y: 4.0}   // named fields (preferred)
p2 := Point{3.0, 4.0}          // positional (fragile -- breaks if fields reorder)
```

The named-field form is strongly preferred in Go because it is resilient to
field reordering and self-documenting.

Access fields with the **dot operator**:

```go
fmt.Println(p1.X, p1.Y)   // 3 4
```

## Zero Values

If you declare a struct variable without initializing it, every field gets its
**zero value** (`0` for numbers, `""` for strings, `false` for bools, `nil` for
pointers and slices):

```go
var p Point
fmt.Println(p.X, p.Y)   // 0 0
```

You can also omit fields in a struct literal -- omitted fields get zero values:

```go
p := Point{X: 5.0}   // Y is 0.0
```

## Methods -- Attaching Behavior

Go does not have classes, but you can define **methods** on any named type. A
method is a function with a special **receiver** parameter between `func` and
the method name:

```go
func (p Point) Distance() float64 {
    return math.Sqrt(p.X*p.X + p.Y*p.Y)
}
```

The receiver `(p Point)` means "this method is attached to the `Point` type."
Call it with dot syntax:

```go
p := Point{X: 3.0, Y: 4.0}
fmt.Println(p.Distance())   // 5
```

## Value Receivers vs Pointer Receivers

The receiver can be either a **value** or a **pointer**. This distinction matters:

| Receiver        | Syntax          | Gets a...   | Can modify original? |
|-----------------|-----------------|-------------|----------------------|
| Value receiver  | `(p Point)`     | Copy        | No                   |
| Pointer receiver| `(p *Point)`    | Pointer     | Yes                  |

Use a **value receiver** when the method only reads data:

```go
func (p Point) String() string {
    return fmt.Sprintf("(%g, %g)", p.X, p.Y)
}
```

Use a **pointer receiver** when the method needs to **modify** the struct:

```go
func (p *Point) Scale(factor float64) {
    p.X *= factor
    p.Y *= factor
}
```

Go automatically takes the address when you call a pointer method on a value:

```go
p := Point{X: 1.0, Y: 2.0}
p.Scale(3.0)                    // Go automatically does (&p).Scale(3.0)
fmt.Println(p.X, p.Y)          // 3 6
```

A good rule of thumb: if any method needs a pointer receiver, give **all** methods
on that type pointer receivers for consistency.

## Struct Embedding -- Composition over Inheritance

Go does not have inheritance. Instead, it uses **embedding** to compose types.
When you embed a struct, its fields and methods are **promoted** to the outer type:

```go
type Animal struct {
    Name string
}

func (a Animal) Speak() string {
    return a.Name + " makes a sound"
}

type Dog struct {
    Animal          // embedded -- no field name
    Breed string
}
```

Now `Dog` has access to `Name` and `Speak()` as if they were its own:

```go
d := Dog{
    Animal: Animal{Name: "Rex"},
    Breed:  "Labrador",
}
fmt.Println(d.Name)       // Rex       (promoted from Animal)
fmt.Println(d.Speak())    // Rex makes a sound
fmt.Println(d.Breed)      // Labrador
```

This is **not** inheritance -- `Dog` is not an `Animal`. It is composition: `Dog`
**has** an `Animal`. The embedded `Animal` is still accessible as `d.Animal`.

## Constructor Functions -- The New Pattern

Go has no constructors. Instead, the convention is to write a function named
`NewTypeName` that returns a pointer to an initialized struct:

```go
func NewPoint(x, y float64) *Point {
    return &Point{X: x, Y: y}
}
```

Use constructors when:
- Fields need validation or computed defaults
- The zero value is not useful
- You want to return a pointer (common for types with pointer receiver methods)

```go
p := NewPoint(3.0, 4.0)
fmt.Println(p.Distance())   // 5
```

If the zero value of your struct is already useful (like `bytes.Buffer` or
`sync.Mutex`), you do not need a constructor.

## Struct Tags

Struct tags are string literals attached to fields that provide metadata. They
are most commonly used with `encoding/json`, but any package can read them via
reflection:

```go
type User struct {
    Name  string `json:"name"`
    Email string `json:"email"`
    Age   int    `json:"age,omitempty"`
}
```

The tag syntax is a raw string literal after the field type. Tags do not affect
how you use the struct in Go code -- they are metadata for libraries. For now,
the key thing to know is the syntax and that they exist. You will use them
extensively when working with JSON, databases, and validation libraries.

## Method Sets and Interfaces

Every type has a **method set** -- the set of methods you can call on values of
that type. The method set rules are:

| Type     | Method set includes                        |
|----------|--------------------------------------------|
| `T`      | Methods with value receiver `(t T)`        |
| `*T`     | Methods with value **or** pointer receiver |

This matters for interfaces (next lesson). A value of type `T` can only satisfy
an interface if all the interface methods are in the method set of `T`. Since
pointer receiver methods are **not** in the method set of `T`, you would need
a `*T` to satisfy that interface.

```go
type Mover interface {
    Move(dx, dy float64)
}

func (p *Point) Move(dx, dy float64) {
    p.X += dx
    p.Y += dy
}

var m Mover = &Point{X: 1, Y: 2}   // OK: *Point has Move
// var m Mover = Point{X: 1, Y: 2}  // ERROR: Point does not have Move
```

For now, remember the rule: if you use pointer receivers, you need a pointer
to satisfy interfaces. You will practice this more in the interfaces lesson.

## Summary

Structs are Go's primary tool for defining custom types. Methods give those
types behavior. Pointer receivers enable mutation. Embedding provides
composition. Constructor functions initialize complex types. Struct tags
attach metadata. And method sets govern which interfaces a type can satisfy.

Together, these features give you everything you need to model complex domains
without classes or inheritance.
