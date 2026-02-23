# Lists and Tuples

So far, every variable you have used holds a single value: one number, one
string, one boolean. But programs constantly deal with **collections** of data:
a list of names, a series of scores, a sequence of coordinates. Python provides
two built-in sequence types for this: **lists** and **tuples**.

Lists are mutable (you can change them after creation). Tuples are immutable
(once created, they cannot be changed). This lesson covers both, starting with
lists since you will use them far more often.

## Creating Lists

A list is an ordered collection of values enclosed in square brackets:

```python
numbers = [10, 20, 30, 40, 50]
names = ["Alice", "Bob", "Charlie"]
empty = []
```

Lists can hold any type, and you can mix types in a single list (though this is
uncommon in practice):

```python
mixed = [1, "hello", 3.14, True]
```

You can also create a list from other sequences using the `list()` function:

```python
letters = list("hello")    # ['h', 'e', 'l', 'l', 'o']
nums = list(range(5))      # [0, 1, 2, 3, 4]
```

## Indexing and Access

List elements are accessed by **index**, starting from zero:

```python
fruits = ["apple", "banana", "cherry"]
print(fruits[0])    # prints: apple
print(fruits[1])    # prints: banana
print(fruits[2])    # prints: cherry
```

Negative indices count from the end:

```python
print(fruits[-1])   # prints: cherry (last element)
print(fruits[-2])   # prints: banana (second to last)
```

Use `len()` to get the number of elements:

```python
print(len(fruits))    # prints: 3
```

If you access an index that does not exist, Python raises an `IndexError`:

```python
# print(fruits[5])   # IndexError: list index out of range
```

You can iterate over a list with a `for` loop:

```python
for fruit in fruits:
    print(fruit)
```

This prints each element on its own line.

## List Methods

Lists have many built-in methods that modify them **in place** (they change the
original list rather than returning a new one):

```python
fruits = ["apple", "banana"]

fruits.append("cherry")       # add to end: ["apple", "banana", "cherry"]
fruits.insert(0, "avocado")   # insert at index 0: ["avocado", "apple", "banana", "cherry"]
fruits.remove("banana")       # remove first occurrence: ["avocado", "apple", "cherry"]
last = fruits.pop()           # remove and return last: "cherry", list is now ["avocado", "apple"]
```

Other useful methods:

| Method             | Description                                      |
|--------------------|--------------------------------------------------|
| `list.extend(x)`  | Append all elements from iterable x              |
| `list.index(val)` | Return index of first occurrence of val          |
| `list.count(val)` | Count how many times val appears                 |
| `list.reverse()`  | Reverse the list in place                        |
| `list.clear()`    | Remove all elements                              |

Note the difference between `append` and `extend`:

```python
a = [1, 2]
a.append([3, 4])    # [1, 2, [3, 4]]  — adds the list as a single element

b = [1, 2]
b.extend([3, 4])    # [1, 2, 3, 4]    — adds each element individually
```

## Slicing

Slicing extracts a portion of a list. The syntax is `list[start:stop]`, where
`start` is inclusive and `stop` is exclusive:

```python
nums = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

print(nums[0:3])     # [0, 1, 2]     — first three elements
print(nums[7:10])    # [7, 8, 9]     — last three elements
print(nums[:3])      # [0, 1, 2]     — omit start means "from the beginning"
print(nums[7:])      # [7, 8, 9]     — omit stop means "to the end"
```

You can also provide a **step** as a third value, `list[start:stop:step]`:

```python
print(nums[::2])     # [0, 2, 4, 6, 8]   — every other element
print(nums[1::2])    # [1, 3, 5, 7, 9]   — every other, starting at index 1
print(nums[::-1])    # [9, 8, 7, ..., 0]  — reversed copy
```

Slicing always returns a **new list**. The original is not modified.

Practical uses of slicing:
- First N elements: `items[:n]`
- Last N elements: `items[-n:]`
- Copy a list: `items[:]`
- Reverse: `items[::-1]`

## List Comprehensions

A **list comprehension** is a concise way to create a new list by transforming
or filtering an existing sequence:

```python
squares = [x ** 2 for x in range(1, 6)]
print(squares)    # [1, 4, 9, 16, 25]
```

This is equivalent to:

```python
squares = []
for x in range(1, 6):
    squares.append(x ** 2)
```

You can add a condition to filter elements:

```python
evens = [x for x in range(10) if x % 2 == 0]
print(evens)    # [0, 2, 4, 6, 8]
```

And you can transform and filter at the same time:

```python
words = ["hello", "WORLD", "Python", "AI"]
lower_long = [w.lower() for w in words if len(w) > 2]
print(lower_long)    # ['hello', 'world', 'python']
```

List comprehensions are a Python hallmark. They are more readable than the
equivalent `for` loop for simple transformations, but for complex logic a
regular loop is often clearer.

## Tuples

A **tuple** is like a list, but immutable. Once created, you cannot add, remove,
or change its elements:

```python
point = (3, 7)
print(point[0])    # prints: 3
print(point[1])    # prints: 7
```

Attempting to modify a tuple raises a `TypeError`:

```python
# point[0] = 5    # TypeError: 'tuple' object does not support item assignment
```

### When to Use Tuples vs Lists

- Use **lists** when you have a collection of similar items that may change
  (a list of scores, a list of filenames).
- Use **tuples** when you have a fixed group of related values that should not
  change (a coordinate pair, an RGB color, a database row).

### Tuple Unpacking

You can assign tuple elements to individual variables in one line:

```python
point = (3, 7)
x, y = point
print(f"x={x}, y={y}")    # prints: x=3, y=7
```

This works with any number of elements:

```python
name, age, city = ("Alice", 30, "Portland")
```

Tuple unpacking is also how functions return multiple values (as you saw in
the Functions lesson).

### Creating Tuples

Parentheses are optional for tuple creation. The comma is what makes it a tuple:

```python
a = (1, 2, 3)    # tuple with parentheses
b = 1, 2, 3      # also a tuple
c = (42,)         # single-element tuple — the comma is required
d = ()            # empty tuple
```

Without the trailing comma, `(42)` is just the integer 42 in parentheses, not a
tuple.

## Sorting

Python provides two ways to sort:

**`sorted()`** returns a **new** sorted list, leaving the original unchanged:

```python
words = ["banana", "apple", "cherry", "date"]
result = sorted(words)
print(result)    # ['apple', 'banana', 'cherry', 'date']
print(words)     # ['banana', 'apple', 'cherry', 'date'] — unchanged
```

**`.sort()`** sorts a list **in place** and returns `None`:

```python
words.sort()
print(words)     # ['apple', 'banana', 'cherry', 'date'] — modified
```

Both accept a `key` parameter for custom sorting and a `reverse` parameter:

```python
words = ["banana", "apple", "cherry", "date"]
by_length = sorted(words, key=len)
print(by_length)    # ['date', 'apple', 'banana', 'cherry']

descending = sorted(words, reverse=True)
print(descending)   # ['date', 'cherry', 'banana', 'apple']
```

The `key` parameter takes a function that is called on each element. The
elements are sorted by the return values of that function. `len` sorts by
string length. You could also use `str.lower` to sort case-insensitively.

In the exercises that follow, you will create lists, use methods to modify them,
slice them, build new lists with comprehensions, work with tuples, and sort.
