# Joining Tables

So far you have been working with a single table at a time. In practice,
relational databases split data across many tables to avoid duplication. A
`customers` table holds customer details. An `orders` table holds purchase
records and references back to the customer by ID. This design is called
**normalization** — store each fact once and link tables together with keys.

The problem: your data is now in separate tables. If you want a report that
shows "which customer ordered which product," you need to pull from both tables
at once. That is exactly what **joins** do.

## INNER JOIN

An `INNER JOIN` combines rows from two tables where a condition is met. Rows
that do not match are excluded from the result.

```sql
SELECT customers.name, orders.product
FROM customers
INNER JOIN orders ON customers.id = orders.customer_id;
```

Breaking this down:

- `FROM customers` is the left table.
- `INNER JOIN orders` brings in the right table.
- `ON customers.id = orders.customer_id` is the **join condition** — it tells
  the database which rows belong together. Only rows where a customer's `id`
  matches an order's `customer_id` appear in the result.

If a customer has three orders, they appear three times (once per order). If a
customer has zero orders, they do not appear at all — that is the "inner" part.

The word `INNER` is optional. `JOIN` by itself means `INNER JOIN`:

```sql
SELECT customers.name, orders.product
FROM customers
JOIN orders ON customers.id = orders.customer_id;
```

Both forms are identical. Using `INNER JOIN` is more explicit and recommended
when you are mixing join types in one query.

## LEFT JOIN

A `LEFT JOIN` (also called `LEFT OUTER JOIN`) returns all rows from the left
table, even if there is no matching row in the right table. When there is no
match, the right table's columns are filled with `NULL`.

```sql
SELECT customers.name, orders.product
FROM customers
LEFT JOIN orders ON customers.id = orders.customer_id;
```

If a customer has no orders, they still appear in the result with `NULL` for the
product column. This is useful when you want to find customers who have not
ordered anything, or when you want a complete list regardless of matches.

To find customers with no orders, add a `WHERE` clause checking for `NULL`:

```sql
SELECT customers.name
FROM customers
LEFT JOIN orders ON customers.id = orders.customer_id
WHERE orders.id IS NULL;
```

`IS NULL` is the correct way to check for NULL in SQL. Using `= NULL` does not
work because NULL is not equal to anything, not even itself.

There is also a `RIGHT JOIN` which does the opposite — keeps all rows from the
right table. SQLite does not support `RIGHT JOIN`, but you can always achieve the
same result by swapping the table order in a `LEFT JOIN`.

## Multiple Joins

You can join more than two tables in a single query. Each `JOIN` clause brings
in one additional table:

```sql
SELECT customers.name, orders.product, orders.amount
FROM customers
JOIN orders ON customers.id = orders.customer_id
JOIN products ON orders.product_id = products.id;
```

The second `JOIN` connects `orders` to `products`. You can chain as many joins
as you need. The database processes them left to right — first it joins
`customers` with `orders`, then joins that result with `products`.

You can mix join types in a single query. For example, `LEFT JOIN` one table and
`INNER JOIN` another:

```sql
SELECT customers.name, orders.product, reviews.rating
FROM customers
LEFT JOIN orders ON customers.id = orders.customer_id
INNER JOIN reviews ON orders.id = reviews.order_id;
```

The order and type of each join matters. Think carefully about which rows you
want to preserve.

## Self-Joins

A **self-join** joins a table to itself. This is useful for hierarchical data
like org charts, where an employee's manager is also stored in the same table.

```sql
SELECT e.name AS employee, m.name AS manager
FROM employees e
JOIN employees m ON e.manager_id = m.id;
```

Here `employees` appears twice — once as `e` (the employee) and once as `m`
(the manager). The join condition links each employee's `manager_id` to the
manager's `id`.

Table aliases are mandatory in self-joins. Without them, the database cannot
tell which reference to `employees` you mean.

