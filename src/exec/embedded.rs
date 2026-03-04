use crate::error::Result;

/// Output from an embedded runtime execution.
#[derive(Debug, Clone)]
pub struct EmbeddedOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Execute SQL using the embedded SQLite runtime.
///
/// - `setup_sql`: Optional SQL to seed the database (CREATE TABLE, INSERT, etc.)
/// - `student_sql`: The student's SQL to execute
///
/// SELECT results are formatted as pipe-separated values (matching SQLite CLI default).
/// DML statements report affected row counts.
pub fn execute_sql(setup_sql: Option<&str>, student_sql: &str) -> Result<EmbeddedOutput> {
    use rusqlite::Connection;

    let conn = match Connection::open_in_memory() {
        Ok(c) => c,
        Err(e) => {
            return Ok(EmbeddedOutput {
                stdout: String::new(),
                stderr: format!("Failed to open in-memory database: {}", e),
                exit_code: 1,
            });
        }
    };

    // Run setup SQL if provided
    if let Some(setup) = setup_sql {
        if let Err(e) = conn.execute_batch(setup) {
            return Ok(EmbeddedOutput {
                stdout: String::new(),
                stderr: format!("Setup error: {}", e),
                exit_code: 1,
            });
        }
    }

    // Execute the student's SQL — may contain multiple statements
    let trimmed = student_sql.trim();
    if trimmed.is_empty() {
        return Ok(EmbeddedOutput {
            stdout: String::new(),
            stderr: "No SQL to execute".to_string(),
            exit_code: 1,
        });
    }

    // Split into individual statements and execute each
    let mut output = String::new();
    let mut stderr_output = String::new();

    for statement in split_sql_statements(trimmed) {
        let stmt = statement.trim();
        if stmt.is_empty() || stmt == ";" {
            continue;
        }

        let effective = strip_leading_comments(stmt);
        let upper = effective.to_uppercase();
        let is_select = upper.starts_with("SELECT")
            || upper.starts_with("WITH")
            || upper.starts_with("EXPLAIN")
            || upper.starts_with("PRAGMA")
            || upper.starts_with("VALUES");

        if is_select {
            match execute_query(&conn, stmt) {
                Ok(rows) => output.push_str(&rows),
                Err(e) => {
                    stderr_output.push_str(&format!("Error: {}\n", e));
                    return Ok(EmbeddedOutput {
                        stdout: output,
                        stderr: stderr_output,
                        exit_code: 1,
                    });
                }
            }
        } else {
            match conn.execute_batch(stmt) {
                Ok(()) => {}
                Err(e) => {
                    stderr_output.push_str(&format!("Error: {}\n", e));
                    return Ok(EmbeddedOutput {
                        stdout: output,
                        stderr: stderr_output,
                        exit_code: 1,
                    });
                }
            }
        }
    }

    Ok(EmbeddedOutput {
        stdout: output,
        stderr: stderr_output,
        exit_code: 0,
    })
}

/// Execute a SELECT query and format results as pipe-separated values.
fn execute_query(
    conn: &rusqlite::Connection,
    sql: &str,
) -> std::result::Result<String, rusqlite::Error> {
    let mut stmt = conn.prepare(sql)?;
    let col_count = stmt.column_count();
    let mut output = String::new();

    let rows = stmt.query_map([], |row| {
        let mut vals = Vec::with_capacity(col_count);
        for i in 0..col_count {
            let val: rusqlite::types::Value = row.get(i)?;
            vals.push(format_value(&val));
        }
        Ok(vals.join("|"))
    })?;

    for row in rows {
        let line = row?;
        output.push_str(&line);
        output.push('\n');
    }

    Ok(output)
}

/// Format a SQLite value for pipe-separated output (matching SQLite CLI behavior).
fn format_value(val: &rusqlite::types::Value) -> String {
    match val {
        rusqlite::types::Value::Null => String::new(),
        rusqlite::types::Value::Integer(i) => i.to_string(),
        rusqlite::types::Value::Real(f) => {
            // Match SQLite CLI: show as integer if whole number, otherwise full precision
            if *f == (*f as i64) as f64 && f.is_finite() {
                format!("{:.1}", f)
            } else {
                f.to_string()
            }
        }
        rusqlite::types::Value::Text(s) => s.clone(),
        rusqlite::types::Value::Blob(b) => {
            // SQLite CLI shows blobs as X'hex'
            let hex: String = b.iter().map(|byte| format!("{:02X}", byte)).collect();
            format!("X'{}'", hex)
        }
    }
}

/// Split SQL text into individual statements by semicolons,
/// respecting string literals (single quotes), line comments (--),
/// and block comments (/* */).
fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = sql.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            current.push(c);
            if c == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_block_comment {
            current.push(c);
            if c == '*' && chars.peek() == Some(&'/') {
                current.push(chars.next().unwrap());
                in_block_comment = false;
            }
            continue;
        }

        if in_single_quote {
            current.push(c);
            if c == '\'' {
                // Check for escaped quote ('')
                if chars.peek() == Some(&'\'') {
                    current.push(chars.next().unwrap());
                } else {
                    in_single_quote = false;
                }
            }
            continue;
        }

        match c {
            '\'' => {
                in_single_quote = true;
                current.push(c);
            }
            '-' if chars.peek() == Some(&'-') => {
                in_line_comment = true;
                current.push(c);
                current.push(chars.next().unwrap());
            }
            '/' if chars.peek() == Some(&'*') => {
                in_block_comment = true;
                current.push(c);
                current.push(chars.next().unwrap());
            }
            ';' => {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    statements.push(trimmed);
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        statements.push(trimmed);
    }

    statements
}

