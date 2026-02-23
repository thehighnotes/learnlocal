# Vectors and Math Foundations

Artificial intelligence is, at its core, math on lists of numbers. Every image,
every sentence, every recommendation you see online — the AI behind it represents
that data as lists of numbers and performs arithmetic to make decisions. These
lists of numbers are called **vectors**.

This lesson teaches you the fundamental operations that power all of AI. If you
can add two lists together, compute a dot product, and multiply a matrix by a
vector, you have the tools to understand everything from search engines to
language models.

No libraries needed. Just Python lists and basic arithmetic.

## Vectors: Lists of Numbers

A **vector** is an ordered list of numbers. In Python, we represent vectors as
plain lists:

```python
position = [3, 7]         # 2D vector (x, y)
color = [255, 128, 0]     # 3D vector (red, green, blue)
word = [0.2, -0.5, 0.8]   # word embedding (meaning as numbers)
```

The numbers in a vector are called its **components** or **elements**. The
number of elements is the vector's **dimension**. A 3D vector has 3 numbers.

In AI, vectors can have hundreds or thousands of dimensions. An image might
become a vector of 50,000 pixel values. A word might become a vector of 300
numbers representing its meaning. The math stays the same regardless of size.

### Vector Addition

Adding two vectors means adding their corresponding elements:

```python
a = [1, 2, 3]
b = [4, 5, 6]
result = [a[i] + b[i] for i in range(len(a))]
# result = [5, 7, 9]
```

Both vectors must have the same length. You cannot add a 2D vector to a 3D
vector.

Vector addition has a geometric meaning: if each vector is an arrow, adding
them places one arrow at the tip of the other. The result points from the
start of the first to the tip of the second.

## The Dot Product

The **dot product** is the single most important operation in AI. It multiplies
corresponding elements and sums the results:

```python
a = [1, 2, 3]
b = [4, 5, 6]
dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2]   # 4 + 10 + 18 = 32
```

Or as a loop:

```python
dot = sum(a[i] * b[i] for i in range(len(a)))
```

The dot product tells you how **similar** two vectors are. If both vectors
point in the same direction, the dot product is large and positive. If they
point in opposite directions, it is large and negative. If they are
perpendicular (at right angles), the dot product is zero.

This one operation powers:
- Neural network computations (weighted sums)
- Search engines (similarity between query and documents)
- Recommendation systems (matching users to items)

## Magnitude

The **magnitude** (or length or norm) of a vector measures how "big" it is:

```python
import math

a = [3, 4]
mag = math.sqrt(a[0]**2 + a[1]**2)   # sqrt(9 + 16) = sqrt(25) = 5.0
```

The general formula: square each element, sum them, take the square root.

```python
mag = math.sqrt(sum(x**2 for x in a))
```

You will recognize this as the Pythagorean theorem extended to any number of
dimensions. A vector `[3, 4]` has magnitude 5 — the same as a right triangle
with sides 3 and 4 having hypotenuse 5.

## Normalization

**Normalizing** a vector scales it to have magnitude 1 (a "unit vector"). You
divide each element by the magnitude:

```python
a = [3, 4]
mag = 5.0
normalized = [x / mag for x in a]   # [0.6, 0.8]
```

Why normalize? When comparing vectors, you often care about **direction** (what
kind of thing it represents) rather than **magnitude** (how much of it there
is). Normalization strips away the magnitude, leaving only direction.

For example, two documents about the same topic might have very different
lengths, but after normalization their vectors point in the same direction.

## Matrices: Lists of Lists

A **matrix** is a rectangular grid of numbers — in Python, a list of lists:

```python
matrix = [
    [1, 2],
    [3, 4],
    [5, 6]
]
```

This is a 3x2 matrix (3 rows, 2 columns). Each inner list is a row.

Multiplying a matrix by a vector applies a **transformation**. Each row of the
matrix computes a dot product with the vector:

```python
matrix = [[1, 2], [3, 4]]
vector = [5, 6]

result = []
for row in matrix:
    result.append(sum(row[i] * vector[i] for i in range(len(vector))))
# result = [17, 39]
```

Row 0: `1*5 + 2*6 = 17`
Row 1: `3*5 + 4*6 = 39`

Matrix-vector multiplication is how neural networks transform data as it flows
through layers. Each layer has a weight matrix. Multiplying the input vector by
this matrix produces the next layer's input.

## Rounding for Clean Output

Throughout this course you will use `round()` to produce clean, deterministic
output:

```python
print(round(3.14159, 2))   # 3.14
print(round(0.6, 1))       # 0.6
```

The second argument is the number of decimal places.

## Checking Your Python Version

All AI code in this course runs on Python. You can check your version:

```bash
python3 --version
```

This prints something like `Python 3.12.3`. Knowing your version matters because
different versions support different features.

## Running a Script

To run a Python script:

```bash
python3 vector_add.py
```

Python reads your file top to bottom and executes it. No compilation step, no
`main()` function. You write the code, you run it, you see the result.
