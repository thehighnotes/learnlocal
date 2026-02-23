# The Perceptron

In the previous lessons you learned to compute dot products, measure similarity,
and build a search engine. Now you will build the fundamental unit of neural
networks: a **neuron**.

A neuron takes some inputs, multiplies each by a weight, adds a bias, and
applies an activation function. That is it. Every neural network — from a simple
image classifier to GPT — is made of these basic units connected together.

## The Neuron Model

A single neuron computes:

```
output = activate(inputs[0]*weights[0] + inputs[1]*weights[1] + ... + bias)
```

Or more compactly:

```
output = activate(dot(inputs, weights) + bias)
```

That dot product should look familiar from lesson 00. The neuron is just a
weighted sum (the dot product) plus a bias, passed through an activation
function.

The **weights** determine how much each input matters. The **bias** shifts the
decision boundary. The **activation function** introduces non-linearity — without
it, stacking neurons would be no more powerful than a single one.

## Activation Functions

### Step Function

The simplest activation: output 1 if the input is non-negative, 0 otherwise.

```python
def step(x):
    return 1 if x >= 0 else 0
```

This is a hard decision: on or off, yes or no. The original perceptron (1958)
used the step function.

### Sigmoid

Sigmoid squishes any real number into the range (0, 1):

```python
import math

def sigmoid(x):
    return 1 / (1 + math.exp(-x))
```

| Input | Output |
|-------|--------|
| -10   | ~0.00  |
| -1    | 0.27   |
| 0     | 0.50   |
| 1     | 0.73   |
| 10    | ~1.00  |

Sigmoid is smooth and differentiable, which makes it possible to train networks
using calculus (backpropagation). Large negative inputs give values near 0, large
positive inputs give values near 1, and 0 maps to exactly 0.5.

### ReLU (Rectified Linear Unit)

The most popular activation function in modern networks:

```python
def relu(x):
    return max(0, x)
```

If the input is positive, pass it through unchanged. If negative, output zero.
ReLU is simple, fast to compute, and works well in practice. Most hidden layers
in modern neural networks use ReLU.

## Logic Gates as Perceptrons

A perceptron with the step activation function can learn simple logic:

### AND Gate

AND outputs 1 only when both inputs are 1:

| A | B | AND |
|---|---|-----|
| 0 | 0 | 0   |
| 0 | 1 | 0   |
| 1 | 0 | 0   |
| 1 | 1 | 1   |

You can implement AND with weights `[1, 1]` and bias `-1.5`. The weighted sum
only reaches 0 or above when both inputs are 1.

### OR Gate

OR outputs 1 when at least one input is 1:

| A | B | OR |
|---|---|-----|
| 0 | 0 | 0   |
| 0 | 1 | 1   |
| 1 | 0 | 1   |
| 1 | 1 | 1   |

Weights `[1, 1]` and bias `-0.5` work: any single 1 pushes the sum above 0.

### XOR: The Impossible Problem

XOR outputs 1 when exactly one input is 1:

| A | B | XOR |
|---|---|-----|
| 0 | 0 | 0   |
| 0 | 1 | 1   |
| 1 | 0 | 1   |
| 1 | 1 | 0   |

No single perceptron can compute XOR. This is because the perceptron can only
draw a single straight line to separate the 0s from the 1s, and XOR's pattern
cannot be separated by one line. This is called **linear separability** — AND
and OR are linearly separable, XOR is not.

This limitation drove AI research toward multi-layer networks, which can draw
curved and complex decision boundaries. You will solve XOR with two layers in
the next lesson.

*See neurons in action in AIquest's Neural Networks 101 module.*

In the exercises that follow, you will implement each activation function, build
a neuron, and discover the XOR limitation firsthand.
