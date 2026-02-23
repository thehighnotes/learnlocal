# Fairness and Bias

AI systems make decisions that affect people: who gets a loan, who gets hired,
who gets flagged by surveillance. When these systems are biased, they can
systematically discriminate against entire groups of people. This lesson teaches
you to detect and measure algorithmic bias.

## How Bias Enters AI

AI learns from data. If the data reflects historical discrimination, the model
learns to discriminate:

- A hiring model trained on past decisions learns that the company historically
  rejected female applicants, and continues the pattern.
- A credit model trained on biased lending data charges higher rates to certain
  zip codes, perpetuating redlining.
- A facial recognition system trained mostly on light-skinned faces fails more
  often on dark-skinned faces.

The model is not "choosing" to discriminate — it is faithfully reproducing
patterns in the data it was given. That makes the problem harder to detect
and fix.

## Per-Group Accuracy

The first step in detecting bias is checking if the model performs equally well
for all groups:

```python
correct_a = sum(1 for p, t in zip(pred_a, true_a) if p == t)
accuracy_a = correct_a / len(pred_a)
```

If accuracy differs significantly between groups, something is wrong.

## Demographic Parity

**Demographic parity** requires that the positive prediction rate is the same
for all groups. If a model approves 70% of group A but only 30% of group B,
it lacks demographic parity.

```python
rate = sum(predictions) / len(predictions)
```

Equal rates do not guarantee fairness (both groups could be poorly served
equally), but unequal rates are a strong signal of bias.

## Disparate Impact Ratio

The **disparate impact ratio** formalizes demographic parity as a number:

```
ratio = min_rate / max_rate
```

The **4/5ths rule** (from US employment law): if the ratio is below 0.80,
there is evidence of potential discrimination. A ratio of 1.0 means perfect
parity.

## Confusion Matrix

A **confusion matrix** breaks predictions into four categories:

| | Predicted Positive | Predicted Negative |
|---|---|---|
| Actually Positive | True Positive (TP) | False Negative (FN) |
| Actually Negative | False Positive (FP) | True Negative (TN) |

Building a confusion matrix per group reveals *where* errors concentrate.
Overall accuracy can hide that one group has far more false negatives (missed
cases) or false positives (false accusations).

## Equalized Odds

**Equalized odds** requires two metrics to be equal across groups:

- **True Positive Rate (TPR)**: TP / (TP + FN) — of all positive cases, how
  many were correctly predicted?
- **False Positive Rate (FPR)**: FP / (FP + TN) — of all negative cases, how
  many were incorrectly flagged?

If group A has TPR 0.90 and group B has TPR 0.60, the model misses 40% of
positive cases in group B but only 10% in group A. That is unequal treatment.

## What to Do About Bias

1. **Measure it** — use the metrics from this lesson
2. **Find the source** — usually in the training data
3. **Fix the data** — rebalance, resample, remove biased features
4. **Re-evaluate** — check if metrics improved after changes
5. **Monitor** — bias can reappear as data distribution shifts

*Explore fairness metrics interactively in AIquest's Bias Detection module.*

In the exercises that follow, you will compute per-group accuracy, check
demographic parity, apply the 4/5ths rule, fix a biased dataset, build
confusion matrices, test equalized odds, and generate a complete fairness
report.
