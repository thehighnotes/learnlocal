# String Methods and Formatting

Strings are one of the most common types you will work with in any programming
language. Python gives you a rich set of built-in methods for transforming,
searching, splitting, and formatting text — all without importing anything.

The key thing to remember: **strings are immutable**. Every string method
returns a *new* string. The original is never modified.

```python
name = "alice"
upper_name = name.upper()
print(name)        # alice  (unchanged)
print(upper_name)  # ALICE  (new string)
```

## Case Conversion

Python provides several methods for changing the case of strings:

| Method         | Description                              | Example                    |
|----------------|------------------------------------------|----------------------------|
| `upper()`      | All characters to uppercase              | `"hello"` -> `"HELLO"`    |
| `lower()`      | All characters to lowercase              | `"HELLO"` -> `"hello"`    |
| `title()`      | First letter of each word capitalized    | `"hello world"` -> `"Hello World"` |
| `capitalize()` | First character of string capitalized    | `"hello world"` -> `"Hello world"` |
| `swapcase()`   | Swap upper and lower case                | `"Hello"` -> `"hELLO"`    |

```python
text = "hello, World!"
print(text.upper())       # HELLO, WORLD!
print(text.lower())       # hello, world!
print(text.title())       # Hello, World!
print(text.capitalize())  # Hello, world!
print(text.swapcase())    # HELLO, wORLD!
```

These methods are especially useful for case-insensitive comparisons:

```python
user_input = "YES"
if user_input.lower() == "yes":
    print("Confirmed")
```

### Stripping Whitespace

The `strip()` method removes leading and trailing whitespace (spaces, tabs,
newlines):

```python
messy = "   hello   "
print(messy.strip())   # "hello"
print(messy.lstrip())  # "hello   "  (left side only)
print(messy.rstrip())  # "   hello"  (right side only)
```

You will use `strip()` constantly when processing user input or reading files,
since extra whitespace is a common source of bugs.

## Searching and Replacing

### Finding Substrings

The `find()` method returns the index of the first occurrence of a substring,
or `-1` if not found:

```python
text = "Hello, World!"
print(text.find("World"))  # 7
print(text.find("Python")) # -1
```

The `count()` method counts how many times a substring appears:

```python
text = "banana"
print(text.count("an"))  # 2
```

You can also check the start or end of a string:

```python
filename = "report.pdf"
print(filename.startswith("report"))  # True
print(filename.endswith(".pdf"))      # True
```

### Replacing Text

The `replace()` method substitutes all occurrences of one substring with
another:

```python
text = "I like cats. I really like cats."
new_text = text.replace("cats", "dogs")
print(new_text)  # I like dogs. I really like dogs.
```

`replace()` is case-sensitive. `"Cats"` and `"cats"` are treated as different
substrings. You can chain multiple replacements:

```python
text = "I like cats. Cats are great."
result = text.replace("cats", "dogs").replace("Cats", "Dogs")
print(result)  # I like dogs. Dogs are great.
```

## Splitting and Joining

### Splitting Strings

`split()` breaks a string into a list of substrings. By default, it splits on
any whitespace:

```python
sentence = "the quick brown fox"
words = sentence.split()
print(words)  # ['the', 'quick', 'brown', 'fox']
```

You can split on a specific separator:

```python
csv_line = "Alice,Bob,Charlie,Diana"
names = csv_line.split(",")
print(names)  # ['Alice', 'Bob', 'Charlie', 'Diana']
```

### Joining Strings

`join()` is the reverse of `split()` — it combines a list of strings into one
string, with a separator between each element:

```python
words = ["Alice", "Bob", "Charlie"]
result = " and ".join(words)
print(result)  # Alice and Bob and Charlie
```

Note the syntax: the separator string calls `.join()`, and the list is the
argument. This often surprises beginners, but it makes sense — the separator
is the "glue" that joins the pieces.

A common pattern is splitting on one separator and joining with another:

```python
csv_line = "Alice,Bob,Charlie"
names = csv_line.split(",")
print(" | ".join(names))  # Alice | Bob | Charlie
```

## String Formatting

Python offers several ways to embed values in strings. The recommended
approach is **f-strings** (formatted string literals), introduced in Python 3.6.

### f-Strings

Prefix the string with `f` and put expressions inside curly braces:

```python
name = "Alice"
age = 30
print(f"{name} is {age} years old")  # Alice is 30 years old
```

f-strings support format specifiers after a colon for controlling output:

```python
price = 49.99
print(f"Price: ${price:.2f}")     # Price: $49.99
print(f"Price: ${price:>10.2f}")  # Price: $     49.99  (right-aligned, width 10)

count = 42
print(f"Count: {count:05d}")      # Count: 00042  (zero-padded, width 5)
```

Common format specifiers:

| Specifier | Meaning                    | Example              |
|-----------|----------------------------|----------------------|
| `:.2f`    | 2 decimal places (float)   | `3.14159` -> `3.14`  |
| `:>10`    | Right-align, width 10      | `"hi"` -> `"        hi"` |
| `:<10`    | Left-align, width 10       | `"hi"` -> `"hi        "` |
| `:05d`    | Zero-pad integer, width 5  | `42` -> `00042`      |

### .format() Method

Before f-strings, the `.format()` method was the standard way:

```python
print("{} scored {}".format("Alice", 95))       # Alice scored 95
print("{name} scored {score}".format(name="Alice", score=95))
```

You will see `.format()` in older code, but f-strings are generally preferred
for new code because they are more readable.

## String Indexing and Slicing

Strings are sequences, just like lists. You can access individual characters
by index and extract substrings with slicing.

### Indexing

```python
word = "Python"
print(word[0])    # P  (first character)
print(word[5])    # n  (last character)
print(word[-1])   # n  (last character, using negative index)
print(word[-2])   # o  (second to last)
```

Negative indices count from the end: `-1` is the last character, `-2` is the
second to last, and so on. Note that `-0` is the same as `0` — it does *not*
give you the last character.

### Slicing

```python
word = "Python"
print(word[0:3])   # Pyt  (characters 0, 1, 2)
print(word[2:])    # thon (from index 2 to the end)
print(word[:3])    # Pyt  (from start to index 3, exclusive)
print(word[-3:])   # hon  (last 3 characters)
```

### Reversing

The slice `[::-1]` reverses a string:

```python
word = "Python"
print(word[::-1])  # nohtyP
```

This works because the third number in a slice is the step. A step of `-1`
means "go backwards through the string."

## Putting It Together

String methods can be chained together because each one returns a new string:

```python
raw_input = "  Hello, World!  "
cleaned = raw_input.strip().lower().replace("world", "python")
print(cleaned)  # hello, python!
```

Here is a more practical example — processing lines from a file:

```python
line = "  Alice, 95, A  "
parts = line.strip().split(", ")
name = parts[0]
score = int(parts[1])
grade = parts[2]
print(f"{name} got {score} ({grade})")  # Alice got 95 (A)
```

In the exercises that follow, you will practice case conversion, find-and-replace,
splitting and joining, f-string formatting, indexing, and combining these tools
to solve a classic string problem.
