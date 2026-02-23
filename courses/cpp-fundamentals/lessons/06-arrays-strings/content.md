# Arrays and Strings

## What Is an Array?

An **array** is a fixed-size collection of elements that are all the same type, stored in contiguous memory. When you need to work with a list of values -- test scores, temperatures, pixel colors -- an array is the basic building block.

```cpp
int scores[5] = {90, 85, 72, 98, 88};
```

This creates an array of 5 integers. The size is fixed at compile time and cannot change later.

You can also let the compiler figure out the size from the initializer:

```cpp
int scores[] = {90, 85, 72, 98, 88};  // compiler knows it's 5
```

## Accessing Array Elements

Array elements are accessed by **index**, starting at **0**. An array of size 5 has valid indices 0 through 4.

```cpp
int nums[] = {10, 20, 30, 40, 50};

std::cout << nums[0] << std::endl;  // 10 (first element)
std::cout << nums[2] << std::endl;  // 30 (third element)
std::cout << nums[4] << std::endl;  // 50 (last element)
```

You can also assign to individual elements:

```cpp
nums[1] = 99;  // change second element from 20 to 99
```

## Iterating Arrays with for Loops

The most common way to process every element in an array is a `for` loop that counts from 0 to the array size:

```cpp
int data[] = {3, 7, 2, 8, 5};
int size = 5;

for (int i = 0; i < size; i++) {
    std::cout << data[i] << std::endl;
}
```

Notice the condition is `i < size`, not `i <= size`. Using `<=` would access `data[5]`, which is past the end of the array.

## Common Array Patterns

**Summing all elements:**

```cpp
int data[] = {3, 7, 2, 8, 5};
int sum = 0;
for (int i = 0; i < 5; i++) {
    sum += data[i];
}
std::cout << sum << std::endl;  // 25
```

**Finding the maximum:**

```cpp
int data[] = {14, 3, 27, 8, 19};
int max = data[0];  // start with the first element
for (int i = 1; i < 5; i++) {
    if (data[i] > max) {
        max = data[i];
    }
}
std::cout << max << std::endl;  // 27
```

The key insight: initialize `max` with the first element, then compare against the rest starting at index 1.

## Range-Based for Loops

C++11 introduced a cleaner syntax for iterating over all elements:

```cpp
int data[] = {3, 7, 2, 8, 5};

for (int val : data) {
    std::cout << val << std::endl;
}
```

This is called a **range-based for loop**. The variable `val` takes on each element's value in turn. Use this when you need every element and don't need the index.

## Out-of-Bounds Access

Accessing an array index outside its valid range is **undefined behavior** in C++. The compiler will not stop you, and the program may appear to work -- or it may crash, corrupt data, or behave unpredictably.

```cpp
int arr[3] = {1, 2, 3};
std::cout << arr[5] << std::endl;  // undefined behavior!
```

This is one of the most common bugs in C and C++ programs. Always double-check your loop bounds.

## std::string Basics

While C has character arrays (`char[]`) for text, C++ provides `std::string` -- a much safer and more convenient string type. You need to include the `<string>` header.

```cpp
#include <iostream>
#include <string>

int main() {
    std::string greeting = "Hello";
    std::cout << greeting << std::endl;      // Hello
    std::cout << greeting.length() << std::endl;  // 5
    return 0;
}
```

`std::string` manages its own memory, grows automatically, and provides useful methods. Always prefer it over C-style `char[]` arrays.

## String Operations

**Concatenation** with `+`:

```cpp
std::string first = "Hello";
std::string second = "World";
std::string combined = first + " " + second;
std::cout << combined << std::endl;  // Hello World
```

**Length** with `.length()` or `.size()` (they do the same thing):

```cpp
std::string name = "C++";
std::cout << name.length() << std::endl;  // 3
std::cout << name.size() << std::endl;    // 3
```

**Accessing characters** with `[]`:

```cpp
std::string word = "Hello";
std::cout << word[0] << std::endl;  // H
std::cout << word[4] << std::endl;  // o
```

## Useful String Methods

**`substr(pos, len)`** -- extract a portion of the string:

```cpp
std::string text = "Hello, World!";
std::string sub = text.substr(7, 5);
std::cout << sub << std::endl;  // World
```

**`find(str)`** -- find the position of a substring (returns `std::string::npos` if not found):

```cpp
std::string text = "Hello, World!";
size_t pos = text.find("World");
if (pos != std::string::npos) {
    std::cout << "Found at position " << pos << std::endl;  // Found at position 7
}
```

**`empty()`** -- check if a string is empty:

```cpp
std::string blank = "";
if (blank.empty()) {
    std::cout << "String is empty" << std::endl;
}
```

## String Comparison

Strings in C++ can be compared with the familiar operators: `==`, `!=`, `<`, `>`, `<=`, `>=`. Comparison is **lexicographic** (dictionary order).

```cpp
std::string a = "apple";
std::string b = "banana";

if (a == b) {
    std::cout << "same" << std::endl;
} else {
    std::cout << "different" << std::endl;  // prints this
}

if (a < b) {
    std::cout << a << " comes first" << std::endl;  // apple comes first
}
```

This is a major improvement over C, where you had to use `strcmp()` and couldn't use `==` on character arrays.

## Range-Based for with Strings

Strings work with range-based for loops too, iterating over each character:

```cpp
std::string word = "Hello";
for (char c : word) {
    std::cout << c << std::endl;
}
// Prints: H, e, l, l, o (each on its own line)
```

## C-Strings vs std::string

You may encounter C-style strings in older code or C libraries. They look like this:

```cpp
const char* msg = "Hello";  // pointer to char array
char name[] = "World";      // char array
```

C-strings are null-terminated arrays of `char`. They lack the safety and convenience of `std::string` -- no bounds checking, no automatic memory management, no `+` for concatenation.

**Rule of thumb:** use `std::string` for new code. Convert to/from C-strings only when interfacing with C libraries.
