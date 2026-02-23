# Basic Queries

SQL (Structured Query Language) is the standard language for working with
relational databases. It has been around since the 1970s and is used everywhere:
web applications, mobile apps, data science, embedded systems, and more. If data
is stored in a structured way, SQL is almost certainly involved.

A **database** is an organized collection of data. A **relational database**
stores data in **tables** — think spreadsheets with rows and columns. Each table
has a fixed set of columns (the schema) and a variable number of rows (the data).

In this course you are using **SQLite**, a lightweight embedded database. Unlike
PostgreSQL or MySQL, SQLite does not run as a separate server — it stores
everything in a single file. This makes it perfect for learning because there is
nothing to install or configure.

## Your First Query: SELECT

The most fundamental SQL statement is `SELECT`. It retrieves data from a table.

```sql
SELECT * FROM users;
```

Breaking this down:

- `SELECT` tells the database you want to read data
- `*` means "all columns"
- `FROM users` specifies which table to read from
- The semicolon `;` terminates the statement

If the `users` table has columns `id`, `name`, and `email`, this query returns
every row with all three columns.

SQL keywords like `SELECT` and `FROM` are case-insensitive. You could write
`select * from users` and it would work the same way. The convention is to write
keywords in uppercase for readability, but it is not required.

## Selecting Specific Columns

You rarely need every column. To select specific ones, list them after `SELECT`:

```sql
SELECT name, email FROM users;
```

This returns only the `name` and `email` columns, ignoring `id`. The columns
appear in the order you list them, not the order they were defined in the table.

You can select a single column:

```sql
SELECT name FROM users;
```

Or reorder them:

```sql
SELECT email, name FROM users;
```

The table itself is unchanged — `SELECT` only controls what you see in the
result, never what is stored.

## Removing Duplicates with DISTINCT

Sometimes a column has repeated values. If you want only unique values, use
`DISTINCT`:

```sql
SELECT DISTINCT category FROM products;
```

If the `products` table has ten rows but only three unique categories, this
returns just those three. Without `DISTINCT`, you would see all ten rows,
duplicates included.

`DISTINCT` applies to the entire row. If you select multiple columns, a row is
considered a duplicate only when all selected columns match:

```sql
SELECT DISTINCT category, price FROM products;
```

Two rows with the same category but different prices are not duplicates here.

## Limiting Results with LIMIT and OFFSET

When a table has thousands of rows, you often want only a few:

```sql
SELECT * FROM users LIMIT 3;
```

This returns at most three rows. The database picks the first three it finds
(the order is not guaranteed unless you also use `ORDER BY`).

To skip rows before taking your limit, use `OFFSET`:

```sql
SELECT * FROM users LIMIT 3 OFFSET 2;
```

This skips the first two rows and then returns the next three. `OFFSET` is
useful for pagination — showing results page by page.

`OFFSET` without `LIMIT` has no effect in SQLite. You always pair them together.

## Sorting with ORDER BY

To control the order of results, use `ORDER BY`:

```sql
SELECT * FROM users ORDER BY name;
```

This sorts the results by the `name` column in ascending order (A to Z for text,
smallest to largest for numbers). Ascending is the default, but you can be
explicit:

```sql
SELECT * FROM users ORDER BY name ASC;
```

To reverse the sort, use `DESC` (descending):

```sql
SELECT * FROM users ORDER BY name DESC;
```

You can sort by multiple columns. The second column breaks ties in the first:

```sql
SELECT * FROM products ORDER BY category ASC, price DESC;
```

This sorts products by category alphabetically, and within each category, by
price from highest to lowest.

You can also sort by column position (1-based):

```sql
SELECT name, email FROM users ORDER BY 1;
```

This sorts by the first column in the result (`name`). Using column names is
clearer, but column numbers work too.

## Column Aliases with AS

Sometimes column names are not descriptive enough, or you want friendlier names
in your output. Use `AS` to create an alias:

```sql
SELECT name AS username FROM users;
```

The result column is labeled `username` instead of `name`. The table itself is
unchanged — `AS` only affects the output.

Aliases are especially useful with expressions (which you will learn later):

```sql
SELECT name, price * 0.9 AS sale_price FROM products;
```

If an alias contains spaces, wrap it in double quotes:

```sql
SELECT name AS "Product Name" FROM products;
```

You can also alias without the `AS` keyword — just put the alias after the
column name:

```sql
SELECT name username FROM users;
```

But using `AS` is more readable and strongly recommended.

## SQL Comments

Comments are notes for humans. The database ignores them.

**Single-line comments** use two dashes:

```sql
-- This is a comment
SELECT * FROM users;  -- This is also a comment
```

Everything after `--` on that line is ignored.

**Block comments** use `/* ... */` and can span multiple lines:

```sql
/*
  This query retrieves all users.
  Used for the admin dashboard.
*/
SELECT * FROM users;
```

Block comments can also appear in the middle of a statement:

```sql
SELECT name, /* email, */ id FROM users;
```

This selects `name` and `id` but not `email` — the `email,` part is commented
out. This is handy when you are experimenting and want to temporarily exclude
a column.

Use comments to explain **why** a query is written a certain way, not **what**
it does. Good SQL is mostly self-explanatory.

## Summary

| Concept   | Syntax                          | Purpose                    |
|-----------|---------------------------------|----------------------------|
| Select all| `SELECT * FROM table`           | Get all columns            |
| Columns   | `SELECT a, b FROM table`        | Get specific columns       |
| Distinct  | `SELECT DISTINCT a FROM table`  | Remove duplicates          |
| Limit     | `... LIMIT n`                   | Cap the number of rows     |
| Offset    | `... LIMIT n OFFSET m`          | Skip rows before limiting  |
| Order     | `... ORDER BY a ASC`            | Sort ascending             |
| Order     | `... ORDER BY a DESC`           | Sort descending            |
| Alias     | `SELECT a AS b FROM table`      | Rename a column in output  |
| Comment   | `-- text` or `/* text */`       | Notes for humans           |

These are the building blocks of every SQL query. The exercises that follow will
have you practice each one individually so the syntax becomes second nature.
