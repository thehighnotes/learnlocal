# Aggregation & Grouping

So far you have been selecting individual rows from tables. But often you need
summaries: how many orders came in? What is the average price? Which category
sold the most? SQL answers these questions with **aggregate functions** and
**GROUP BY**.

## Aggregate Functions

An aggregate function takes a set of values (usually an entire column) and
returns a single result. SQLite provides five built-in aggregates:

| Function   | Purpose                              |
|------------|--------------------------------------|
| `COUNT()`  | Count rows or non-NULL values        |
| `SUM()`    | Add up numeric values                |
| `AVG()`    | Compute the arithmetic mean          |
| `MIN()`    | Find the smallest value              |
| `MAX()`    | Find the largest value               |

### COUNT — Counting Rows

`COUNT(*)` counts every row in the result, regardless of NULL values:

```sql
SELECT COUNT(*) FROM sales;
```

If the `sales` table has 12 rows, this returns `12`.

`COUNT(column)` counts only the rows where that column is not NULL:

```sql
SELECT COUNT(email) FROM users;
```

If three users have no email (NULL), and five have emails, this returns `5`
while `COUNT(*)` would return `8`.

The difference matters. Use `COUNT(*)` when you want total rows. Use
`COUNT(column)` when you want to know how many rows have a value in that
specific column.

### SUM — Adding Up Values

`SUM` adds up all non-NULL values in a column:

```sql
SELECT SUM(quantity) FROM sales;
```

If the quantities across all rows add up to 60, this returns `60`.

An important type detail: SUM of an INTEGER column returns an integer. SUM of a
REAL column returns a real number. This affects how the output looks — `60`
versus `60.0`.

`SUM` ignores NULLs. If a column has values 10, NULL, 20, the sum is 30, not
NULL.

### AVG — Computing the Mean

`AVG` divides the sum by the count of non-NULL values:

```sql
SELECT AVG(quantity) FROM sales;
```

`AVG` always returns a REAL (floating-point) value, even when applied to an
integer column. So if the sum is 60 and there are 12 rows, you get `5.0`, not
`5`.

Like `SUM`, `AVG` ignores NULLs in its calculation. This means the count it
divides by is the number of non-NULL values, not the total number of rows. If
a column has values 10, NULL, 20, the average is 15.0 (sum 30 divided by count
2), not 10.0 (sum 30 divided by 3 rows).

### MIN and MAX — Finding Extremes

`MIN` returns the smallest value, `MAX` returns the largest:

```sql
SELECT MIN(price), MAX(price) FROM sales;
```

For numbers, smallest and largest are obvious. For text, `MIN` gives the first
value in alphabetical order and `MAX` gives the last. For dates stored as text
in ISO format (`YYYY-MM-DD`), `MIN` and `MAX` work correctly because
alphabetical order matches chronological order.

`MIN` and `MAX` preserve the original type. If the column is INTEGER, the result
is INTEGER. If it is REAL, the result is REAL.

## Combining Aggregates

You can use multiple aggregate functions in one query:

```sql
SELECT COUNT(*), SUM(quantity), AVG(price) FROM sales;
```

This returns a single row with three values: the total number of sales, the
sum of all quantities, and the average price.

You can also mix aggregates with literal values and expressions:

```sql
SELECT
  COUNT(*) AS total_sales,
  SUM(quantity) AS units_sold,
  MIN(price) AS cheapest,
  MAX(price) AS most_expensive
FROM sales;
```

Aliases make the output columns self-documenting.

## GROUP BY — Aggregating Per Group

Without GROUP BY, aggregate functions process all rows together and return one
result. GROUP BY splits the rows into groups based on one or more columns, then
applies the aggregate to each group separately.

```sql
SELECT category, COUNT(*)
FROM sales
GROUP BY category;
```

This counts how many sales belong to each category. If there are three
categories (Books, Clothing, Electronics), the result has three rows — one
per group.

Every non-aggregated column in your SELECT must appear in the GROUP BY clause.
This query is wrong:

```sql
-- WRONG: product is not in GROUP BY and not aggregated
SELECT category, product, COUNT(*)
FROM sales
GROUP BY category;
```

SQLite will not error on this (unlike stricter databases), but the value of
`product` in each row is arbitrary — it just picks one from the group. Always
include every non-aggregated column in GROUP BY.

### GROUP BY with ORDER BY

GROUP BY does not guarantee any particular order. Add ORDER BY to sort the
grouped results:

```sql
SELECT category, COUNT(*) AS sale_count
FROM sales
GROUP BY category
ORDER BY sale_count DESC;
```

This shows the category with the most sales first. You can ORDER BY the alias,
the aggregate expression, or the column name.

### GROUP BY with Multiple Columns

You can group by more than one column. Each unique combination of values becomes
its own group:

```sql
SELECT category, region, SUM(quantity)
FROM sales
GROUP BY category, region
ORDER BY category, region;
```

If you have 3 categories and 3 regions, you could have up to 9 groups (one for
each combination). Groups with no matching rows simply do not appear.

This is how you create cross-tabulations: sales by category within each region,
errors by type within each module, and so on.

## HAVING — Filtering Groups

WHERE filters individual rows before aggregation. But what if you want to filter
groups after aggregation? That is what HAVING does.

```sql
SELECT category, SUM(quantity) AS total_qty
FROM sales
GROUP BY category
HAVING SUM(quantity) > 15;
```

This first groups by category and computes the sum, then only keeps groups where
the total quantity exceeds 15. Groups that do not meet the condition are excluded
from the result.

### WHERE vs HAVING

This is one of the most common sources of confusion in SQL:

- **WHERE** filters rows before they enter the aggregation
- **HAVING** filters groups after the aggregation is complete

```sql
-- Only count sales in the North region, then keep categories with > 5 units
SELECT category, SUM(quantity) AS total_qty
FROM sales
WHERE region = 'North'
GROUP BY category
HAVING SUM(quantity) > 5;
```

The processing order is:

1. **FROM** — start with the sales table
2. **WHERE** — discard rows where region is not 'North'
3. **GROUP BY** — group remaining rows by category
4. **Aggregates** — compute SUM(quantity) for each group
5. **HAVING** — discard groups where the sum is 15 or less
6. **SELECT** — return the surviving columns
7. **ORDER BY** — sort the final result (if present)

You cannot use aggregate functions in WHERE:

```sql
-- WRONG: WHERE cannot use SUM
SELECT category, SUM(quantity)
FROM sales
WHERE SUM(quantity) > 15
GROUP BY category;
```

This is a syntax error. Aggregates only exist after grouping, and WHERE runs
before grouping. Use HAVING instead.

## Aggregation with DISTINCT

You can combine DISTINCT with aggregate functions:

```sql
SELECT COUNT(DISTINCT category) FROM sales;
```

This counts how many unique categories exist, rather than counting all rows.
Without DISTINCT, `COUNT(category)` counts every non-NULL category value
including duplicates.

## NULL Handling in Aggregates

All aggregate functions except `COUNT(*)` ignore NULL values:

| Expression       | Rows: 10, NULL, 20 | Result |
|------------------|---------------------|--------|
| `COUNT(*)`       | Counts all rows     | 3      |
| `COUNT(column)`  | Counts non-NULL     | 2      |
| `SUM(column)`    | 10 + 20             | 30     |
| `AVG(column)`    | 30 / 2              | 15.0   |
| `MIN(column)`    | min(10, 20)         | 10     |
| `MAX(column)`    | max(10, 20)         | 20     |

If all values in a group are NULL, `SUM`, `AVG`, `MIN`, and `MAX` return NULL.
`COUNT(column)` returns 0. `COUNT(*)` still counts the rows.

## Summary

| Concept           | Syntax                                     | Purpose                           |
|-------------------|--------------------------------------------|-----------------------------------|
| Count rows        | `COUNT(*)`                                 | Total number of rows              |
| Count non-NULL    | `COUNT(column)`                            | Rows with a value in that column  |
| Sum               | `SUM(column)`                              | Add up all values                 |
| Average           | `AVG(column)`                              | Arithmetic mean (always REAL)     |
| Minimum           | `MIN(column)`                              | Smallest value                    |
| Maximum           | `MAX(column)`                              | Largest value                     |
| Group             | `GROUP BY col1, col2`                      | Aggregate per unique combination  |
| Filter groups     | `HAVING condition`                         | Keep only groups meeting criteria |
| Count unique      | `COUNT(DISTINCT column)`                   | Number of distinct values         |

The processing order to remember: FROM, WHERE, GROUP BY, aggregates, HAVING,
SELECT, ORDER BY. WHERE filters rows. HAVING filters groups. Getting these
two mixed up is the single most common aggregation mistake.
