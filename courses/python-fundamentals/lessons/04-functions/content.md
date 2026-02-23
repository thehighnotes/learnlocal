# Functions

As your programs grow, you will find yourself writing the same logic more than
once. Functions solve this problem. A function is a named block of code that you
define once and call whenever you need it. Functions make programs shorter, easier
to read, easier to test, and easier to change. If you fix a bug in a function,
every place that calls it gets the fix automatically.

Python comes with many built-in functions like `print()`, `len()`, and `input()`.
In this lesson you will learn to write your own.

## Defining a Function

Use the `def` keyword, followed by a name, parentheses, and a colon. The
function body is indented:

```python
def greet():
    print("Hello, World!")
```

This defines a function called `greet`. It does not run yet. To run it, you
**call** it by writing its name followed by parentheses:

```python
greet()    # prints: Hello, World!
```

A few rules:
- Function names follow the same rules as variable names: lowercase, underscores
  for spaces, no leading digits.
- The body must be indented (typically four spaces).
- A function must be defined before it is called. Python reads top to bottom, so
  put definitions above the code that uses them.

```python
def say_goodbye():
    print("Goodbye!")

say_goodbye()    # works
say_goodbye()    # you can call it as many times as you want
```

## Parameters and Arguments

Most functions need input to do useful work. You specify **parameters** in the
parentheses of the function definition. When you call the function, you pass
**arguments** that fill those parameters:

```python
def greet(name):
    print(f"Hello, {name}!")

greet("Alice")    # prints: Hello, Alice!
greet("Bob")      # prints: Hello, Bob!
```

The parameter `name` acts like a local variable inside the function. Each call
gets its own value.

Functions can take multiple parameters, separated by commas:

```python
def add(a, b):
    print(a + b)

add(3, 7)     # prints: 10
add(10, 20)   # prints: 30
```

Arguments are matched to parameters by position. The first argument goes to the
first parameter, and so on.

## Return Values

Functions that only print are limited. Usually you want a function to compute
something and **return** the result so you can use it elsewhere:

```python
def add(a, b):
    return a + b

result = add(3, 7)
print(result)    # prints: 10
```

The `return` statement does two things: it sends a value back to the caller, and
it immediately exits the function. Any code after `return` in the same block does
not run.

```python
def absolute(n):
    if n < 0:
        return -n
    return n

print(absolute(-5))    # prints: 5
print(absolute(3))     # prints: 3
```

If a function has no `return` statement (or just `return` with no value), it
returns `None` by default:

```python
def do_nothing():
    pass

result = do_nothing()
print(result)    # prints: None
```

## Default Parameters

You can give a parameter a default value. If the caller does not provide that
argument, the default is used:

```python
def power(base, exp=2):
    return base ** exp

print(power(3))       # uses exp=2, prints: 9
print(power(2, 10))   # uses exp=10, prints: 1024
```

Default parameters must come after non-default parameters in the definition.
This is valid:

```python
def greet(name, greeting="Hello"):
    print(f"{greeting}, {name}!")
```

This is **not** valid:

```python
def greet(greeting="Hello", name):    # SyntaxError
    print(f"{greeting}, {name}!")
```

A common gotcha: never use a mutable object (like a list) as a default value.
The default is created once when the function is defined, not each time it is
called. Use `None` as a sentinel instead:

```python
def append_item(item, lst=None):
    if lst is None:
        lst = []
    lst.append(item)
    return lst
```

## Returning Multiple Values

Python functions can return multiple values by separating them with commas.
Under the hood, this creates a tuple:

```python
def min_max(numbers):
    return min(numbers), max(numbers)

low, high = min_max([4, 1, 7, 3, 9])
print(low)     # prints: 1
print(high)    # prints: 9
```

The line `low, high = min_max(...)` is called **tuple unpacking**. The first
returned value goes to `low`, the second to `high`. You could also capture
the tuple directly:

```python
result = min_max([4, 1, 7, 3, 9])
print(result)    # prints: (1, 9)
```

This is useful whenever a function naturally produces more than one piece of
information, like bounds, coordinates, or a value-and-status pair.

## Variable Scope

Variables created inside a function are **local** to that function. They do not
exist outside it:

```python
def set_x():
    x = 10
    print(x)

set_x()      # prints: 10
# print(x)   # NameError: name 'x' is not defined
```

If a variable with the same name exists outside the function, the function
creates its own separate copy:

```python
x = 5

def set_x():
    x = 10
    print(x)

set_x()      # prints: 10
print(x)     # prints: 5 — the global x is unchanged
```

Python looks up variable names using the **LEGB rule**: Local, Enclosing,
Global, Built-in. It searches in that order and uses the first match.

You *can* modify a global variable from inside a function using the `global`
keyword, but this is generally discouraged because it makes code harder to
follow:

```python
count = 0

def increment():
    global count
    count += 1

increment()
print(count)    # prints: 1
```

The better practice is to pass values in as parameters and return results.

## Recursion

A function can call itself. This is called **recursion**. Every recursive
function needs two parts:

1. **Base case** — a condition that stops the recursion
2. **Recursive case** — the function calls itself with a smaller or simpler input

The classic example is factorial. The factorial of n (written n!) is
n * (n-1) * (n-2) * ... * 1. By definition, 0! = 1 and 1! = 1.

```python
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print(factorial(5))    # prints: 120
```

Here is how the calls unfold:

```
factorial(5)
  = 5 * factorial(4)
  = 5 * 4 * factorial(3)
  = 5 * 4 * 3 * factorial(2)
  = 5 * 4 * 3 * 2 * factorial(1)
  = 5 * 4 * 3 * 2 * 1
  = 120
```

Without the base case (`if n <= 1: return 1`), the function would call itself
forever until Python raises a `RecursionError`. Python has a default recursion
limit of 1000 calls deep, which protects you from infinite recursion.

Recursion is elegant for problems that have a naturally recursive structure:
factorials, Fibonacci numbers, tree traversals, and directory walkers. For
simple loops, a `for` or `while` loop is usually clearer and faster.

## Command-Line Arguments

So far, your programs get input from `input()` or from hardcoded values.
There is a third way: **command-line arguments** — values passed to your
script when you run it.

Python provides these through `sys.argv`, a list in the `sys` module:

```python
import sys
print(sys.argv)
```

If you run `python3 script.py Alice 30`, then `sys.argv` is:

```python
['script.py', 'Alice', '30']
```

- `sys.argv[0]` is always the script name
- `sys.argv[1]` is the first argument
- `sys.argv[2]` is the second argument, and so on

A practical example:

```python
import sys
name = sys.argv[1]
print(f"Hello, {name}!")
```

```bash
python3 greet.py Alice
```

Output: `Hello, Alice!`

Command-line arguments let you make scripts configurable without changing the
code. They are strings by default — use `int()` or `float()` to convert when
needed.

In the exercises that follow, you will define your own functions, work with
parameters and return values, explore scope, and write a recursive function.
