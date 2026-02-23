# Advanced Queries

You have already learned the fundamentals of SQL: selecting, filtering, joining,
aggregating, subqueries, data modification, and schema design. This lesson covers
the tools that turn good queries into great ones — conditional logic, NULL
handling, text manipulation, date arithmetic, window functions, and CTEs.

## CASE Expressions

A `CASE` expression lets you add if/then logic directly inside a query. It
evaluates conditions in order and returns the first matching result:

```sql
SELECT name, salary,
       CASE
           WHEN salary >= 90000 THEN 'Senior'
           WHEN salary >= 70000 THEN 'Mid'
           ELSE 'Junior'
       END AS level
FROM employees;
```

Each `WHEN` is tested top to bottom. The first true condition wins. If none
match, the `ELSE` value is used. If you omit `ELSE` and nothing matches, the
result is `NULL`.

You can use `CASE` anywhere an expression is valid — in `SELECT`, `WHERE`,
`ORDER BY`, even inside aggregate functions:

```sql
SELECT department,
       COUNT(CASE WHEN salary >= 80000 THEN 1 END) AS high_earners
FROM employees
GROUP BY department;
```

Here `CASE` returns `1` for high earners and `NULL` for everyone else. `COUNT`
ignores NULLs, so it effectively counts only the high earners per department.

There is also a simple form for equality checks:

```sql
SELECT name,
       CASE department
           WHEN 'Engineering' THEN 'Eng'
           WHEN 'Marketing' THEN 'Mkt'
           ELSE 'Other'
       END AS dept_code
FROM employees;
```

The searched form (`CASE WHEN condition`) is more flexible because it supports
any comparison, not just equality.

## Handling NULLs with COALESCE

`NULL` represents "unknown" or "missing" in SQL. It is not zero, not an empty
string, not false — it is the absence of a value. Any arithmetic or comparison
with `NULL` produces `NULL`:

```sql
SELECT 5 + NULL;    -- NULL
SELECT NULL = NULL;  -- NULL (not true!)
```

`COALESCE` takes a list of values and returns the first non-NULL one:

```sql
SELECT name, COALESCE(email, 'no email on file') AS contact
FROM employees;
```

If `email` is `NULL`, the result is `'no email on file'`. If `email` has a
value, that value is returned unchanged.

You can chain multiple fallbacks:

```sql
SELECT COALESCE(nickname, first_name, 'Unknown') AS display_name
FROM users;
```

This tries `nickname` first, then `first_name`, then falls back to `'Unknown'`.

`IFNULL(a, b)` is a SQLite-specific shorthand for `COALESCE(a, b)` with exactly
two arguments. `COALESCE` is standard SQL and works everywhere.

## String Functions

SQLite provides a solid set of string functions. Here are the most useful ones.

**LENGTH** returns the number of characters:

```sql
SELECT name, LENGTH(name) FROM employees;
-- 'Alice' -> 5
```

**UPPER and LOWER** change case:

```sql
SELECT UPPER('hello');  -- 'HELLO'
SELECT LOWER('HELLO');  -- 'hello'
```

**SUBSTR** extracts a substring. It is 1-indexed:

```sql
SELECT SUBSTR('SQLite', 1, 3);  -- 'SQL'
SELECT SUBSTR('SQLite', 4);     -- 'ite' (to end)
```

The signature is `SUBSTR(string, start, length)`. If `length` is omitted, it
goes to the end of the string.

**REPLACE** swaps all occurrences of a substring:

```sql
SELECT REPLACE('hello world', 'world', 'SQL');  -- 'hello SQL'
```

**TRIM** removes leading and trailing whitespace (or specified characters):

```sql
SELECT TRIM('  hello  ');          -- 'hello'
SELECT TRIM('xxhelloxx', 'x');    -- 'hello'
SELECT LTRIM('  hello');           -- 'hello' (left only)
SELECT RTRIM('hello  ');           -- 'hello' (right only)
```

**INSTR** finds the position of the first occurrence of a substring:

```sql
SELECT INSTR('hello world', 'world');  -- 7
SELECT INSTR('hello', 'xyz');          -- 0 (not found)
```

**|| (concatenation)** joins strings together:

```sql
SELECT 'Hello' || ' ' || 'World';  -- 'Hello World'
SELECT name || ' <' || email || '>' FROM employees;
```

This is the standard SQL concatenation operator. Unlike some databases, SQLite
does not have a `CONCAT` function — use `||` instead.

## Date Functions

SQLite stores dates as text in `'YYYY-MM-DD'` format. It provides several
functions to work with them.

**DATE** extracts or computes a date:

```sql
SELECT DATE('2024-03-15');              -- '2024-03-15'
SELECT DATE('2024-03-15', '+7 days');   -- '2024-03-22'
SELECT DATE('2024-03-15', '-1 month');  -- '2024-02-15'
SELECT DATE('now');                     -- today's date
```

**TIME** and **DATETIME** work similarly but include time components:

```sql
SELECT DATETIME('2024-03-15 10:30:00', '+2 hours');
-- '2024-03-15 12:30:00'
```

**STRFTIME** formats a date using format codes:

```sql
SELECT STRFTIME('%Y', '2024-03-15');    -- '2024' (year)
SELECT STRFTIME('%m', '2024-03-15');    -- '03'   (month)
SELECT STRFTIME('%d', '2024-03-15');    -- '15'   (day)
SELECT STRFTIME('%w', '2024-03-15');    -- '5'    (weekday, 0=Sunday)
```

Common format codes: `%Y` (4-digit year), `%m` (month 01-12), `%d` (day 01-31),
`%H` (hour 00-23), `%M` (minute 00-59), `%S` (second 00-59), `%w` (weekday
0-6), `%j` (day of year 001-366).

