# Functions

Functions let you organize code into reusable, named blocks. Instead of writing
the same logic over and over, you define it once in a function and call it
whenever you need it.

## Declaring and Defining Functions

A function has four parts: a return type, a name, a parameter list, and a body.

```cpp
return_type function_name(parameter_list) {
    // body
}
```

The simplest function takes no parameters and returns nothing (`void`):

```cpp
void greet() {
    std::cout << "Hello, World!" << std::endl;
}
```

To use a function, you **call** it by name with parentheses:

```cpp
int main() {
    greet();  // prints "Hello, World!"
    return 0;
}
```

A function must be declared before it is called. You can either define the
function above `main`, or write a **forward declaration** (prototype) at the
top and define it later:

```cpp
void greet();  // forward declaration

int main() {
    greet();
    return 0;
}

void greet() {  // definition
    std::cout << "Hello, World!" << std::endl;
}
```

## Parameters

Parameters let you pass data into a function. Each parameter has a type and
a name, separated by commas:

```cpp
void printSum(int a, int b) {
    std::cout << a + b << std::endl;
}
```

When you call the function, you provide **arguments** that match the parameters:

```cpp
printSum(3, 7);  // prints 10
```

## Return Values

Functions can send a result back to the caller using `return`. The return type
in the function signature tells the compiler what kind of value to expect:

```cpp
int square(int x) {
    return x * x;
}
```

You capture the returned value in the calling code:

```cpp
int result = square(5);
std::cout << result << std::endl;  // prints 25
```

A `void` function does not return a value. If your function computes something
the caller needs, use a non-void return type --- do not accidentally declare it
`void`.

## Function Overloading

C++ allows multiple functions with the **same name** as long as their parameter
lists differ. This is called **overloading**:

```cpp
void print(int value) {
    std::cout << "Integer: " << value << std::endl;
}

void print(double value) {
    std::cout << "Double: " << value << std::endl;
}
```

The compiler picks the right version based on the arguments you pass:

```cpp
print(42);    // calls print(int)
print(3.14);  // calls print(double)
```

Overloading is resolved at compile time by matching argument types to parameter
types. The functions must differ in the number or types of their parameters ---
differing only in return type is not enough.

## Pass by Value vs Pass by Reference

So far, every parameter we have written is **pass by value**: the function
receives a copy of the argument. Changing the copy inside the function has no
effect on the original variable:

```cpp
void tryDouble(int x) {
    x = x * 2;  // modifies the local copy only
}

int main() {
    int n = 5;
    tryDouble(n);
    std::cout << n << std::endl;  // still 5
}
```

When you need a function to modify the caller's variable, use a **reference
parameter** by adding `&` after the type:

```cpp
void doubleIt(int& x) {
    x = x * 2;  // modifies the original
}

int main() {
    int n = 5;
    doubleIt(n);
    std::cout << n << std::endl;  // now 10
}
```

A reference parameter is an alias for the original variable --- no copy is
made. This is also useful for large objects (like strings or vectors) where
copying would be expensive. If you want the efficiency of a reference but do
not want to modify the argument, use `const&`:

```cpp
void show(const std::string& text) {
    std::cout << text << std::endl;  // read-only, no copy
}
```

## Default Parameters

You can give parameters **default values** so the caller can omit them:

```cpp
void greet(std::string name, std::string greeting = "Hello") {
    std::cout << greeting << ", " << name << "!" << std::endl;
}

greet("Alice");         // Hello, Alice!
greet("Bob", "Hi");    // Hi, Bob!
```

The rules for default parameters:

- Defaults must start from the **rightmost** parameter and go left. You cannot
  skip a parameter in the middle.
- A parameter with a default can be omitted by the caller, and the default
  value is used automatically.
- Default values are specified in the declaration (or the definition if there
  is no separate declaration), not both.

```cpp
// OK: rightmost parameters have defaults
void setup(int width, int height = 600, int depth = 32);

// ERROR: non-default parameter after a default
// void setup(int width = 800, int height);
```

## Recursion

A **recursive** function is one that calls itself. Every recursive function
needs two things:

1. **Base case** --- a condition that stops the recursion.
2. **Recursive case** --- the function calls itself with a smaller problem.

The classic example is factorial (n! = n * (n-1) * ... * 1):

```cpp
int factorial(int n) {
    if (n <= 1) return 1;     // base case
    return n * factorial(n - 1);  // recursive case
}
```

Walking through `factorial(4)`:

- `factorial(4)` returns `4 * factorial(3)`
- `factorial(3)` returns `3 * factorial(2)`
- `factorial(2)` returns `2 * factorial(1)`
- `factorial(1)` hits the base case, returns `1`
- Results unwind: 2 * 1 = 2, 3 * 2 = 6, 4 * 6 = 24

**Warning:** Every recursive call adds a frame to the call stack. Without a
correct base case, the function calls itself forever and crashes with a stack
overflow. Always make sure the input gets closer to the base case on every call.

## Compiling Multiple Files

Real C++ projects split code across multiple files. A common pattern is:

- A **header file** (`.h`) that declares function prototypes
- A **source file** (`.cpp`) that defines the function bodies
- A **main file** (`main.cpp`) that calls those functions

```
math_utils.h     // declares: int add(int, int);
math_utils.cpp   // defines:  int add(int a, int b) { return a + b; }
main.cpp         // calls:    add(3, 4)
```

To compile, you list all `.cpp` files together. The compiler compiles each one
and the **linker** combines them into a single executable:

```bash
g++ -o calculator main.cpp math_utils.cpp
./calculator
```

You do **not** list `.h` files on the command line -- `#include` handles that.
The compiler reads each `.cpp` file, sees the `#include "math_utils.h"`
directive, and uses the header to verify that function calls match their
declarations.

## Summary

| Concept             | Key Point                                      |
|---------------------|-------------------------------------------------|
| Declaration         | return_type name(params);                       |
| void functions      | Perform an action, return nothing               |
| Parameters          | Pass data in; matched by position               |
| Return values       | Send a result back to the caller                |
| Overloading         | Same name, different parameter lists            |
| Pass by reference   | `&` lets a function modify the original variable|
| Default parameters  | Rightmost params can have fallback values       |
| Recursion           | Function calls itself; needs a base case        |

With functions in your toolbox, you can break problems into small, testable
pieces and reuse logic across your programs.
