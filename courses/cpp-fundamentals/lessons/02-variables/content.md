# Variables and Types

Every program needs to store data. In C++, you store data in **variables** —
named containers that hold a value of a specific type.

## Declaring Variables

Before you can use a variable, you must **declare** it. A declaration tells the
compiler the variable's name and type:

```cpp
int age;         // declares an integer variable named "age"
double price;    // declares a floating-point variable named "price"
char letter;     // declares a character variable named "letter"
```

The general form is:

```
type name;
```

## Initialization

Declaring a variable does not give it a value. An uninitialized variable
contains whatever garbage data was in that memory location. Always initialize
your variables:

```cpp
int age = 25;
double price = 9.99;
char letter = 'A';
```

You can also use C++11 uniform initialization with braces:

```cpp
int age{25};
double price{9.99};
char letter{'A'};
```

Both forms are correct. The brace style catches narrowing conversions
(for example, `int x{3.14};` would be a compile error).

## Primitive Types

C++ has several built-in (primitive) types:

| Type     | Description              | Example         |
|----------|--------------------------|-----------------|
| `int`    | Whole numbers            | `42`, `-7`      |
| `double` | Floating-point (64-bit)  | `3.14`, `-0.5`  |
| `float`  | Floating-point (32-bit)  | `2.5f`          |
| `char`   | Single character         | `'A'`, `'9'`    |
| `bool`   | True or false            | `true`, `false`  |

### int

Integers store whole numbers. On most systems, `int` is 32 bits, giving a
range of roughly -2 billion to +2 billion.

### double and float

Both store decimal numbers. `double` has twice the precision of `float` and is
the default choice for floating-point in C++.

### char

A `char` holds a single character enclosed in single quotes. Under the hood, it
stores the character's numeric code (typically ASCII).

### bool

A `bool` holds `true` or `false`. When printed with `std::cout`, booleans
display as `1` or `0` by default. To print the words "true" or "false", use
`std::boolalpha`:

```cpp
bool active = true;
std::cout << std::boolalpha << active << std::endl;  // prints "true"
```

## Assignment

After declaring a variable, you can change its value with the **assignment
operator** (`=`):

```cpp
int score = 0;
score = 100;     // score is now 100
score = score + 5; // score is now 105
```

A variable must be declared before it is used. Attempting to use an undeclared
variable is a compile error.

## Constants

If a value should never change, declare it with `const`:

```cpp
const double PI = 3.14;
const int MAX_SCORE = 100;
```

The compiler will reject any attempt to modify a `const` variable:

```cpp
const int x = 10;
x = 20;  // ERROR: assignment of read-only variable
```

Use `const` for values like mathematical constants, configuration limits, or
any value that is meant to remain fixed throughout the program.

## Type Conversion

C++ can convert values between types. This happens in two ways:

**Implicit conversion** (the compiler does it automatically) occurs when you
assign a value to a wider type. This is safe because no data is lost:

```cpp
int whole = 7;
double widened = whole;  // int → double, safe (7.0)
```

**Explicit conversion** is needed when you narrow a type or want to force a
conversion. Use `static_cast<type>(value)`:

```cpp
double precise = 3.99;
int truncated = static_cast<int>(precise);  // 3 (decimal part dropped)
```

A common pitfall is **integer division**. When both operands are integers, the
result is also an integer with the fractional part discarded:

```cpp
int a = 7;
std::cout << a / 2 << std::endl;    // prints 3 (integer division)
std::cout << a / 2.0 << std::endl;  // prints 3.5 (one operand is double)
```

Making at least one operand a `double` triggers floating-point division.

## The auto Keyword

Since C++11, you can use `auto` to let the compiler **deduce the type** from
the initializer:

```cpp
auto count = 42;      // int (integer literal)
auto pi = 3.14;       // double (floating-point literal)
auto letter = 'Z';    // char (character literal)
```

The compiler figures out the type at compile time — there is no runtime cost.
`auto` is especially handy with long type names (you will see this later with
iterators and templates), but for simple types like `int` and `double` it is
mostly a matter of style. When using `auto`, you **must** provide an
initializer — `auto x;` without a value is a compile error.

## Variable Scope

Every variable has a **scope** — the region of code where it exists and can be
used. In C++, scope is defined by curly braces `{}`:

```cpp
int main() {
    int x = 10;

    if (x > 5) {
        int y = 20;         // y is created here
        std::cout << y;     // OK — y is in scope
    }
    // y no longer exists here — its scope ended at the closing brace

    std::cout << x;  // OK — x is still in scope
    return 0;
}
```

A variable declared inside a block (between `{` and `}`) is destroyed when
that block ends. Trying to use it outside that block is a compile error.

If you need a variable after a block ends, declare it **before** the block
and assign to it inside:

```cpp
int result = 0;
if (true) {
    result = 42;   // assigns to the outer variable
}
std::cout << result;  // OK — result is still in scope
```

## Putting It Together

Here is a short program that uses several variable types:

```cpp
#include <iostream>

int main() {
    int count = 3;
    double average = 4.5;
    char grade = 'B';
    bool passed = true;
    const int MAX = 100;

    std::cout << "Count: " << count << std::endl;
    std::cout << "Average: " << average << std::endl;
    std::cout << "Grade: " << grade << std::endl;
    std::cout << std::boolalpha << "Passed: " << passed << std::endl;
    std::cout << "Max: " << MAX << std::endl;

    return 0;
}
```

In the exercises that follow, you will practice declaring, assigning, and using
variables of different types — including type conversions, `auto`, and scope.
