# Subqueries

So far every query you have written has been a single SELECT statement. But SQL
lets you nest one query inside another. The inner query is called a **subquery**
(or sometimes a "nested query" or "inner query"). The outer query that contains
it is the **outer query**.

Subqueries are wrapped in parentheses and can appear in several places: the
WHERE clause, the SELECT list, and even the FROM clause. They are one of the
most powerful features in SQL because they let you break complex problems into
smaller, composable steps.

## Scalar Subqueries

A **scalar subquery** returns exactly one value — one row, one column. You can
use it anywhere a single value is expected: in a WHERE comparison, in a SELECT
expression, or as a function argument.

```sql
SELECT name FROM students
WHERE grade_level = (SELECT MAX(grade_level) FROM students);
```

The inner query `(SELECT MAX(grade_level) FROM students)` runs first and
produces a single number. The outer query then uses that number in its WHERE
clause, as if you had written `WHERE grade_level = 12`.

You can also use scalar subqueries in the SELECT list:

```sql
SELECT name,
       (SELECT COUNT(*) FROM enrollments WHERE student_id = students.id) AS num_courses
FROM students;
```

This adds a computed column to each row. The subquery runs once per student and
counts how many courses that student is enrolled in.

If a scalar subquery returns more than one row, the database raises an error.
If it returns zero rows, the result is NULL.

## IN with Subquery

The IN operator checks whether a value matches any value in a list. Instead of
writing the list by hand, you can generate it with a subquery:

```sql
SELECT name FROM students
WHERE id IN (
  SELECT student_id FROM enrollments
  JOIN courses ON enrollments.course_id = courses.id
  WHERE courses.department = 'Science'
);
```

The inner query returns a set of student IDs — everyone enrolled in a Science
course. The outer query then finds the names that match those IDs.

IN subqueries return a **set of values** (one column, zero or more rows). The
outer query checks each of its rows against that set.

You can combine IN with DISTINCT in the subquery to avoid duplicates, though
the database handles this efficiently either way:

```sql
SELECT name FROM students
WHERE id IN (
  SELECT DISTINCT student_id FROM enrollments WHERE score > 90
);
```

## EXISTS

EXISTS does not care about what the subquery returns — only whether it returns
**any rows at all**. It evaluates to true if the subquery produces at least one
row, false otherwise.

```sql
SELECT name FROM students s
WHERE EXISTS (
  SELECT 1 FROM enrollments WHERE student_id = s.id
);
```

This finds all students who have at least one enrollment. The `SELECT 1` is a
convention — it does not matter what the subquery selects because EXISTS only
checks for the existence of rows, not their content. You could write
`SELECT *`, `SELECT 42`, or `SELECT 'hello'` and the result would be the same.

EXISTS is especially useful when you need to check for the presence of related
data without actually retrieving it.

## Correlated Subqueries

In the examples above, some subqueries reference a column from the outer query.
These are called **correlated subqueries**. They are "correlated" because the
inner query depends on the current row of the outer query.

```sql
SELECT s.name,
       (SELECT MAX(score) FROM enrollments WHERE student_id = s.id) AS best_score
FROM students s;
```

For each student in the outer query, the subquery finds that specific student's
highest score. The reference to `s.id` is what makes it correlated — without
it, the subquery would return the global maximum across all students.

Correlated subqueries conceptually run once per row of the outer query. This
can be slower than a JOIN for large tables, but the database optimizer often
rewrites them behind the scenes. Write whichever is clearer for your use case.

A common correlated pattern is filtering rows based on aggregates of related
data:

```sql
SELECT s.name FROM students s
WHERE (SELECT AVG(score) FROM enrollments WHERE student_id = s.id) > 85;
```

This finds students whose average score across all their courses exceeds 85.

## Subqueries in FROM (Derived Tables)

A subquery in the FROM clause creates a temporary result set — sometimes called
a **derived table** or **inline view**. You can query it just like a regular
table:

```sql
SELECT sub.name, sub.num_courses
FROM (
  SELECT s.name, COUNT(*) AS num_courses
  FROM students s
  JOIN enrollments e ON s.id = e.student_id
  GROUP BY s.id
) AS sub
WHERE sub.num_courses >= 3
ORDER BY sub.name;
```

The inner query groups students by ID and counts their enrollments. The outer
query then filters that result, keeping only students with three or more
courses. The alias `sub` is required — every derived table needs a name.

Derived tables are useful when you need to filter on an aggregate (like
`COUNT` or `AVG`) without using HAVING, or when you want to layer one
transformation on top of another.

## Nested Subqueries

There is no limit to how deep you can nest subqueries. A subquery can contain
another subquery, which can contain yet another:

```sql
SELECT DISTINCT s.name
FROM students s
JOIN enrollments e ON s.id = e.student_id
WHERE e.course_id IN (
  SELECT course_id FROM enrollments
  WHERE student_id = (SELECT id FROM students WHERE name = 'Alice')
)
AND s.name != 'Alice'
ORDER BY s.name;
```

Reading from the inside out:

1. The innermost query finds Alice's student ID.
2. The middle query finds all course IDs that Alice is enrolled in.
3. The outer query finds all other students enrolled in any of those courses.

Each level solves one piece of the problem. While deeply nested subqueries can
become hard to read, two or three levels are common and perfectly fine.

## NOT IN and NOT EXISTS

Just as IN checks for membership, NOT IN checks for **non-membership**:

```sql
SELECT name FROM students
WHERE id NOT IN (SELECT student_id FROM enrollments);
```

This finds students who have zero enrollments. It is the logical inverse of the
IN subquery.

NOT EXISTS works similarly:

```sql
SELECT name FROM students s
WHERE NOT EXISTS (
  SELECT 1 FROM enrollments WHERE student_id = s.id
);
```

Both queries produce the same result, but there is an important difference when
NULLs are involved. If the NOT IN subquery returns any NULL values, the entire
NOT IN expression evaluates to unknown (effectively false for every row),
meaning **no rows are returned**. NOT EXISTS does not have this problem because
it only checks for row existence.

Rule of thumb: if the subquery column might contain NULLs, prefer NOT EXISTS
over NOT IN.

## When to Use Subqueries vs JOINs

Many subqueries can be rewritten as JOINs, and vice versa. Neither is always
better — it depends on readability and the specific problem.

Use a **subquery** when:

- You need a single computed value (scalar subquery)
- The logic is naturally a membership check (IN / NOT IN)
- You want to check existence without retrieving data (EXISTS)
- Breaking the problem into steps makes it clearer

Use a **JOIN** when:

- You need columns from multiple tables in the result
- Performance matters and the optimizer handles joins better
- The relationship between tables is straightforward

In practice, you will use both. The exercises that follow will give you hands-on
practice with each type of subquery so you can recognize when each one fits.

## Summary

| Subquery Type      | Placement    | Returns             | Use Case                          |
|--------------------|-------------|---------------------|-----------------------------------|
| Scalar             | WHERE/SELECT | One value           | Compare against computed value    |
| IN                 | WHERE        | Set of values       | Membership check                  |
| EXISTS             | WHERE        | True/false          | Check for related rows            |
| Correlated         | WHERE/SELECT | Depends on outer    | Per-row computation               |
| Derived table      | FROM         | Result set          | Layer transformations             |
| NOT IN             | WHERE        | Set exclusion       | Non-membership (watch for NULLs)  |
| NOT EXISTS         | WHERE        | True/false          | Safer exclusion with NULLs        |
