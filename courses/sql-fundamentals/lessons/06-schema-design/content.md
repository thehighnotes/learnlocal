# Schema Design

Until now you have been querying and modifying tables that already existed. In this
lesson you learn how to create tables from scratch, define their columns and
constraints, modify them after creation, and remove them entirely.

The collection of tables, columns, types, and constraints that define the structure
of a database is called its **schema**. Good schema design is critical because
changing a schema later — after data is stored — is much harder than getting it
right up front.

## CREATE TABLE

The `CREATE TABLE` statement defines a new table:

```sql
CREATE TABLE students (
    id INTEGER,
    name TEXT,
    grade INTEGER
);
```

Breaking this down:

- `CREATE TABLE students` names the new table `students`.
- Inside the parentheses, each line declares a column with a name and a type.
- Columns are separated by commas. The last column has no trailing comma.
- The semicolon terminates the statement.

If the table already exists, `CREATE TABLE` raises an error. To avoid this, use
`IF NOT EXISTS`:

```sql
CREATE TABLE IF NOT EXISTS students (
    id INTEGER,
    name TEXT,
    grade INTEGER
);
```

This silently does nothing if the table already exists.

## SQLite Data Types

SQLite uses a flexible type system called **type affinity**. Unlike PostgreSQL or
MySQL, SQLite does not rigidly enforce column types. Instead, it uses five storage
classes:

| Storage Class | Description                                  |
|---------------|----------------------------------------------|
| NULL          | A missing or unknown value                   |
| INTEGER       | A whole number (positive, negative, or zero)  |
| REAL          | A floating-point number                      |
| TEXT          | A string of characters                       |
| BLOB          | Raw binary data (images, files, etc.)        |

When you declare a column type, SQLite maps it to one of these affinities. For
example, `VARCHAR(100)` maps to TEXT affinity, and `FLOAT` maps to REAL affinity.
SQLite will try to convert values to the column's affinity, but it will not reject
a value that does not match. You could store the text `"hello"` in an INTEGER
column — SQLite allows it, though it is almost always a mistake.

For this course, stick to the five storage classes directly. They are the clearest
way to express your intent in SQLite.

## PRIMARY KEY

A **primary key** uniquely identifies each row in a table. No two rows can have the
same primary key value, and it cannot be NULL.

```sql
CREATE TABLE students (
    id INTEGER PRIMARY KEY,
    name TEXT,
    grade INTEGER
);
```

When a column is declared `INTEGER PRIMARY KEY`, SQLite treats it specially: it
becomes an alias for the internal `rowid`. If you insert a row without specifying
the `id`, SQLite automatically assigns the next available integer.

```sql
INSERT INTO students (name, grade) VALUES ('Alice', 90);
-- id is automatically assigned as 1
```

### AUTOINCREMENT

By default, SQLite reuses deleted rowid values. If you insert rows 1, 2, 3, delete
row 3, and insert again, the new row might get id 3. To guarantee that ids always
increase and are never reused, add `AUTOINCREMENT`:

```sql
CREATE TABLE students (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    grade INTEGER
);
```

`AUTOINCREMENT` has a small performance cost because SQLite must maintain an extra
internal table. Use it only when you need guaranteed monotonically increasing ids,
such as for audit logs or external references.

## FOREIGN KEY

A **foreign key** creates a link between two tables. It says "this column's value
must exist in another table's column."

```sql
CREATE TABLE classes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE enrollments (
    id INTEGER PRIMARY KEY,
    student_id INTEGER,
    class_id INTEGER,
    FOREIGN KEY (student_id) REFERENCES students(id),
    FOREIGN KEY (class_id) REFERENCES classes(id)
);
```

The `FOREIGN KEY` line declares that `student_id` must contain a value that exists
in `students.id`. If you try to insert an enrollment with a `student_id` that does
not exist in the students table, the database should reject it.

**Important SQLite caveat:** Foreign key enforcement is disabled by default. You
must enable it for each connection:

```sql
PRAGMA foreign_keys = ON;
```

Without this pragma, SQLite silently accepts invalid foreign key values. Always
include this at the top of your SQL files when working with foreign keys.

## NOT NULL and DEFAULT

By default, any column in SQLite can contain NULL. To require a value, add the
`NOT NULL` constraint:

