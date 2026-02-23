# Hello World & Program Structure

Every C++ program follows the same basic pattern. Before you write anything
fancy, you need to understand the skeleton that every program is built on.
This lesson walks through each piece so that nothing feels like magic.

## Your First Program

Here is the simplest useful C++ program:

```cpp
#include <iostream>

int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}
```

That is five lines of code and it does one thing: prints "Hello, World!" to the
terminal. Every part of it matters, so let us break it down.

### The main() Function

Every C++ program must have exactly one function named `main`. This is the
**entry point** -- the first function the operating system calls when you run
your program.

```cpp
int main() {
    // your code goes here
    return 0;
}
```

Breaking this down:

- `int` is the return type. `main` returns an integer to the operating system.
- `main()` is the function name with an empty parameter list.
- The curly braces `{ }` contain the function body -- the actual code that runs.
- `return 0;` tells the operating system "everything went fine." A non-zero
  return value signals an error.

You will always write `int main()`. Some old tutorials show `void main()`, but
that is not valid standard C++. Compilers may accept it, but it is technically
wrong and some compilers will reject it outright.

## Headers and #include

The very first line is a **preprocessor directive**:

```cpp
#include <iostream>
```

This tells the compiler: "Before you compile my code, copy the contents of the
`iostream` header into this file." A header is a file that declares tools you
can use. The `iostream` header gives you `std::cout` (for printing output) and
`std::cin` (for reading input).

Think of `#include` as importing a toolbox. Without it, the compiler has no idea
what `std::cout` means and will refuse to compile your code.

Key points about `#include`:

- It always starts with `#` (no space before it)
- Standard library headers use angle brackets: `<iostream>`
- You can include multiple headers, each on its own line
- If you forget the `#include`, you will get a compile error about undeclared names

Some common headers you will see later in this course:

```cpp
#include <iostream>   // input/output (cout, cin)
#include <string>     // std::string type
#include <vector>     // std::vector (dynamic arrays)
#include <cmath>      // math functions (sqrt, pow, etc.)
```

### Statements and Semicolons

Every instruction in C++ is a **statement**, and every statement ends with a
**semicolon** `;`. Forget the semicolon and the compiler will give you an error:

```cpp
std::cout << "This is fine" << std::endl;   // OK
std::cout << "Missing semicolon" << std::endl  // ERROR!
```

Semicolons are one of the most common beginner mistakes. If you get a confusing
compile error, check the line **above** the one the compiler points to -- it is
often a missing semicolon.

Note that preprocessor directives like `#include` do **not** end with a
semicolon. They are processed before compilation and follow different rules.

## Printing with std::cout

`std::cout` is the standard output stream. You send data to it using the
**insertion operator** `<<`:

```cpp
std::cout << "Hello, World!";
```

You can chain multiple `<<` operators to print several things in a row:

```cpp
std::cout << "Name: " << "Alice" << ", Age: " << 30;
```

This prints: `Name: Alice, Age: 30` -- all on one line, with no automatic
spaces between items. You control the formatting entirely.

### Ending a Line

There are two common ways to move to a new line:

```cpp
std::cout << "First line" << std::endl;
std::cout << "Second line" << "\n";
```

Both produce a newline. The difference:

- `std::endl` writes a newline **and** flushes the output buffer (forces the
  text to appear immediately).
- `"\n"` writes a newline without flushing. It is slightly faster.

For learning purposes, either one is fine. You will see both in real code.

## Comments

Comments let you leave notes in your code. The compiler ignores them completely.

**Single-line comments** start with `//`:

```cpp
// This is a comment
std::cout << "Hello" << std::endl;  // This is also a comment
```

**Multi-line comments** are wrapped in `/* */`:

```cpp
/* This comment
   spans multiple
   lines */
```

Use comments to explain **why** you did something, not **what** the code does.
Good code is mostly self-explanatory; comments fill in the reasoning.

Comments are also useful for temporarily disabling code:

```cpp
// std::cout << "This line won't run" << std::endl;
```

## Escape Characters

Sometimes you need to print characters that have special meaning, like quotes or
tabs. C++ uses **escape sequences** that start with a backslash `\`:

| Escape | Meaning          | Example output   |
|--------|------------------|------------------|
| `\n`   | Newline          | (moves to next line) |
| `\t`   | Tab              | (horizontal tab) |
| `\\`   | Literal backslash | `\`             |
| `\"`   | Literal quote    | `"`              |

For example, to print a tab-separated table:

```cpp
std::cout << "Item\tPrice" << std::endl;
std::cout << "Apple\t1.50" << std::endl;
```

Output:

```
Item    Price
Apple   1.50
```

You can put multiple escape characters in a single string. They work anywhere
inside double quotes.

## Checking Your Compiler

The code you write is called **source code** and lives in a `.cpp` file. But
your computer cannot run source code directly. It needs to be **compiled** into
an executable -- a file of machine instructions your CPU understands.

The process looks like this:

```
hello.cpp  -->  [g++ compiler]  -->  hello  (executable)
```

Before you compile anything, you can verify that g++ is installed:

```bash
g++ --version
```

This prints version information like `g++ (Ubuntu 13.2.0) 13.2.0`. The version
number tells you which C++ standards your compiler supports.

## Compiling and Running

Using the g++ compiler from the command line:

```bash
g++ -o hello hello.cpp
./hello
```

The flags:

- `-o hello` names the output executable "hello" (without it, g++ creates `a.out`)
- `hello.cpp` is your source file

If there are errors in your code, the compiler will print error messages and
refuse to produce an executable. You fix the errors, then compile again. This
compile-fix cycle is a normal part of programming.

## Piping Input to a Program

When a program reads from `std::cin`, you can provide input by piping data to it
with `echo`:

```bash
echo Alice | ./greeting
```

This sends `"Alice"` to the program's standard input, just as if someone typed
it interactively. The `|` (pipe) connects the output of `echo` to the input of
your program.
