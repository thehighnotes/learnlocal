# Neural Networks

In the previous lesson you built a single neuron and discovered that it cannot
solve XOR. The solution: stack neurons into layers. When multiple neurons work
together, each drawing its own decision boundary, the combined network can
learn far more complex patterns.

This lesson takes you from a single neuron to a trainable multi-layer network.
By the end, you will train a network to solve XOR — the problem that stumped a
single perceptron.

## Layers

A **layer** is a group of neurons that all take the same inputs. Each neuron
has its own weights and bias:

```python
def layer_forward(inputs, weights, biases):
    output = []
    for i in range(len(weights)):
        z = sum(weights[i][j] * inputs[j] for j in range(len(inputs))) + biases[i]
        output.append(sigmoid(z))
    return output
```

The `weights` parameter is a matrix (list of lists) where each row is one
neuron's weights. The `biases` list has one bias per neuron.

For example, a layer with 2 neurons taking 2 inputs:
- `weights = [[0.5, 0.1], [0.3, 0.7]]` — 2 neurons, each with 2 weights
- `biases = [0.3, 0.3]`

Each neuron independently computes its weighted sum + bias + sigmoid.

## Forward Propagation

**Forward propagation** passes data through multiple layers in sequence:

```
Input → Layer 1 (hidden) → Layer 2 (output) → Prediction
```

The output of one layer becomes the input to the next:

```python
hidden = layer_forward(inputs, w1, b1)
output = layer_forward(hidden, w2, b2)
```

The hidden layer transforms the input into a new representation. The output
layer transforms that representation into the final prediction. This is the
key insight: the hidden layer learns to represent the data in a way that makes
the output layer's job easy.

## Loss Functions

A **loss function** measures how wrong the network's predictions are. Lower
loss means better predictions. The most common is **Mean Squared Error (MSE)**:

```python
mse = sum((predicted[i] - actual[i])**2 for i in range(n)) / n
```

MSE squares each error (so both positive and negative errors count equally)
and averages them. A perfect prediction gives MSE = 0.

## Backpropagation

Training a network means adjusting weights to reduce the loss. This requires
knowing how much each weight contributes to the loss — its **gradient**.

**Backpropagation** computes gradients using the chain rule of calculus. The
key idea: start at the output and work backward, layer by layer, computing
how each weight affects the loss.

For the output layer:
```
gradient = (output error) × (sigmoid derivative) × (input to this neuron)
```

For hidden layers, the error is propagated back through the next layer's
weights.

### The Sigmoid Derivative

The derivative of sigmoid is remarkably simple:

```python
sigmoid_derivative(x) = sigmoid(x) * (1 - sigmoid(x))
```

At x=0: sigmoid = 0.5, derivative = 0.5 * 0.5 = 0.25 (maximum).
At x=±∞: sigmoid approaches 0 or 1, derivative approaches 0.

This means the gradient is largest when the neuron's output is near 0.5
(uncertain) and smallest when it is near 0 or 1 (confident). This is called
the **vanishing gradient** problem — deep networks with sigmoid activations
can have very small gradients in early layers.

## The Training Loop

Training repeats three steps:

1. **Forward pass**: compute the output
2. **Compute loss**: measure how wrong the output is
3. **Backward pass + update**: compute gradients, adjust weights

```python
for epoch in range(num_epochs):
    output = forward(inputs, weights)
    loss = compute_loss(output, target)
    gradients = backward(output, target, weights)
    weights = update(weights, gradients, learning_rate)
```

The **learning rate** controls how big each weight update is. Too large and the
network overshoots; too small and it barely moves.

*Watch data flow through layers in AIquest's Neural Networks module.*

In the exercises that follow, you will implement each piece: a single layer,
two chained layers, MSE loss, the sigmoid derivative, gradient updates, a
complete training step, and finally a trained XOR solver.