/// Strip leading SQL comments (-- and /* */) and whitespace to find the first keyword.
fn strip_leading_comments(sql: &str) -> String {
    let mut result = sql.trim_start().to_string();
    loop {
        if result.starts_with("--") {
            // Skip to end of line
            if let Some(pos) = result.find('\n') {
                result = result[pos + 1..].trim_start().to_string();
            } else {
                return String::new(); // entire string is a comment
            }
        } else if result.starts_with("/*") {
            // Skip to end of block comment
            if let Some(pos) = result.find("*/") {
                result = result[pos + 2..].trim_start().to_string();
            } else {
                return String::new(); // unclosed block comment
            }
        } else {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_select() {
        let result = execute_sql(
            Some("CREATE TABLE t(id INTEGER, name TEXT); INSERT INTO t VALUES(1, 'Alice');"),
            "SELECT * FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "1|Alice\n");
    }

    #[test]
    fn test_multiple_rows() {
        let result = execute_sql(
            Some("CREATE TABLE t(a INTEGER, b TEXT); INSERT INTO t VALUES(1,'x'); INSERT INTO t VALUES(2,'y');"),
            "SELECT * FROM t;",
        ).unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "1|x\n2|y\n");
    }

    #[test]
    fn test_insert_then_select() {
        let result = execute_sql(
            Some("CREATE TABLE t(id INTEGER);"),
            "INSERT INTO t VALUES(42); SELECT * FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "42\n");
    }

    #[test]
    fn test_syntax_error() {
        let result = execute_sql(None, "SELECTT * FROM nope;").unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(!result.stderr.is_empty());
    }

    #[test]
    fn test_null_value() {
        let result = execute_sql(
            Some("CREATE TABLE t(a INTEGER); INSERT INTO t VALUES(NULL);"),
            "SELECT * FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "\n"); // NULL renders as empty string
    }

    #[test]
    fn test_empty_sql() {
        let result = execute_sql(None, "  ").unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("No SQL"));
    }

    #[test]
    fn test_setup_error() {
        let result = execute_sql(Some("CREATE TABLE INVALID SYNTAX"), "SELECT 1;").unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("Setup error"));
    }

    #[test]
    fn test_multiple_statements() {
        let result = execute_sql(
            Some("CREATE TABLE t(x INTEGER);"),
            "INSERT INTO t VALUES(1); INSERT INTO t VALUES(2); SELECT COUNT(*) FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "2\n");
    }

    #[test]
    fn test_string_with_semicolon() {
        let result = execute_sql(
            Some("CREATE TABLE t(msg TEXT);"),
            "INSERT INTO t VALUES('hello; world'); SELECT * FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "hello; world\n");
    }

    #[test]
    fn test_no_setup() {
        let result = execute_sql(None, "SELECT 1+1;").unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "2\n");
    }

    #[test]
    fn test_split_sql_basic() {
        let stmts = split_sql_statements("SELECT 1; SELECT 2;");
        assert_eq!(stmts, vec!["SELECT 1", "SELECT 2"]);
    }

    #[test]
    fn test_split_sql_string_semicolon() {
        let stmts = split_sql_statements("INSERT INTO t VALUES('a;b'); SELECT 1;");
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[0], "INSERT INTO t VALUES('a;b')");
    }

    #[test]
    fn test_split_sql_comment() {
        let stmts = split_sql_statements("-- comment with ; inside\nSELECT 1;");
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("SELECT 1"));
    }

    #[test]
    fn test_split_sql_block_comment() {
        let stmts = split_sql_statements("SELECT /* skip; this */ 1;");
        assert_eq!(stmts.len(), 1);
        assert!(stmts[0].contains("SELECT"));
    }

    #[test]
    fn test_select_with_leading_comment() {
        let result = execute_sql(
            Some("CREATE TABLE t(id INTEGER, name TEXT); INSERT INTO t VALUES(1, 'Alice');"),
            "-- This is a comment\nSELECT * FROM t;",
        )
        .unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "1|Alice\n");
    }

    #[test]
    fn test_select_with_block_comment() {
        let result = execute_sql(
            Some("CREATE TABLE t(id INTEGER, name TEXT, email TEXT); INSERT INTO t VALUES(1, 'Alice', 'a@b.com');"),
            "SELECT id, /* name, */ email FROM t;",
        ).unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "1|a@b.com\n");
    }

    #[test]
    fn test_strip_leading_comments() {
        assert_eq!(strip_leading_comments("SELECT 1"), "SELECT 1");
        assert_eq!(strip_leading_comments("-- comment\nSELECT 1"), "SELECT 1");
        assert_eq!(strip_leading_comments("/* block */ SELECT 1"), "SELECT 1");
        assert_eq!(
            strip_leading_comments("-- line\n/* block */\nSELECT 1"),
            "SELECT 1"
        );
        assert_eq!(strip_leading_comments("-- only comment"), "");
    }
}