Note: the CEO or top-level manager typically has a `NULL` manager_id. An
`INNER JOIN` self-join will exclude them from the result. Use `LEFT JOIN` if
you want to include employees with no manager.

## Table Aliases

When queries involve multiple tables, typing full table names gets tedious.
**Table aliases** assign short names:

```sql
SELECT c.name, o.product, o.amount
FROM customers AS c
JOIN orders AS o ON c.id = o.customer_id;
```

`c` is now shorthand for `customers`, and `o` for `orders`. The `AS` keyword is
optional — you can write `FROM customers c` — but using `AS` is clearer.

Aliases are especially valuable when:

- Table names are long (`customer_order_line_items` becomes `li`)
- You join the same table multiple times (self-joins)
- You want the query to fit on screen

Once you define an alias, use it everywhere in the query. Mixing the full name
and the alias in the same query is valid SQL but confusing to read.

## Complex Join Conditions

The `ON` clause is not limited to simple equality. You can use any condition:

```sql
SELECT c.name, o.product, o.amount
FROM customers c
JOIN orders o ON c.id = o.customer_id AND o.amount > 100;
```

This joins customers to orders but only includes orders over 100. Rows with
smaller amounts are excluded entirely from the join, which is different from
filtering with `WHERE` after the join — especially with `LEFT JOIN`.

With `LEFT JOIN`, conditions in `ON` and `WHERE` behave differently:

```sql
-- ON condition: customer appears even if no orders > 100
SELECT c.name, o.product
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id AND o.amount > 100;

-- WHERE condition: customer excluded if they have no orders > 100
SELECT c.name, o.product
FROM customers c
LEFT JOIN orders o ON c.id = o.customer_id
WHERE o.amount > 100;
```

The `ON` version preserves the LEFT JOIN behavior — every customer appears. The
`WHERE` version filters after the join, which can eliminate rows that the LEFT
JOIN was supposed to keep.

You can also join on inequality conditions or even ranges:

```sql
SELECT a.name, b.name
FROM employees a
JOIN employees b ON a.id < b.id;
```

This generates all unique pairs of employees. The `<` condition ensures each
pair appears only once (Alice-Bob but not Bob-Alice).

## CROSS JOIN

A `CROSS JOIN` produces the **Cartesian product** — every row from the left
table combined with every row from the right table. There is no `ON` condition.

```sql
SELECT colors.name, sizes.name
FROM colors
CROSS JOIN sizes;
```

If `colors` has 3 rows and `sizes` has 4 rows, the result has 12 rows (3 x 4).

Cross joins are rarely needed, but they are useful when you genuinely want every
combination — generating a matrix of all possible color-size pairs, for example.

Be careful with cross joins on large tables. Crossing two 1000-row tables
produces 1,000,000 rows.

An accidental cross join is what happens when you forget the `ON` condition
in a regular join. If you see unexpectedly large results, check your join
conditions first.

You can also write a cross join as a comma-separated FROM clause:

```sql
SELECT colors.name, sizes.name
FROM colors, sizes;
```

This is the older SQL syntax and is equivalent to `CROSS JOIN`. The explicit
`CROSS JOIN` keyword is preferred because it makes the intent clear.

## Summary

| Join Type   | Syntax                              | Behavior                               |
|-------------|-------------------------------------|----------------------------------------|
| INNER JOIN  | `A JOIN B ON condition`             | Only matching rows from both tables    |
| LEFT JOIN   | `A LEFT JOIN B ON condition`        | All rows from A, NULLs when B missing  |
| Self-join   | `A JOIN A ON condition`             | Table joined to itself (needs aliases) |
| CROSS JOIN  | `A CROSS JOIN B`                    | Every combination (no ON needed)       |

Key points:

- INNER JOIN is the default and most common — use it when you only want matches
- LEFT JOIN is essential for "show all, even without matches" queries
- Self-joins require aliases to distinguish the two references
- Put filtering in ON (not WHERE) to preserve LEFT JOIN behavior
- CROSS JOIN is powerful but produces large results — use deliberately
