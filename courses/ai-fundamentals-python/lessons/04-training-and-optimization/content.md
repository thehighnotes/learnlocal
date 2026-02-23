# Training and Optimization

In the previous lesson you built a neural network that could solve XOR by
training for 1000 epochs. But how do you design training for a real problem?
What learning rate should you use? How do you split your data? How do you know
when to stop?

This lesson teaches you to design training pipelines. You will implement the
components that every training system needs, and understand the decisions that
make the difference between a model that learns and one that fails.

**Important:** the goal is to understand how to set up and structure training,
not to run models to convergence. Real training runs take hours or days on GPUs.
Here you focus on the design — the concepts and code patterns that make
training work.

## Gradient Descent

Training a model means adjusting its parameters to reduce the loss. **Gradient
descent** does this by computing the gradient (slope) of the loss function and
stepping in the opposite direction:

```python
gradient = compute_gradient(parameters, data)
parameters = parameters - learning_rate * gradient
```

The gradient tells you which direction increases the loss. By going in the
opposite direction (subtracting), you decrease the loss.

For a simple function like `f(x) = (x - 3)²`:
- The derivative is `f'(x) = 2(x - 3)`
- At x=0: gradient = -6 (loss decreases by moving right)
- Update: x_new = 0 - lr * (-6) = 0 + 0.6 = 0.6

## Learning Rate

The **learning rate** (lr) controls how big each step is:

| Learning Rate | Effect |
|---------------|--------|
| Too small (0.001) | Very slow convergence, may get stuck |
| Just right (0.01-0.1) | Steady progress toward minimum |
| Too large (1.0+) | Overshoots, oscillates, may diverge |

Finding the right learning rate is one of the most important decisions in
training. A common approach: start with 0.01, increase if training is too slow,
decrease if loss jumps around.

## Batching

Real datasets have millions of examples. Computing the gradient on all of them
at once is too expensive. **Mini-batch gradient descent** splits the data into
small batches and updates parameters after each batch:

```python
for batch in split_into_batches(data, batch_size):
    gradient = compute_gradient(model, batch)
    model.update(gradient, lr)
```

Common batch sizes: 16, 32, 64, 128. Smaller batches train faster per step
(less computation) but noisier (gradient is an approximation). Larger batches
give more accurate gradients but take longer per step.

## Train/Validation Split

You need two separate datasets:

- **Training set** (typically 80%): used to update model parameters
- **Validation set** (typically 20%): used to check if the model generalizes

If training loss keeps going down but validation loss starts going up, the model
is **overfitting** — memorizing training data instead of learning patterns.

```python
split = int(len(data) * 0.8)
train = data[:split]
val = data[split:]
```

Always shuffle before splitting, so the split is random rather than taking the
last 20% which might have different characteristics.

## Shuffling

Shuffling the training data each epoch prevents the model from learning the
order of examples instead of their content. If you always show all cat images
first and dog images second, the model might learn "first half = cat" instead
of visual features.

```python
import random
for epoch in range(num_epochs):
    random.shuffle(train_data)
    for batch in make_batches(train_data, batch_size):
        train_on_batch(model, batch)
```

## Learning Rate Schedules

Starting with a higher learning rate and gradually reducing it often works
better than a fixed rate. Common schedules:

- **Step decay**: multiply lr by 0.9 every N epochs
- **Exponential decay**: lr = initial_lr * decay^epoch
- **Warmup**: start small, increase for a few epochs, then decay

```python
lr = initial_lr * (decay ** epoch)
```

## The Training Loop

Every training system follows the same structure:

```
for each epoch:
    shuffle training data
    for each batch:
        1. Forward pass   — compute prediction
        2. Compute loss   — how wrong is it?
        3. Backward pass  — compute gradients
        4. Update weights — adjust parameters
    evaluate on validation set
    adjust learning rate
```

The exercises that follow implement each component. You will not run training to
convergence — instead you will build the scaffolding that makes training
possible and verify each piece works correctly.
