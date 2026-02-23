# Arrays, Slices & Maps

Go has three core collection types. **Arrays** are fixed-size sequences.
**Slices** are dynamic, flexible views backed by arrays. **Maps** are
unordered key-value stores. Most Go code uses slices and maps far more
than raw arrays, but understanding arrays is essential because slices
are built on top of them.

## Arrays

An array has a fixed length that is part of its type. `[5]int` and `[3]int`
are different types -- you cannot assign one to the other.

```go
var numbers [5]int             // zero-valued: [0 0 0 0 0]
numbers[0] = 1
numbers[4] = 25

primes := [4]int{2, 3, 5, 7}  // literal initialization
fmt.Println(len(primes))       // 4
```

You can also let the compiler count the elements:

```go
colors := [...]string{"red", "green", "blue"}  // length inferred as 3
```

Arrays are values in Go, not references. Assigning one array to another
copies all elements. This is different from most other languages.

```go
a := [3]int{1, 2, 3}
b := a          // b is a copy
b[0] = 99
fmt.Println(a)  // [1 2 3] -- unchanged
fmt.Println(b)  // [99 2 3]
```

## Slices

A slice is a dynamically-sized, flexible view into an array. Slices are
what you will use most of the time.

```go
fruits := []string{"apple", "banana", "cherry"}
fmt.Println(len(fruits))  // 3
fmt.Println(cap(fruits))  // 3
```

Notice the syntax difference: `[5]int` is an array (size specified),
`[]int` is a slice (no size). Every slice has three properties:

- **length** (`len(s)`) -- the number of elements it contains
- **capacity** (`cap(s)`) -- the number of elements in the underlying array,
  counting from the slice's start
- **pointer** -- to the first element in the underlying array

You can create a slice from an array:

```go
arr := [5]int{10, 20, 30, 40, 50}
s := arr[1:4]      // [20 30 40]  -- elements at index 1, 2, 3
fmt.Println(len(s)) // 3
fmt.Println(cap(s)) // 4 (from index 1 to end of arr)
```

The slice expression `[low:high]` selects elements from index `low` up to
(but not including) `high`. This is the same half-open interval convention
used in Python and many other languages.

## Slice Operations

### append

The `append` function adds elements to the end of a slice. It returns a new
slice -- you must capture the return value:

```go
s := []int{1, 2, 3}
s = append(s, 4)
s = append(s, 5, 6)   // multiple elements at once
fmt.Println(s)         // [1 2 3 4 5 6]
```

When the underlying array is full, `append` allocates a new, larger array
and copies the elements. This is why you must always use the return value.

### Slicing an existing slice

You can re-slice a slice to get a sub-range:

```go
nums := []int{1, 2, 3, 4, 5}
sub := nums[1:4]    // [2 3 4]
```

Important: the new slice shares the same underlying array. Modifying one
can affect the other:

```go
sub[0] = 99
fmt.Println(nums)  // [1 99 3 4 5] -- nums changed too!
```

### copy

To get an independent slice, use `copy`:

```go
src := []int{1, 2, 3}
dst := make([]int, len(src))
copy(dst, src)
dst[0] = 99
fmt.Println(src)  // [1 2 3] -- unchanged
```

## Maps

A map is an unordered collection of key-value pairs. Keys must be
comparable types (strings, ints, etc.). Values can be any type.

```go
capitals := map[string]string{
    "France": "Paris",
    "Japan":  "Tokyo",
    "Brazil": "Brasilia",
}
fmt.Println(capitals["France"])  // Paris
```

### Adding and deleting

```go
capitals["Germany"] = "Berlin"   // add
delete(capitals, "Brazil")       // remove
```

### Checking if a key exists

Looking up a missing key returns the zero value, which can be ambiguous.
Use the comma-ok idiom:

```go
val, ok := capitals["France"]
fmt.Println(val, ok)   // Paris true

val, ok = capitals["Mars"]
fmt.Println(val, ok)   // "" false
```

The second value `ok` is a boolean: `true` if the key was found, `false`
otherwise. This is a Go idiom you will see everywhere.

## The range Keyword

The `range` keyword iterates over arrays, slices, maps, strings, and
channels. It returns two values per iteration.

For slices and arrays, `range` gives the index and value:

```go
nums := []int{10, 20, 30}
for i, v := range nums {
    fmt.Printf("%d: %d\n", i, v)
}
// 0: 10
// 1: 20
// 2: 30
```

If you only need the index, drop the second variable:

```go
for i := range nums {
    fmt.Println(i)
}
```

If you only need the value, use the blank identifier `_`:

```go
for _, v := range nums {
    fmt.Println(v)
}
```

For maps, `range` gives the key and value:

```go
for country, capital := range capitals {
    fmt.Printf("%s -> %s\n", country, capital)
}
```

**Warning:** Map iteration order is not guaranteed in Go. Each run may
produce a different ordering. If you need sorted output, collect the keys
into a slice, sort it, and iterate over that.

## make() for Allocation

The `make` function creates slices, maps, and channels. It is the
idiomatic way to create these types when you know the initial size:

```go
// Slice with length 5, capacity 10
s := make([]int, 5, 10)

// Slice with length 0, capacity 100 (grow via append)
buf := make([]int, 0, 100)

// Map with space pre-allocated for ~50 entries
m := make(map[string]int, 50)

// Map with default capacity
m2 := make(map[string]int)
```

Pre-allocating with `make` avoids repeated resizing when you know roughly
how many elements you will add. This is a performance optimization, not
a requirement -- your code works without it, just slightly slower for
large collections.

## Nested Collections

Go lets you compose collection types freely. A common pattern is a map
whose values are slices:

```go
hobbies := map[string][]string{
    "Alice": {"coding", "reading"},
    "Bob":   {"gaming", "hiking"},
}
```

To build this incrementally:

```go
hobbies := make(map[string][]string)
hobbies["Alice"] = append(hobbies["Alice"], "coding")
hobbies["Alice"] = append(hobbies["Alice"], "reading")
hobbies["Bob"] = append(hobbies["Bob"], "gaming", "hiking")
```

This works because `append` on a nil slice creates a new slice. You do not
need to initialize each key's slice before appending to it.

You can also have slices of maps, maps of maps, or any other combination.
The type system keeps everything explicit:

```go
// Slice of maps
records := []map[string]string{
    {"name": "Alice", "role": "engineer"},
    {"name": "Bob", "role": "designer"},
}

// Map of maps
org := map[string]map[string]string{
    "engineering": {"lead": "Alice", "size": "12"},
    "design":      {"lead": "Bob", "size": "5"},
}
```

In the exercises that follow, you will build and manipulate each of these
collection types.
