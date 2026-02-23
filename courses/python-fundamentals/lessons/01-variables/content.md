# Variables and Types

Every program needs to store data. In Python, you store data in **variables** —
names that refer to values. Unlike C++ or Java, Python does not require you to
declare a type. You simply assign a value, and Python figures out the rest.

## Creating Variables

To create a variable, use the assignment operator `=`:

```python
age = 25
name = "Alice"
price = 9.99
```

That is it. No `int`, no `var`, no `let`. The variable comes into existence the
moment you assign to it. If you assign to the same name again, it gets the new
value.

### Naming Rules

Variable names in Python must follow these rules:

- Start with a letter or underscore (`_`)
- Contain only letters, digits, and underscores
- Cannot be a Python keyword (`if`, `for`, `class`, `return`, etc.)
- Case sensitive: `age`, `Age`, and `AGE` are three different variables

The convention in Python is **snake_case** for variable names:

```python
first_name = "Alice"
max_score = 100
is_active = True
```

This is different from C++ or Java, which commonly use camelCase. In Python,
snake_case is the standard and virtually all Python code follows it.

## Data Types

Python has four fundamental types you will use constantly:

| Type    | Description          | Example            |
|---------|----------------------|--------------------|
| `int`   | Whole numbers        | `42`, `-7`, `0`    |
| `float` | Decimal numbers      | `3.14`, `-0.5`     |
| `str`   | Text (strings)       | `"hello"`, `'hi'`  |
| `bool`  | True or false        | `True`, `False`    |

### int

Integers store whole numbers with no decimal point. Unlike C++, Python integers
have **no size limit** — they can be as large as your memory allows:

```python
small = 42
big = 10 ** 100    # 10 to the power of 100 — perfectly fine
```

### float

Floats store numbers with a decimal point. Under the hood, Python uses 64-bit
IEEE 754 floating point (equivalent to C++'s `double`):

```python
pi = 3.14159
temperature = -40.0
```

Be aware that floating-point arithmetic can produce small rounding errors:

```python
print(0.1 + 0.2)  # prints 0.30000000000000004
```

This is not a Python bug — it is how floating-point numbers work in every
language.

### str

Strings are sequences of characters. They can use single or double quotes:

```python
greeting = "Hello"
name = 'Alice'
```

Strings are **immutable** in Python — once created, they cannot be changed.
Operations on strings always produce a new string.

### bool

Booleans have exactly two values: `True` and `False` (note the capital letters).
They are used in conditions and comparisons:

```python
active = True
finished = False
```

## Checking Types

The `type()` function tells you what type a value or variable is:

```python
x = 42
print(type(x))    # <class 'int'>

y = "hello"
print(type(y))    # <class 'str'>
```

The output format `<class 'int'>` tells you the type. This is useful when
debugging — if something behaves unexpectedly, check its type first.

You can also use `isinstance()` to check whether a value is a specific type:

```python
x = 42
print(isinstance(x, int))    # True
print(isinstance(x, str))    # False
```

`isinstance()` returns a boolean and is often used in conditional logic.

## Dynamic Typing

Python is **dynamically typed**. A variable can hold any type, and you can
reassign it to a completely different type at any time:

```python
x = 10        # x is an int
x = "ten"     # now x is a str — perfectly legal
x = [1, 2]    # now x is a list
```

In C++ or Java, this would be a compile error. In Python, the variable is just
a name that points to a value. When you reassign it, the name simply points to
a new value.

This flexibility is powerful but can cause bugs if you are not careful. If a
function expects an integer and you accidentally pass it a string, Python will
not warn you until the code runs and crashes.

## String Concatenation

The `+` operator joins strings together:

```python
first = "Hello"
second = "World"
print(first + ", " + second + "!")  # Hello, World!
```

However, you **cannot** concatenate a string and a number directly:

```python
age = 25
print("Age: " + age)   # TypeError!
```

This raises a `TypeError` because Python does not implicitly convert numbers to
strings. You must convert explicitly:

```python
age = 25
print("Age: " + str(age))   # Age: 25
```

The `str()` function converts any value to its string representation.

## f-Strings

f-strings (formatted string literals) are the modern way to embed values in
strings. Prefix the string with `f` and put expressions inside curly braces:

```python
name = "Alice"
age = 30
print(f"{name} is {age} years old")  # Alice is 30 years old
```

Any valid Python expression works inside the braces:

```python
x = 10
print(f"Double: {x * 2}")   # Double: 20
print(f"Type: {type(x)}")   # Type: <class 'int'>
```

f-strings are generally preferred over string concatenation because they are
more readable and handle type conversion automatically — no need to call
`str()` on numbers.

### Formatting Numbers

f-strings support format specifiers after a colon:

```python
pi = 3.14159
print(f"Pi is approximately {pi:.2f}")  # Pi is approximately 3.14
```

The `.2f` means "2 decimal places, fixed-point notation." You will use this
more as you progress, but it is good to know it exists.

## Multiple Assignment

Python lets you assign multiple variables in a single line:

```python
x, y, z = 1, 2, 3
```

This is equivalent to:

```python
x = 1
y = 2
z = 3
```

You can also assign the same value to multiple variables:

```python
a = b = c = 0
```

A classic use of multiple assignment is swapping two variables:

```python
a, b = b, a    # swaps a and b — no temporary variable needed
```

This works because Python evaluates the entire right side before assigning to
the left side.

## Putting It Together

Here is a short program that uses several variable types:

```python
count = 3
average = 4.5
grade = "B"
passed = True

print(f"Count: {count}")
print(f"Average: {average}")
print(f"Grade: {grade}")
print(f"Passed: {passed}")
print(f"Type of count: {type(count)}")
```

In the exercises that follow, you will practice creating variables, checking
types, using f-strings, and exploring Python's dynamic typing.
