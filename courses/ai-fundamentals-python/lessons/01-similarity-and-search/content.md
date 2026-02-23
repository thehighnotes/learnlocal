# Similarity and Search

How does a computer know that "cat" and "kitten" are related? How does a search
engine find relevant results? How does a recommendation system suggest movies
you might like?

The answer is the same for all three: represent things as vectors, then measure
how similar those vectors are. This lesson teaches you the two most common
similarity measures in AI and puts them to work building a search engine from
scratch.

## Representing Things as Vectors

In the previous lesson, you worked with vectors as abstract lists of numbers.
In practice, AI systems convert real-world data into vectors:

- **Words** become vectors of 300 numbers that capture meaning. "Cat" and
  "kitten" get similar vectors because they appear in similar contexts.
- **Documents** become vectors based on the words they contain.
- **Images** become vectors of pixel values or learned features.
- **Users** become vectors based on their preferences and behavior.

The key insight: once everything is a vector, comparing "how similar are these
two things?" becomes "how similar are these two lists of numbers?"

## Cosine Similarity

**Cosine similarity** measures the angle between two vectors. It ranges from -1
to 1:

| Value | Meaning |
|-------|---------|
| 1.0   | Identical direction (most similar) |
| 0.0   | Perpendicular (unrelated) |
| -1.0  | Opposite direction (most dissimilar) |

The formula uses the dot product and magnitudes from lesson 00:

```python
cos_sim = dot(a, b) / (magnitude(a) * magnitude(b))
```

Why cosine similarity instead of just the dot product? Because cosine
similarity ignores magnitude. A short document and a long document about the
same topic will have different-length vectors, but cosine similarity sees them
as equally similar because it only cares about direction.

```python
import math

def dot(a, b):
    return sum(a[i] * b[i] for i in range(len(a)))

def magnitude(v):
    return math.sqrt(sum(x**2 for x in v))

def cosine_sim(a, b):
    return dot(a, b) / (magnitude(a) * magnitude(b))

a = [1, 2, 3]
b = [2, 4, 6]     # same direction, different length
print(cosine_sim(a, b))   # 1.0
```

Even though `b` is twice as long as `a`, cosine similarity is 1.0 because they
point in the same direction.

## Euclidean Distance

**Euclidean distance** measures the straight-line distance between two points:

```python
def euclidean_dist(a, b):
    return math.sqrt(sum((a[i] - b[i])**2 for i in range(len(a))))
```

This is the familiar distance formula from geometry, extended to any number of
dimensions. Smaller distance means more similar.

When should you use cosine similarity vs Euclidean distance?

- **Cosine similarity** when you care about direction (text, word meanings)
- **Euclidean distance** when you care about absolute position (clustering,
  nearest neighbors in physical space)

In practice, cosine similarity is more common in AI because most representations
care about what type of thing something is, not how much of it there is.

## Word Vectors

Modern AI systems learn vector representations for words called **embeddings**.
Words with similar meanings get similar vectors:

```python
words = {
    "cat":    [0.9, 0.1, 0.0],
    "kitten": [0.85, 0.15, 0.0],
    "dog":    [0.8, 0.2, 0.1],
    "car":    [0.0, 0.1, 0.9],
}
```

In this simplified example, "cat" and "kitten" have very similar vectors (high
cosine similarity), while "cat" and "car" are very different. Real word vectors
have 300+ dimensions and are learned from billions of words of text.

## Word Analogies

The most remarkable property of word vectors is that vector arithmetic captures
meaning:

```
king - man + woman ≈ queen
```

This works because the vector difference "king - man" captures the concept of
"royalty minus maleness." Adding "woman" gives "royalty plus femaleness," which
lands near "queen."

```python
target = [king[i] - man[i] + woman[i] for i in range(len(king))]
# Then find the word whose vector is most similar to target
```

## Building a Search Engine

A search engine is just similarity measurement at scale:

1. Represent each document as a vector
2. Represent the search query as a vector
3. Compute cosine similarity between the query and every document
4. Return the documents with highest similarity

```python
scored = []
for title, doc_vec in documents.items():
    sim = cosine_sim(query, doc_vec)
    scored.append((sim, title))
scored.sort(reverse=True)
```

This is the foundation of how real search engines work. Google's earliest
versions used a similar approach (TF-IDF vectors + cosine similarity). Modern
search engines use learned vector representations, but the core idea is the
same.

*See these concepts visualized interactively in AIquest's Semantic
Understanding module.*

In the exercises that follow, you will implement cosine similarity, build a word
analogy solver, and create a working search engine — all in pure Python.
