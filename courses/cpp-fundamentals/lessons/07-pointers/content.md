# Pointers and References

## What Is a Pointer?

Every variable in your program lives at a specific location in memory. A **pointer** is a variable that stores the memory address of another variable, rather than storing a data value directly.

Think of it like a street address: the address isn't the house itself, but it tells you exactly where to find the house.

```cpp
int x = 42;
int* ptr = &x;  // ptr holds the address of x
```

## The Address-Of Operator: &

The `&` operator, when placed before a variable name, gives you the address of that variable in memory.

```cpp
int score = 100;
int* p = &score;  // p now points to score
```

You generally should not print raw addresses (they change every run). What matters is the relationship: `p` points to `score`, and through `p` you can access the value stored there.

## Dereferencing a Pointer: *

The `*` operator, when used on a pointer, gives you the value stored at the address the pointer holds. This is called **dereferencing**.

```cpp
int x = 42;
int* ptr = &x;

std::cout << *ptr << std::endl;  // prints 42
```

Notice the dual use of `*`:
- In a **declaration** (`int* ptr`), it means "ptr is a pointer to int."
- In an **expression** (`*ptr`), it means "get the value that ptr points to."

You can also modify the original variable through a dereferenced pointer:

```cpp
int x = 10;
int* ptr = &x;
*ptr = 99;
std::cout << x << std::endl;  // prints 99 -- x was changed through ptr
```

## Pointer Arithmetic

When you have a pointer to an element of an array, you can use arithmetic to move between elements. Adding 1 to a pointer advances it to the next element (not the next byte -- the compiler accounts for the element size).

```cpp
int arr[5] = {10, 20, 30, 40, 50};
int* p = arr;       // points to arr[0]

std::cout << *(p + 2) << std::endl;  // prints 30 (arr[2])
```

This is why arrays and pointers are so closely related in C++: the array name itself decays to a pointer to the first element.

## References

A **reference** is an alias for an existing variable. Once created, it refers to the same memory location as the original.

```cpp
int x = 10;
int& ref = x;   // ref is another name for x
ref = 20;
std::cout << x << std::endl;  // prints 20
```

Key differences from pointers:
- A reference **must be initialized** when declared.
- A reference **cannot be reassigned** to refer to a different variable.
- No special syntax to access the value -- just use the reference name directly.

References are commonly used for function parameters to avoid copying large objects and to allow functions to modify the caller's variables.

## nullptr -- The Null Pointer

Sometimes a pointer doesn't point to anything yet, or you need to represent "no object." In modern C++, you use the keyword `nullptr` for this.

```cpp
int* ptr = nullptr;  // ptr points to nothing
```

In older C and C++ code, you might see `NULL` or even `0` used for this purpose. Prefer `nullptr` in modern C++ -- it's type-safe and avoids ambiguity with the integer `0`.

**Always check before dereferencing** a pointer that might be null. Dereferencing a null pointer is undefined behavior -- your program will almost certainly crash.

```cpp
int* ptr = nullptr;

if (ptr != nullptr) {
    std::cout << *ptr << std::endl;
} else {
    std::cout << "pointer is null" << std::endl;
}
```

Because pointers implicitly convert to `bool` (`nullptr` is falsy, anything else is truthy), you can also write:

```cpp
if (ptr) {
    std::cout << *ptr << std::endl;
}
```

Both styles are common. Use whichever your team prefers -- the explicit `!= nullptr` is clearer for beginners.

## The Array-Pointer Relationship

When you pass an array to a function, it **decays** to a pointer to its first element. The function receives a pointer, not a copy of the array.

```cpp
void print_array(int* arr, int size) {
    for (int i = 0; i < size; i++) {
        std::cout << *(arr + i) << std::endl;
    }
}

int main() {
    int nums[3] = {10, 20, 30};
    print_array(nums, 3);  // nums decays to &nums[0]
}
```

This means the function can modify the original array's elements through the pointer. It also means the function has no way to know the array's size -- you must pass the size separately (or use `std::array` / `std::vector` from the standard library).

You can iterate through an array using a pointer and `++`:

```cpp
int arr[5] = {1, 2, 3, 4, 5};
int* p = arr;

for (int i = 0; i < 5; i++) {
    std::cout << *p << std::endl;
    p++;  // advance to next element
}
```

This pattern -- walking a pointer through an array -- is fundamental to how C and C++ work under the hood. Even bracket notation `arr[i]` is defined as `*(arr + i)`.

## Common Pointer Pitfalls

Pointers are powerful but demand care. Three common mistakes:

**Dangling pointers** -- A pointer that refers to memory that has been freed or a local variable that has gone out of scope. Dereferencing it is undefined behavior.

**Uninitialized pointers** -- A pointer declared without a value holds garbage. Always initialize pointers, even if just to `nullptr`.

```cpp
int* p;         // BAD -- p holds garbage
int* p = nullptr;  // GOOD -- explicitly "points to nothing"
```

**Double-free** -- Freeing (deleting) the same memory twice. This corrupts the heap and can crash your program or worse. We will revisit this when we cover dynamic memory.

For now, the key habit: **initialize your pointers** and **check before you dereference**.

## Pointers vs References -- When to Use Which

| Feature          | Pointer         | Reference       |
|------------------|-----------------|-----------------|
| Can be null      | Yes             | No              |
| Can be reassigned| Yes             | No              |
| Syntax overhead  | `*` and `&`     | None after init |
| Use case         | Dynamic memory, arrays, optional values | Function params, aliases |

As a general guideline: prefer references when you can, use pointers when you must. Specifically:

- **Use a reference** when the target will always exist and won't change.
- **Use a pointer** when you need to represent "nothing" (`nullptr`), when you need to reassign to a different object, or when working with arrays and dynamic memory.
- **Use `const` references** (`const int& x`) to pass large objects without copying and without allowing modification.

In practice, most function parameters in modern C++ are references or `const` references. Pointers appear when you need their specific capabilities -- nullability, reassignment, or array traversal.
