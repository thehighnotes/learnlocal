# Filtering Data

So far you have retrieved entire tables with `SELECT`. In practice, tables contain
thousands or millions of rows, and you almost never want all of them. The `WHERE`
clause lets you specify exactly which rows to return.

## The WHERE Clause

`WHERE` goes after `FROM` and before `ORDER BY`. It evaluates a condition for each
row and only includes rows where the condition is true:

```sql
SELECT name, department
FROM employees
WHERE department = 'Engineering';
```

This returns only employees in the Engineering department. Every other row is
filtered out before the result is returned.

The general form is:

```sql
SELECT columns
FROM table
WHERE condition;
```

`WHERE` does not change the data in the table. It only controls which rows appear
in the query result.

## Comparison Operators

SQL provides six comparison operators. They work on numbers, text, and dates:

| Operator | Meaning                  |
|----------|--------------------------|
| `=`      | Equal to                 |
| `!=`     | Not equal to             |
| `<`      | Less than                |
| `>`      | Greater than             |
| `<=`     | Less than or equal to    |
| `>=`     | Greater than or equal to |

Note that SQL uses a single `=` for equality comparison, not `==` like most
programming languages.

```sql
-- Employees earning more than 70000
SELECT name, salary
FROM employees
WHERE salary > 70000;

-- Employees NOT in Sales
SELECT name, department
FROM employees
WHERE department != 'Sales';
```

Text comparisons are case-sensitive in most databases. In SQLite specifically,
`=` and `!=` on text are case-sensitive by default, while `LIKE` is
case-insensitive (more on that later).

## Logical Operators: AND, OR, NOT

You can combine multiple conditions using logical operators.

**AND** requires both conditions to be true:

```sql
-- Engineering employees earning over 80000
SELECT name, salary
FROM employees
WHERE department = 'Engineering' AND salary > 80000;
```

**OR** requires at least one condition to be true:

```sql
-- Employees in Engineering OR Sales
SELECT name, department
FROM employees
WHERE department = 'Engineering' OR department = 'Sales';
```

**NOT** negates a condition:

```sql
-- Everyone except Engineering
SELECT name, department
FROM employees
WHERE NOT department = 'Engineering';
```

When mixing AND and OR, **AND binds tighter** (higher precedence). Use
parentheses to make your intent explicit:

```sql
-- This might not do what you expect:
WHERE department = 'Engineering' OR department = 'Sales' AND salary > 70000
-- Evaluated as: Engineering OR (Sales AND salary > 70000)

-- Use parentheses to be clear:
WHERE (department = 'Engineering' OR department = 'Sales') AND salary > 70000
```

Always use parentheses when combining AND and OR. It prevents bugs and makes
the query readable for the next person (including future you).

## BETWEEN: Range Filtering

`BETWEEN` checks if a value falls within an inclusive range:

```sql
SELECT name, salary
FROM employees
WHERE salary BETWEEN 50000 AND 70000;
```

This is equivalent to:

```sql
WHERE salary >= 50000 AND salary <= 70000
```

Both endpoints are **included**. `BETWEEN 50000 AND 70000` includes rows where
salary is exactly 50000 or exactly 70000.

`BETWEEN` works with text and dates too:

```sql
-- Employees hired in 2022
SELECT name, hire_date
FROM employees
WHERE hire_date BETWEEN '2022-01-01' AND '2022-12-31';
```

You can negate it with `NOT BETWEEN`:

```sql
WHERE salary NOT BETWEEN 50000 AND 70000
```

## IN: Set Membership

`IN` checks if a value matches any item in a list:

```sql
SELECT name, department
FROM employees
WHERE department IN ('Engineering', 'Sales', 'Marketing');
```

This is equivalent to:

```sql
WHERE department = 'Engineering'
   OR department = 'Sales'
   OR department = 'Marketing'
```

`IN` is much cleaner when you have more than two values. It works with numbers
too:

```sql
WHERE id IN (1, 3, 5, 7)
```

Negate with `NOT IN`:

```sql
WHERE department NOT IN ('Engineering', 'Sales')
```

Later in this course you will learn to use subqueries inside `IN`, which makes
it extremely powerful.

## LIKE: Pattern Matching

`LIKE` matches text against a pattern using two wildcard characters:

| Wildcard | Meaning                          |
|----------|----------------------------------|
| `%`      | Matches zero or more characters  |
| `_`      | Matches exactly one character    |

```sql
-- Names starting with 'J'
SELECT name FROM employees WHERE name LIKE 'J%';

-- Names ending with 'son'
SELECT name FROM employees WHERE name LIKE '%son';

-- Names containing 'ar'
SELECT name FROM employees WHERE name LIKE '%ar%';

-- Names with exactly 5 characters
SELECT name FROM employees WHERE name LIKE '_____';
```

In SQLite, `LIKE` is **case-insensitive** for ASCII characters by default.
`WHERE name LIKE 'john%'` would match "John", "JOHN", and "john". This differs
from `=`, which is case-sensitive.

Negate with `NOT LIKE`:

```sql
WHERE name NOT LIKE 'J%'
```

If you need to match a literal `%` or `_` in the text, use the `ESCAPE` clause:

```sql
WHERE discount LIKE '10\%' ESCAPE '\'
```

## NULL Handling

`NULL` in SQL means "unknown" or "missing". It is not zero, not an empty string,
not false -- it is the absence of a value.

The critical rule: **NULL is not equal to anything, not even itself.**

```sql
-- This finds NOTHING, even if manager_id is NULL:
WHERE manager_id = NULL     -- WRONG

-- This is the correct way:
WHERE manager_id IS NULL    -- RIGHT
```

Use `IS NULL` and `IS NOT NULL` to test for missing values:

```sql
-- Find employees with no manager
SELECT name FROM employees WHERE manager_id IS NULL;

-- Find employees who have a manager
SELECT name FROM employees WHERE manager_id IS NOT NULL;
```

NULL also affects other comparisons. Any arithmetic or comparison involving NULL
produces NULL (not true, not false):

```sql
-- These all evaluate to NULL (unknown), not true or false:
NULL = NULL      -- NULL
NULL != NULL     -- NULL
NULL > 5         -- NULL
NULL + 10        -- NULL
```

This is why `WHERE manager_id = NULL` returns no rows -- the comparison evaluates
to NULL, which is not true, so the row is excluded.

When writing queries, always consider: "Could this column contain NULL? If so,
do I need to handle that case?"

## Combining Everything

Real queries often combine several filtering techniques:

```sql
SELECT name, department, salary
FROM employees
WHERE department IN ('Engineering', 'Sales')
  AND salary BETWEEN 60000 AND 90000
  AND manager_id IS NOT NULL
ORDER BY salary DESC;
```

The order of conditions in WHERE does not affect the result (though it can
affect readability). The database engine decides the most efficient evaluation
order internally.

In the exercises that follow, you will practice each filtering technique on
an employees table.
