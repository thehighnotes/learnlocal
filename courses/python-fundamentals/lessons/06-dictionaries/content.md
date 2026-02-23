# Dictionaries

Imagine a phone book: you look up a person's name and get their phone number.
You do not search through the entries one by one — you go straight to the name
you want. Python **dictionaries** work the same way. They store **key-value
pairs**, letting you look up a value instantly by its key.

Dictionaries are one of the most important data structures in Python. They show
up everywhere: configuration files, database rows, JSON data from web APIs, and
countless everyday programming tasks.

## Creating Dictionaries

Use curly braces `{}` with `key: value` pairs separated by commas:

```python
person = {"name": "Alice", "age": 30, "city": "Paris"}
```

Each key is followed by a colon, then its value. Keys are usually strings, but
they can be any immutable type (strings, numbers, tuples). Values can be
anything.

You can also create dictionaries with the `dict()` constructor:

```python
person = dict(name="Alice", age=30, city="Paris")
```

This syntax only works when the keys are valid Python identifiers (no spaces,
no special characters).

An empty dictionary is created with `{}` or `dict()`:

```python
empty = {}
also_empty = dict()
```

## Accessing Values

Use square brackets with the key to retrieve a value:

```python
person = {"name": "Alice", "age": 30, "city": "Paris"}
print(person["name"])    # Alice
print(person["age"])     # 30
```

If you try to access a key that does not exist, Python raises a `KeyError`:

```python
print(person["email"])   # KeyError: 'email'
```

To avoid this, use the `.get()` method, which returns a default value instead
of crashing:

```python
print(person.get("email", "not set"))  # not set
print(person.get("name", "unknown"))   # Alice (key exists, so default is ignored)
```

If you omit the second argument, `.get()` returns `None` for missing keys:

```python
print(person.get("email"))  # None
```

The `in` operator checks whether a key exists:

```python
if "name" in person:
    print("Has a name")
```

## Dictionary Methods

Dictionaries have several useful methods for inspecting their contents:

| Method     | Returns                                  |
|------------|------------------------------------------|
| `keys()`   | A view of all keys                       |
| `values()` | A view of all values                     |
| `items()`  | A view of all (key, value) tuples        |
| `len(d)`   | The number of key-value pairs            |
| `in`       | Whether a key exists                     |

```python
person = {"name": "Alice", "age": 30}
print(person.keys())    # dict_keys(['name', 'age'])
print(person.values())  # dict_values(['Alice', 30])
print(person.items())   # dict_items([('name', 'Alice'), ('age', 30)])
print(len(person))      # 2
```

The views returned by `keys()`, `values()`, and `items()` are live — they
reflect changes to the dictionary automatically.

## Adding and Updating

To add a new key-value pair, simply assign to a new key:

```python
scores = {"Alice": 85}
scores["Bob"] = 92      # adds new key
print(scores)           # {'Alice': 85, 'Bob': 92}
```

To update an existing key, assign to it again:

```python
scores["Alice"] = 90    # updates existing key
print(scores)           # {'Alice': 90, 'Bob': 92}
```

The `.update()` method merges another dictionary (or keyword arguments) into
the current one:

```python
scores.update({"Charlie": 78, "Alice": 95})
print(scores)  # {'Alice': 95, 'Bob': 92, 'Charlie': 78}
```

To remove a key, use `del`:

```python
del scores["Bob"]
print(scores)  # {'Alice': 95, 'Charlie': 78}
```

The `.pop()` method removes a key and returns its value:

```python
removed = scores.pop("Alice")
print(removed)  # 95
print(scores)   # {'Charlie': 78}
```

## Iterating Over Dictionaries

Looping over a dictionary gives you the keys:

```python
ages = {"Alice": 30, "Bob": 25, "Charlie": 35}

for name in ages:
    print(name)
# Alice
# Bob
# Charlie
```

To get both keys and values, use `.items()`:

```python
for name, age in ages.items():
    print(f"{name} is {age}")
# Alice is 30
# Bob is 25
# Charlie is 35
```

You can also loop over just the values:

```python
for age in ages.values():
    print(age)
```

Since Python 3.7, dictionaries maintain **insertion order** — items come out in
the order you added them. This is a language guarantee, not an implementation
detail.

## Practical Pattern: Counting

One of the most common uses for dictionaries is counting occurrences. Here is
the pattern:

```python
text = "apple banana apple cherry banana apple"
counts = {}

for word in text.split():
    if word in counts:
        counts[word] += 1
    else:
        counts[word] = 1

print(counts)  # {'apple': 3, 'banana': 2, 'cherry': 1}
```

The `.get()` method makes this more concise:

```python
counts = {}
for word in text.split():
    counts[word] = counts.get(word, 0) + 1
```

This works because `counts.get(word, 0)` returns `0` for new words and the
current count for existing ones. Either way, adding 1 gives the correct new
count.

You will use this pattern frequently — for counting characters, tallying votes,
grouping data, and many other tasks. In the exercises that follow, you will
practice creating dictionaries, using their methods, and building a word
counter from scratch.
