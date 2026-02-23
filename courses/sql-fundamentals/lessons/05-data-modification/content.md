# Inserting, Updating & Deleting

So far you have only read data from tables. Now it is time to modify them. SQL
provides three core statements for changing data: INSERT adds new rows, UPDATE
changes existing rows, and DELETE removes rows. These are collectively called
**DML** — Data Manipulation Language.

One critical thing to understand: INSERT, UPDATE, and DELETE produce **no output**.
They silently modify the table. To see the results, you follow them with a SELECT
query. This is a pattern you will use in every exercise.

## INSERT INTO — Adding Rows

The simplest form inserts a single row with explicit values:

```sql
INSERT INTO contacts (id, name, email, phone, city)
VALUES (1, 'Alice', 'alice@example.com', '555-0101', 'Portland');
```

You list the columns you want to fill, then the values in the same order. The
column list is technically optional — you can write:

```sql
INSERT INTO contacts VALUES (1, 'Alice', 'alice@example.com', '555-0101', 'Portland');
```

But always include the column list. It protects you when the table schema changes
and makes the statement self-documenting.

### Inserting Multiple Rows

You can insert several rows in a single statement by separating value tuples
with commas:

```sql
INSERT INTO contacts (id, name, email, phone, city) VALUES
  (1, 'Alice', 'alice@example.com', '555-0101', 'Portland'),
  (2, 'Bob', 'bob@example.com', '555-0102', 'Seattle'),
  (3, 'Charlie', 'charlie@example.com', '555-0103', 'Portland');
```

This is more efficient than three separate INSERT statements. The database
processes them in a single operation, which matters when you are inserting
thousands of rows.

### INSERT INTO ... SELECT

Instead of literal values, you can insert rows from a query:

```sql
INSERT INTO archive (id, name, email, phone, city)
SELECT id, name, email, phone, city
FROM contacts
WHERE city = 'Portland';
```

This copies every Portland contact into the archive table. The SELECT can be as
complex as you like — joins, aggregations, subqueries — as long as the output
columns match the target table.

### NULL and Default Values

If you omit a column from the column list, SQLite fills it with the column's
default value. If there is no default, it uses NULL:

```sql
INSERT INTO contacts (id, name, email) VALUES (4, 'Diana', 'diana@example.com');
-- phone and city will be NULL
```

For INTEGER PRIMARY KEY columns, you can pass NULL to let SQLite auto-generate
the next value:

```sql
INSERT INTO contacts (id, name, email, phone, city)
VALUES (NULL, 'Eve', 'eve@example.com', '555-0105', 'Denver');
-- id will be auto-assigned
```

## UPDATE — Changing Existing Rows

UPDATE modifies columns in rows that match a condition:

```sql
UPDATE contacts
SET email = 'newalice@example.com'
WHERE name = 'Alice';
```

You can update multiple columns at once:

```sql
UPDATE contacts
SET email = 'newalice@example.com', city = 'Seattle'
WHERE name = 'Alice';
```

### The WHERE Clause is Critical

Without WHERE, UPDATE changes **every row** in the table:

```sql
-- DANGER: this updates ALL contacts
UPDATE contacts SET city = 'Unknown';
```

This is almost never what you want. Always write the WHERE clause first, or
write a SELECT with the same WHERE to preview which rows will be affected:

```sql
-- Preview first
SELECT * FROM contacts WHERE name = 'Alice';

-- Then update
UPDATE contacts SET city = 'Seattle' WHERE name = 'Alice';
```

### Updating with Expressions

You can use expressions and other columns in the SET clause:

```sql
UPDATE products SET price = price * 1.10 WHERE category = 'Electronics';
```

This raises prices by 10% for all electronics. The right side of `=` can
reference any column in the table.

## DELETE — Removing Rows

DELETE removes rows that match a condition:

```sql
DELETE FROM contacts WHERE name = 'Bob';
```

Like UPDATE, **always use WHERE**:

```sql
-- DANGER: this deletes ALL rows
DELETE FROM contacts;
```

Without WHERE, the table is emptied completely. The table itself still exists
(unlike DROP TABLE), but every row is gone.

### Deleting with Subqueries

You can combine DELETE with subqueries:

```sql
DELETE FROM contacts
WHERE city IN (SELECT city FROM closed_offices);
```

This removes contacts whose city appears in a list of closed offices. The
subquery runs first, producing the list of cities, then DELETE uses that list.

## REPLACE INTO — SQLite's Upsert

REPLACE INTO is a SQLite-specific extension. It works like INSERT, but if a row
with the same primary key already exists, it deletes the old row and inserts the
new one:

```sql
REPLACE INTO contacts (id, name, email, phone, city)
VALUES (1, 'Alice Updated', 'alice.new@example.com', '555-9999', 'Denver');
```

If id 1 exists, the old row is deleted and the new row takes its place. If id 1
does not exist, it is simply inserted.

This is useful for "insert or update" scenarios. However, be aware: REPLACE
deletes and re-inserts rather than modifying in place. This means:

- Any columns you omit get their default values (not the old row's values)
- Foreign key ON DELETE triggers will fire
- The rowid may change

For true upsert behavior that preserves unmentioned columns, use INSERT with
ON CONFLICT (covered in a later lesson). REPLACE is the simpler, blunter tool.

## Verifying Your Changes

Since DML statements produce no output, you need a follow-up SELECT to see what
happened:

```sql
INSERT INTO contacts (id, name, email, phone, city)
VALUES (6, 'Frank', 'frank@example.com', '555-0106', 'Denver');

SELECT * FROM contacts WHERE id = 6;
```

The INSERT runs silently, then the SELECT shows the new row. In the exercises
that follow, every solution ends with a SELECT that verifies the modification.

## Safety Practices

A few habits that prevent data loss:

1. **Always write WHERE first.** Before typing UPDATE or DELETE, write the
   WHERE clause. Then go back and add the SET or DELETE FROM part.

2. **Preview with SELECT.** Run a SELECT with the same WHERE to see which rows
   will be affected before you commit to the change.

3. **Use transactions for multi-step changes.** Wrap related statements in
   BEGIN and COMMIT so they either all succeed or all roll back. (Transactions
   are covered in a later lesson.)

4. **Back up before bulk operations.** If you are about to UPDATE or DELETE
   many rows, copy the table or the database file first.

These practices apply everywhere SQL is used — from SQLite on your laptop to
PostgreSQL clusters in production.

## Quick Reference

| Statement                                    | Purpose                       |
|----------------------------------------------|-------------------------------|
| `INSERT INTO t (cols) VALUES (vals)`         | Add a single row              |
| `INSERT INTO t (cols) VALUES (...), (...)`   | Add multiple rows             |
| `INSERT INTO t (cols) SELECT ...`            | Add rows from a query         |
| `UPDATE t SET col = val WHERE ...`           | Change existing rows          |
| `DELETE FROM t WHERE ...`                    | Remove rows                   |
| `REPLACE INTO t (cols) VALUES (vals)`        | Insert or replace on PK match |

In the exercises that follow, you will practice each of these on a `contacts`
table. Every exercise modifies the data and then uses SELECT to verify the
result. Pay attention to the pipe-separated output format — that is how SQLite
displays query results.
