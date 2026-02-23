# Operators and Expressions

Everything useful a program does comes down to expressions -- combinations of
values and operators that produce a result. Python gives you a clean, readable
set of operators for arithmetic, comparison, logic, and more. This lesson covers
all the operators you will use daily.

## Arithmetic Operators

Python supports the standard math operations you would expect, plus a few extras:

```python
print(3 + 5)     # 8   addition
print(10 - 4)    # 6   subtraction
print(6 * 7)     # 42  multiplication
print(15 / 4)    # 3.75  division (always returns float)
print(2 ** 10)   # 1024  exponentiation (power)
```

One thing that catches beginners: the `/` operator **always returns a float**,
even when dividing evenly:

```python
print(10 / 2)    # 5.0, not 5
print(type(10 / 2))  # <class 'float'>
```

This is different from many other languages where integer division returns an
integer. Python made this choice deliberately to avoid subtle bugs.

Operator precedence follows standard math rules. Exponentiation binds tightest,
then multiplication/division, then addition/subtraction. Use parentheses to
make your intent clear:

```python
result = 2 + 3 * 4     # 14, not 20 (multiplication first)
result = (2 + 3) * 4   # 20 (parentheses override precedence)
```

## Floor Division and Modulo

Two operators that deserve special attention:

```python
print(17 // 5)   # 3   floor division (rounds down to nearest integer)
print(17 % 5)    # 2   modulo (remainder after division)
```

**Floor division `//`** drops the fractional part, always rounding toward
negative infinity:

```python
print(17 // 5)    # 3
print(-17 // 5)   # -4  (rounds toward negative infinity, not toward zero)
print(7 // 2)     # 3
```

**Modulo `%`** gives the remainder. It has many practical uses:

```python
# Check if a number is even or odd
number = 14
print(number % 2)   # 0 means even, 1 means odd

# Extract digits
print(12345 % 10)   # 5 (last digit)

# Wrap around (clock arithmetic)
hour = 23
print((hour + 3) % 24)  # 2 (wraps around midnight)
```

Floor division and modulo work together. For any integers `a` and `b`:
`a == (a // b) * b + (a % b)`. This identity always holds.

## Comparison Operators

Comparison operators compare two values and return `True` or `False`:

```python
x = 10
y = 20

print(x == y)   # False  (equal to)
print(x != y)   # True   (not equal to)
print(x < y)    # True   (less than)
print(x > y)    # False  (greater than)
print(x <= y)   # True   (less than or equal to)
print(x >= y)   # False  (greater than or equal to)
```

A common mistake is using `=` (assignment) when you mean `==` (comparison).
Python will usually catch this as a syntax error in conditions.

Python supports **chained comparisons**, which is unusual and very readable:

```python
age = 25
print(18 <= age <= 65)   # True (is age between 18 and 65?)

x = 5
print(1 < x < 10)       # True (is x between 1 and 10, exclusive?)
```

This is equivalent to `18 <= age and age <= 65`, but cleaner.

## Logical Operators

Python uses **English words** for logical operators, not symbols:

```python
a = True
b = False

print(a and b)   # False (both must be True)
print(a or b)    # True  (at least one must be True)
print(not a)     # False (flips True to False)
```

Here is the truth table:

| `a`   | `b`   | `a and b` | `a or b` | `not a` |
|-------|-------|-----------|----------|---------|
| True  | True  | True      | True     | False   |
| True  | False | False     | True     | False   |
| False | True  | False     | True     | True    |
| False | False | False     | False    | True    |

Python uses **short-circuit evaluation**: it stops evaluating as soon as the
result is determined.

```python
# If the first operand of "and" is False, the second is never evaluated
False and print("This never runs")

# If the first operand of "or" is True, the second is never evaluated
True or print("This never runs either")
```

Logical operators are most commonly used in `if` statements, which you will
learn in the next lesson.

## String Operators

Strings support two operators that reuse arithmetic symbols:

**Concatenation `+`** joins strings end-to-end:

```python
first = "Hello"
second = "World"
greeting = first + " " + second
print(greeting)   # Hello World
```

**Repetition `*`** repeats a string a given number of times:

```python
line = "-" * 30
print(line)   # ------------------------------

print("ha" * 3)   # hahaha
```

You **cannot** add a string and a number directly:

```python
# This causes a TypeError:
# print("Age: " + 25)

# Convert the number to a string first:
print("Age: " + str(25))   # Age: 25
```

As you learned in the previous lesson, f-strings are usually a better choice
than concatenation for mixing strings and values.

## Augmented Assignment

When you want to update a variable using its current value, Python provides
shorthand operators:

```python
score = 100
score += 10    # same as: score = score + 10  (now 110)
score -= 5     # same as: score = score - 5   (now 105)
score *= 2     # same as: score = score * 2   (now 210)
score /= 3     # same as: score = score / 3   (now 70.0)
score //= 2    # same as: score = score // 2  (now 35.0)
score **= 2    # same as: score = score ** 2  (now 1225.0)
score %= 100   # same as: score = score % 100 (now 25.0)
```

These are called **augmented assignment** operators. They are a convenience --
they do not do anything you could not do with the long form. But they make code
more concise and less error-prone (you only type the variable name once).

Augmented assignment works with strings too:

```python
message = "Hello"
message += " World"   # message is now "Hello World"
```

Note: Python does not have `++` or `--` operators like C or Java. Use `x += 1`
and `x -= 1` instead.

## Type Casting

Sometimes you need to convert between types. Python provides built-in functions
for this:

```python
# String to integer
age = int("25")       # 25

# String to float
price = float("9.99") # 9.99

# Number to string
text = str(42)         # "42"

# Float to integer (truncates, does not round)
whole = int(3.9)       # 3

# Integer to float
decimal = float(7)     # 7.0

# Anything to boolean
print(bool(0))         # False
print(bool(42))        # True
print(bool(""))        # False (empty string)
print(bool("hello"))   # True  (non-empty string)
```

The most common use of type casting is converting user input. The `input()`
function always returns a string, so you must convert it if you need a number:

```python
text = input("Enter a number: ")   # returns a string like "42"
number = int(text)                  # convert to integer
doubled = number * 2               # now you can do math
print(doubled)                     # 84
```

A common shorthand combines these into one line:

```python
number = int(input("Enter a number: "))
```

Be careful: if the string cannot be converted, Python raises a `ValueError`:

```python
int("hello")   # ValueError: invalid literal for int()
int("3.14")    # ValueError: cannot convert float string to int directly
```

To convert a float string to an integer, convert through float first:

```python
int(float("3.14"))   # 3
```

In the exercises that follow, you will practice every operator covered in this
lesson.
