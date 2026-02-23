# The Attention Mechanism

Every modern language model — GPT, BERT, Claude, Llama — is built on one core
mechanism: **attention**. This lesson implements it from scratch.

The idea is simple: when processing a sequence (like a sentence), each element
needs to "look at" other elements to understand context. The word "bank" means
something different in "river bank" vs "bank account." Attention is how the
model decides which other words matter for understanding each word.

## Softmax: Making Probabilities

Before getting to attention, you need **softmax**. It converts any list of
numbers into probabilities that sum to 1:

```python
import math

def softmax(xs):
    exps = [math.exp(x) for x in xs]
    total = sum(exps)
    return [e / total for e in exps]
```

Larger inputs get larger probabilities. Softmax amplifies differences: if one
input is much larger than the others, it gets nearly all the probability.

```python
softmax([1, 2, 3])    # [0.09, 0.24, 0.67]
softmax([1, 1, 1])    # [0.33, 0.33, 0.33]  — equal inputs, equal probs
softmax([1, 5, 1])    # [0.02, 0.96, 0.02]  — one dominant
```

## Scaled Dot-Product Attention

Attention has three inputs:
- **Query (Q)**: what am I looking for?
- **Key (K)**: what do I contain?
- **Value (V)**: what information do I carry?

The process:

1. **Score**: compute dot product between the query and each key, scaled by
   `sqrt(d)` where d is the vector dimension
2. **Weight**: apply softmax to the scores to get attention weights
3. **Output**: compute a weighted sum of the value vectors

The formula:

```
Attention(Q, K, V) = softmax(Q · K^T / sqrt(d)) · V
```

### Why Scale by sqrt(d)?

Without scaling, dot products of high-dimensional vectors produce large numbers
that push softmax into extreme values (nearly 0 or 1). Dividing by `sqrt(d)`
keeps the values in a range where softmax produces meaningful gradients.

## Self-Attention

In **self-attention**, the queries, keys, and values all come from the same
sequence. Each token generates all three. This lets every token "attend to"
every other token:

```
"The cat sat" → each word looks at all three words
```

The attention weights reveal relationships:
- "cat" might attend strongly to "sat" (subject-verb connection)
- "the" might attend to "cat" (determiner-noun connection)

## Multi-Head Attention

A single attention computation captures one kind of relationship. **Multi-head
attention** runs multiple attention computations in parallel, each on a
different portion of the vector:

1. Split each vector into N heads (e.g., 2 heads from a 4-dimensional vector:
   first 2 dimensions and last 2 dimensions)
2. Run independent attention on each head
3. Concatenate the results

Different heads learn different relationships: one might capture syntax, another
semantics, another positional relationships.

*Watch attention patterns form in AIquest's Attention Visualizer.*

In the exercises that follow, you will implement each piece: softmax, attention
scores, attention weights, the weighted output, complete scaled dot-product
attention, self-attention patterns, and multi-head attention.