```sql
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL NOT NULL
);
```

If you try to insert a row without a `name` or `price`, SQLite raises an error.

The `DEFAULT` constraint provides a fallback value when none is given:

```sql
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    price REAL NOT NULL,
    in_stock INTEGER NOT NULL DEFAULT 1
);
```

Now if you omit `in_stock` during an insert, it defaults to 1 (meaning "yes, in
stock"). You can combine `NOT NULL` and `DEFAULT` — the default satisfies the
not-null requirement when no value is provided.

Common default values:

- `DEFAULT 0` for counters or flags
- `DEFAULT ''` for empty strings
- `DEFAULT CURRENT_TIMESTAMP` for creation timestamps

## UNIQUE Constraint

The `UNIQUE` constraint ensures that no two rows have the same value in a column:

```sql
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL
);
```

Unlike a primary key, a table can have multiple UNIQUE columns. Also, UNIQUE
columns can contain NULL values (unless also marked NOT NULL), and multiple NULLs
are considered distinct — they do not violate the uniqueness constraint.

You can also create a UNIQUE constraint across multiple columns:

```sql
CREATE TABLE enrollments (
    student_id INTEGER,
    class_id INTEGER,
    UNIQUE(student_id, class_id)
);
```

This allows a student to enroll in many classes and a class to have many students,
but prevents the same student from enrolling in the same class twice.

## ALTER TABLE

After creating a table, you can modify it with `ALTER TABLE`. SQLite supports a
limited set of alterations compared to other databases.

### Add a Column

```sql
ALTER TABLE students ADD COLUMN email TEXT;
```

The new column is added at the end. Existing rows get NULL for the new column
(unless you specify a DEFAULT).

```sql
ALTER TABLE students ADD COLUMN active INTEGER NOT NULL DEFAULT 1;
```

### Rename a Table

```sql
ALTER TABLE students RENAME TO learners;
```

All references to `students` must be updated manually — SQLite does not
automatically update foreign keys or queries.

### Rename a Column

```sql
ALTER TABLE students RENAME COLUMN grade TO score;
```

This renames the column in the table definition. Existing data is preserved.

### Limitations

SQLite does not support:

- Dropping a column (added in SQLite 3.35.0, but not universally available)
- Changing a column's type
- Adding or removing constraints on existing columns

When you need these changes, the workaround is to create a new table with the
desired schema, copy the data, drop the old table, and rename the new one.

## DROP TABLE

To remove a table entirely:

```sql
DROP TABLE students;
```

This deletes the table and all its data. There is no undo. Use with caution.

To avoid errors when the table might not exist:

```sql
DROP TABLE IF EXISTS students;
```

This silently does nothing if the table does not exist.

## Summary

| Concept         | Syntax                                       | Purpose                          |
|-----------------|----------------------------------------------|----------------------------------|
| Create table    | `CREATE TABLE t (col TYPE, ...)`             | Define a new table               |
| If not exists   | `CREATE TABLE IF NOT EXISTS t (...)`         | Avoid error if table exists      |
| Primary key     | `col INTEGER PRIMARY KEY`                    | Unique row identifier            |
| Autoincrement   | `col INTEGER PRIMARY KEY AUTOINCREMENT`      | Never-reused auto ids            |
| Foreign key     | `FOREIGN KEY (col) REFERENCES other(col)`    | Link to another table            |
| Enable FK       | `PRAGMA foreign_keys = ON;`                  | Turn on FK enforcement           |
| Not null        | `col TYPE NOT NULL`                          | Require a value                  |
| Default         | `col TYPE DEFAULT value`                     | Fallback when value omitted      |
| Unique          | `col TYPE UNIQUE`                            | No duplicate values              |
| Add column      | `ALTER TABLE t ADD COLUMN col TYPE`          | Add column to existing table     |
| Rename table    | `ALTER TABLE t RENAME TO new_name`           | Change table name                |
| Rename column   | `ALTER TABLE t RENAME COLUMN old TO new`     | Change column name               |
| Drop table      | `DROP TABLE t`                               | Delete table and all data        |

These are the tools for designing and evolving your database schema. The exercises
that follow will have you practice each concept so you are comfortable creating and
modifying tables on your own.
