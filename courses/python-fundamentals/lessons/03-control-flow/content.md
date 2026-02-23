# Control Flow

So far every program you have written executes every line, top to bottom, once.
Real programs need to make decisions and repeat work. Control flow gives you
that power: conditionals let your program choose between paths, and loops let
it repeat blocks of code.

Python uses **indentation** to define blocks, not curly braces. This is not just
a style choice -- it is part of the language syntax. If your indentation is
wrong, your program will not run.

## if, elif, else

The `if` statement runs a block of code only when a condition is true:

```python
temperature = 35

if temperature > 30:
    print("It's hot outside")
```

The syntax has three pieces: the `if` keyword, a condition, and a colon. The
indented lines below it form the body -- they run only when the condition is
true. Convention is 4 spaces of indentation.

Add `else` to handle the opposite case:

```python
temperature = 15

if temperature > 30:
    print("It's hot outside")
else:
    print("It's not too hot")
```

Python considers several values as "falsy" (treated as False):
- `False`, `0`, `0.0`
- `""` (empty string), `[]` (empty list), `None`

Everything else is "truthy." This means you can write:

```python
name = ""
if name:
    print(f"Hello, {name}")
else:
    print("No name provided")
```

## The elif Chain

When you have more than two cases, use `elif` (short for "else if"):

```python
score = 85

if score >= 90:
    print("A")
elif score >= 80:
    print("B")
elif score >= 70:
    print("C")
elif score >= 60:
    print("D")
else:
    print("F")
```

Python checks each condition from top to bottom. The first one that is true
has its block executed, and the rest are skipped entirely. This is why
**order matters** -- if you put `score >= 60` first, every passing score
would get a "D."

The `else` at the end is optional. It acts as a catch-all for anything that
did not match the conditions above it.

You can have as many `elif` branches as you need. There is no `switch` or
`match` statement in older Python. (Python 3.10 added `match/case`, but
`if/elif/else` remains the standard approach for most code.)

## while Loops

A `while` loop repeats its body as long as a condition is true:

```python
count = 1
while count <= 5:
    print(count)
    count += 1
```

This prints 1 through 5. The flow is:
1. Check the condition (`count <= 5`).
2. If true, run the body.
3. Go back to step 1.
4. If false, skip the body and continue after the loop.

Be careful: if the condition never becomes false, you get an **infinite loop**:

```python
# DO NOT RUN THIS -- infinite loop!
# while True:
#     print("forever")
```

Common while loop patterns:

```python
# Counting down
n = 5
while n > 0:
    print(n)
    n -= 1
print("Go!")

# Accumulating a sum
total = 0
i = 1
while i <= 100:
    total += i
    i += 1
print(total)   # 5050
```

## for Loops and range()

The `for` loop iterates over a sequence of values. The most common sequence is
`range()`:

```python
for i in range(5):
    print(i)
# Prints: 0, 1, 2, 3, 4
```

`range(n)` produces integers from 0 up to (but not including) n. This "up to
but not including" convention is consistent throughout Python.

`range()` accepts up to three arguments:

```python
range(5)          # 0, 1, 2, 3, 4
range(2, 7)       # 2, 3, 4, 5, 6
range(1, 10, 2)   # 1, 3, 5, 7, 9  (step of 2)
range(10, 0, -1)  # 10, 9, 8, 7, 6, 5, 4, 3, 2, 1  (counting down)
```

The general form is `range(start, stop, step)`:
- `start` is the first value (default 0)
- `stop` is the boundary (never included)
- `step` is the increment (default 1)

For loops are preferred over while loops when you know how many iterations you
need. They are cleaner and less prone to off-by-one errors:

```python
# Print numbers 1 to 5 -- for loop version
for i in range(1, 6):
    print(i)
```

## FizzBuzz

FizzBuzz is a classic programming exercise that combines loops with conditionals.
The rules are simple:

- For each number from 1 to N:
  - If divisible by both 3 and 5, print "FizzBuzz"
  - If divisible by 3, print "Fizz"
  - If divisible by 5, print "Buzz"
  - Otherwise, print the number

```python
for i in range(1, 16):
    if i % 3 == 0 and i % 5 == 0:
        print("FizzBuzz")
    elif i % 3 == 0:
        print("Fizz")
    elif i % 5 == 0:
        print("Buzz")
    else:
        print(i)
```

The key insight is **condition ordering**: check the most specific case (both 3
and 5) before the individual cases. If you check `i % 3 == 0` first, the
number 15 would print "Fizz" instead of "FizzBuzz."

## break and continue

Two keywords let you alter the normal flow of a loop:

**`break`** exits the loop immediately:

```python
for i in range(1, 100):
    if i * i > 50:
        print(f"{i} squared exceeds 50")
        break
# Prints: 8 squared exceeds 50
```

**`continue`** skips the rest of the current iteration and moves to the next:

```python
for i in range(1, 11):
    if i % 2 == 0:
        continue   # skip even numbers
    print(i)
# Prints: 1, 3, 5, 7, 9
```

Use `break` when you are searching for something and want to stop as soon as
you find it. Use `continue` when you want to skip certain items but keep going
through the rest. Both work in `for` and `while` loops.

You can combine them:

```python
for i in range(1, 20):
    if i % 2 == 0:
        continue      # skip even numbers
    if i > 10:
        break         # stop after 10
    print(i)
# Prints: 1, 3, 5, 7, 9
```

## Iterating Over Strings

Strings are sequences in Python, which means you can loop over them directly:

```python
for char in "Hello":
    print(char)
# Prints each character on its own line: H, e, l, l, o
```

If you need both the index and the character, use `enumerate()`:

```python
for index, char in enumerate("Python"):
    print(f"{index}: {char}")
# 0: P
# 1: y
# 2: t
# ...
```

This works with any iterable, not just strings. You will see `enumerate()` used
frequently with lists in a later lesson.

## Nested Loops

You can put a loop inside another loop. The inner loop runs completely for each
iteration of the outer loop:

```python
for row in range(1, 4):
    for col in range(1, 4):
        print(f"{row}x{col}={row*col}", end="\t")
    print()  # newline after each row
```

Output:
```
1x1=1   1x2=2   1x3=3
2x1=2   2x2=4   2x3=6
3x1=3   3x2=6   3x3=9
```

A common use of nested loops is building text patterns:

```python
for row in range(1, 5):
    print("*" * row)
```

Output:
```
*
**
***
****
```

This uses string repetition from the previous lesson instead of an inner loop.
Often there is a simpler way to achieve the same result -- but understanding
nested loops is essential because many real problems (grids, tables, matrices)
require them.

In the exercises that follow, you will practice each of these control flow
structures and combine them together.
