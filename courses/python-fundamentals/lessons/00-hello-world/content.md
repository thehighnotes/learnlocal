# Hello World & Getting Started

Python is one of the most beginner-friendly programming languages. There is no
boilerplate, no compilation step, and no ceremony. You write code, you run it,
and it works. This lesson introduces the fundamentals: printing output, working
with strings, leaving comments, and reading input from the user.

## Your First Program

Here is the simplest Python program:

```python
print("Hello, World!")
```

That is one line. Run it and you see `Hello, World!` in the terminal. Compare
this to C++ or Java, where you need includes, a class, or a `main` function
just to get started. Python skips all of that.

To run a Python file from the command line:

```bash
python3 hello.py
```

Python reads your file top to bottom, executing each line in order. There is no
`main()` function required (though you can write one later for organization).

## The print() Function

`print()` is a built-in function that sends text to the terminal. You call it
with parentheses and pass the value you want to display:

```python
print("Hello, World!")
print(42)
print(3.14)
```

You can pass multiple arguments separated by commas. By default, Python puts a
space between each one:

```python
print("Name:", "Alice", "Age:", 30)
```

Output: `Name: Alice Age: 30`

### Customizing print()

The `print()` function has two useful optional parameters:

- `sep` controls the separator between arguments (default is a space)
- `end` controls what comes after the printed text (default is a newline)

```python
print("A", "B", "C", sep="-")     # prints: A-B-C
print("Hello", end="")            # prints "Hello" with no newline after
print(" World")                    # continues on the same line
```

When `end=""` is used, the next `print()` continues on the same line. This is
handy when you want to build up a line piece by piece.

## Strings in Python

Strings are sequences of characters enclosed in quotes. Python accepts both
single quotes and double quotes — they are interchangeable:

```python
print("Hello")   # double quotes
print('Hello')   # single quotes — same result
```

Use whichever you prefer. The convention is to pick one style and be consistent.
Double quotes are more common in many codebases, but single quotes are fine too.

If your string contains a quote character, use the other style to avoid escaping:

```python
print("It's a sunny day")    # double quotes around a string with an apostrophe
print('She said "hello"')    # single quotes around a string with double quotes
```

### Triple Quotes

For strings that span multiple lines, use triple quotes (`"""` or `'''`):

```python
print("""Line one
Line two
Line three""")
```

This prints three lines exactly as written. Triple-quoted strings preserve
newlines and indentation inside them.

### Comments

Comments are notes for humans. Python ignores them completely.

**Single-line comments** start with `#`:

```python
# This is a comment
print("Hello")  # This is also a comment
```

Everything after the `#` on that line is ignored. There is no multi-line comment
syntax in Python, but you can comment multiple lines by putting `#` at the start
of each:

```python
# This is line one of a comment
# This is line two of a comment
# This is line three of a comment
```

You might see triple-quoted strings used as multi-line comments:

```python
"""
This looks like a comment, but it is actually
a string that Python evaluates and then discards.
"""
```

This works in practice, but it is technically a string literal, not a comment.
True docstrings (triple-quoted strings right after a function or class definition)
have a special purpose — they document that function or class.

Use comments to explain **why** you did something, not **what** the code does.
Good code is mostly self-explanatory; comments fill in the reasoning.

## Escape Characters

Sometimes you need to include special characters in a string that cannot be
typed directly. Python uses **escape sequences** starting with a backslash `\`:

| Escape | Meaning              | Example output       |
|--------|----------------------|----------------------|
| `\n`   | Newline              | (moves to next line) |
| `\t`   | Tab                  | (horizontal tab)     |
| `\\`   | Literal backslash    | `\`                  |
| `\"`   | Literal double quote | `"`                  |
| `\'`   | Literal single quote | `'`                  |

For example, to print a tab-separated table:

```python
print("Item\tPrice")
print("Apple\t1.50")
```

Output:

```
Item    Price
Apple   1.50
```

You can put multiple escape sequences in a single string:

```python
print("First\tSecond\nThird\tFourth")
```

This prints two lines, each with two tab-separated columns.

If you want to disable escape sequences entirely, use a **raw string** by
prefixing with `r`:

```python
print(r"No \n newline here")  # prints literally: No \n newline here
```

Raw strings are useful for file paths and regular expressions where backslashes
are common.

## Reading Input

The `input()` function reads a line of text from the user:

```python
name = input()
print("Hello,", name)
```

If you run this, Python waits for the user to type something and press Enter.
Whatever they type becomes a string stored in the variable `name`.

You can pass a prompt string to `input()`:

```python
name = input("What is your name? ")
print("Hello,", name)
```

The prompt is displayed before the cursor, so the user knows what to type. Note
the trailing space in the prompt — it gives the user a bit of room.

### Converting Input

`input()` always returns a string, even if the user types a number. To do math
with it, you need to convert:

```python
text = input("Enter a number: ")
number = int(text)          # convert to integer
print(number * 2)
```

Common conversions:

- `int("42")` converts a string to an integer
- `float("3.14")` converts a string to a floating-point number
- `str(42)` converts a number back to a string

If the string cannot be converted (for example, `int("hello")`), Python raises
a `ValueError`. For now, we will assume the user types valid input.

## Checking Your Python Version

Beyond writing scripts, the `python3` command has several useful modes. Before
anything else, you can check which version is installed:

```bash
python3 --version
```

This prints something like `Python 3.12.3`. Knowing your version matters because
different versions support different features.

## Running a Script

The most common way to run Python code is to save it in a `.py` file and pass
it to the interpreter:

```bash
python3 hello.py
```

Python reads the file top to bottom, executing each statement in order. If there
is a syntax error, Python stops and prints an error message showing the line
number and what went wrong.

## One-Liner Execution

The `-c` flag runs a Python statement directly from the command line without
creating a file:

```bash
python3 -c "print(6 * 7)"
```

This prints `42`. The `-c` flag is handy for quick calculations and testing
snippets.
