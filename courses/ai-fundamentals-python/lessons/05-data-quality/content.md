# Data Quality

Data scientists often say they spend 80% of their time cleaning data and 20%
building models. This is not an exaggeration. The best neural network in the
world will produce garbage if you feed it garbage data. This lesson teaches you
to detect and fix the most common data problems.

## Why Data Quality Matters

A model learns patterns from its training data. If the data has problems, the
model learns the wrong patterns:

- **Missing values** can crash a model or bias it toward incomplete records
- **Mislabeled data** teaches the model wrong answers
- **Class imbalance** makes the model ignore rare but important cases
- **Different scales** let large-valued features dominate small-valued ones

Fixing these problems before training is almost always more effective than
building a more complex model.

## Loading Data

Real-world data often comes as CSV (comma-separated values). You can parse it
into a list of dictionaries:

```python
csv_data = """name,age,income
Alice,30,50000
Bob,25,60000"""

lines = csv_data.strip().split("\n")
headers = lines[0].split(",")
rows = []
for line in lines[1:]:
    values = line.split(",")
    row = {}
    for i, h in enumerate(headers):
        row[h] = values[i]
    rows.append(row)
```

Each row becomes a dictionary like `{"name": "Alice", "age": "30", "income":
"50000"}`. Notice that all values are strings — you convert to numbers when
needed.

## Missing Values

Missing values show up as empty strings, "NA", "null", or similar markers.
Detection is the first step:

```python
for col in headers:
    count = sum(1 for row in rows if row[col] == "")
    if count > 0:
        print(f"{col}: {count} missing")
```

### Strategies for Missing Values

| Strategy | When to Use |
|----------|-------------|
| Drop the row | Few missing values, large dataset |
| Fill with mean | Numerical column, roughly symmetric distribution |
| Fill with median | Numerical column, skewed distribution |
| Fill with mode | Categorical column |
| Fill with 0 | When missing genuinely means "none" |

**Mean imputation** is the most common for numerical data:

```python
known = [float(row[col]) for row in rows if row[col] != ""]
mean = sum(known) / len(known)
for row in rows:
    if row[col] == "":
        row[col] = str(mean)
```

## Label Noise

**Label noise** is when training examples have the wrong label. It happens
more often than you might think — human annotators make mistakes, automated
labeling has errors, and data collection processes introduce confusion.

Even small amounts of noise degrade accuracy:

```
0% noise  → 90% accuracy
10% noise → ~82% accuracy
25% noise → ~69% accuracy
```

The effect is amplified because each mislabeled example pushes the model in
the wrong direction.

## Class Imbalance

When one class vastly outnumbers another, a simple "predict the majority"
classifier achieves high accuracy while being completely useless:

```
Dataset: 90% positive, 10% negative
"Always predict positive" → 90% accuracy!
But: 0% recall on the negative class
```

This is called the **accuracy paradox**. Accuracy alone is misleading for
imbalanced data. Better metrics include recall per class, precision, and F1
score.

## Normalization

Features at different scales can cause problems. If "age" ranges 0-100 and
"income" ranges 0-100,000, income dominates in distance calculations.

**Min-max normalization** scales each feature to the range 0-1:

```python
normalized = (value - min) / (max - min)
```

After normalization, all features contribute equally regardless of their
original scale.

*See the impact of bad data interactively in AIquest's Data Quality module.*

In the exercises that follow, you will load CSV data, detect and fix missing
values, observe the effect of label noise, expose the accuracy paradox, and
build a complete data cleaning pipeline.