**julianday** converts a date to a Julian day number — the number of days since
November 24, 4714 BC. This is useful for computing the difference between dates:

```sql
SELECT julianday('2024-12-31') - julianday('2024-01-01');
-- 365.0 (days between the two dates)
```

A practical example — finding how many days each employee has been working:

```sql
SELECT name,
       CAST(julianday('2025-01-01') - julianday(hire_date) AS INTEGER) AS days_employed
FROM employees;
```

`CAST(... AS INTEGER)` truncates the decimal because `julianday` returns a
float.

## Window Functions

Window functions perform calculations across a set of rows related to the
current row, without collapsing them into a single result like `GROUP BY` does.

The key difference: `GROUP BY` reduces many rows to one per group. Window
functions keep every row and add computed columns.

**ROW_NUMBER** assigns a sequential number to each row:

```sql
SELECT name, department, salary,
       ROW_NUMBER() OVER (ORDER BY salary DESC) AS rank
FROM employees;
```

`OVER (ORDER BY salary DESC)` defines the window — all rows, ordered by salary
descending. The first row gets 1, the second gets 2, and so on.

**PARTITION BY** creates sub-windows within the data:

```sql
SELECT name, department, salary,
       ROW_NUMBER() OVER (PARTITION BY department ORDER BY salary DESC) AS dept_rank
FROM employees;
```

Now each department gets its own numbering. The highest-paid person in
Engineering gets 1, the highest-paid in Marketing gets 1, etc.

**RANK** is similar to `ROW_NUMBER` but handles ties:

```sql
SELECT name, salary,
       RANK() OVER (ORDER BY salary DESC) AS salary_rank
FROM employees;
```

If two employees have the same salary, they get the same rank, and the next rank
is skipped. With `ROW_NUMBER`, ties are broken arbitrarily.

**Aggregate window functions** apply `SUM`, `AVG`, `COUNT`, `MIN`, `MAX` over a
window instead of collapsing rows:

```sql
SELECT name, department, salary,
       SUM(salary) OVER (PARTITION BY department) AS dept_total,
       AVG(salary) OVER (PARTITION BY department) AS dept_avg
FROM employees;
```

Every row shows the employee's own salary plus their department's total and
average. No `GROUP BY` needed — you keep the individual rows.

You can also compute running totals:

```sql
SELECT name, salary,
       SUM(salary) OVER (ORDER BY hire_date) AS running_total
FROM employees;
```

The `ORDER BY` inside `OVER` makes it a running sum — each row includes itself
and all previous rows in that order.

Window functions require SQLite 3.25 or later (released September 2018).

## Common Table Expressions (CTEs)

A CTE is a named temporary result set that exists only for the duration of a
single query. Think of it as a `WITH` clause that creates a temporary view:

```sql
WITH high_earners AS (
    SELECT name, department, salary
    FROM employees
    WHERE salary >= 80000
)
SELECT department, COUNT(*) AS count
FROM high_earners
GROUP BY department;
```

The `WITH ... AS (...)` defines a CTE named `high_earners`. The main query then
uses it like a table. CTEs make complex queries more readable by breaking them
into named steps.

You can define multiple CTEs separated by commas:

```sql
WITH
    dept_stats AS (
        SELECT department, AVG(salary) AS avg_salary
        FROM employees
        GROUP BY department
    ),
    above_avg AS (
        SELECT e.name, e.department, e.salary, d.avg_salary
        FROM employees e
        JOIN dept_stats d ON e.department = d.department
        WHERE e.salary > d.avg_salary
    )
SELECT name, department, salary, avg_salary
FROM above_avg
ORDER BY salary DESC;
```

This first computes per-department averages, then finds employees above their
department's average. Without CTEs, you would need nested subqueries, which are
harder to read.

CTEs can reference earlier CTEs in the same `WITH` clause. The second CTE
(`above_avg`) references the first (`dept_stats`). This chaining is what makes
CTEs so powerful for multi-step data transformations.

CTEs require SQLite 3.8.3 or later (released February 2014).

## Summary

| Feature        | Syntax                                                      | Purpose                           |
|----------------|-------------------------------------------------------------|-----------------------------------|
| CASE           | `CASE WHEN cond THEN val ELSE val END`                      | Conditional logic in queries      |
| COALESCE       | `COALESCE(a, b, c)`                                         | First non-NULL value              |
| LENGTH         | `LENGTH(str)`                                               | Character count                   |
| UPPER / LOWER  | `UPPER(str)` / `LOWER(str)`                                 | Case conversion                   |
| SUBSTR         | `SUBSTR(str, start, len)`                                   | Extract substring (1-indexed)     |
| REPLACE        | `REPLACE(str, old, new)`                                    | Substitute text                   |
| TRIM           | `TRIM(str)` / `LTRIM` / `RTRIM`                             | Remove whitespace                 |
| INSTR          | `INSTR(str, substr)`                                        | Find substring position           |
| Concatenation  | `a \|\| b`                                                  | Join strings                      |
| DATE           | `DATE(str, modifier)`                                       | Date arithmetic                   |
| STRFTIME       | `STRFTIME(fmt, date)`                                       | Format dates                      |
| julianday      | `julianday(date)`                                           | Date to day number (for diffs)    |
| ROW_NUMBER     | `ROW_NUMBER() OVER (...)`                                   | Sequential row numbering          |
| RANK           | `RANK() OVER (...)`                                         | Ranking with ties                 |
| SUM/AVG OVER   | `SUM(col) OVER (PARTITION BY ...)`                          | Aggregate without GROUP BY        |
| CTE            | `WITH name AS (SELECT ...) SELECT ... FROM name`            | Named temporary result set        |

These advanced features are what separate basic SQL users from proficient ones.
The exercises that follow will give you hands-on practice with each one.
