use regex::Regex;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;

#[derive(Debug, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

#[derive(Debug)]
pub struct Diagnostic {
    pub location: Option<String>,
    pub level: DiagnosticLevel,
    pub message: String,
    pub context_lines: Vec<String>,
}

#[derive(Debug)]
pub enum ParsedOutput {
    Structured {
        diagnostics: Vec<Diagnostic>,
        error_count: usize,
        warning_count: usize,
    },
    Raw(String),
}

/// Parse g++/clang-style compiler output into structured diagnostics.
/// Falls back to Raw for unrecognized formats.
pub fn parse_compiler_output(stderr: &str) -> ParsedOutput {
    let header_re =
        Regex::new(r"^(.+?:\d+:\d+):\s+(error|warning|note|fatal error):\s+(.*)$").unwrap();
    let context_re = Regex::new(r"^\s+\d+\s*\|").unwrap();
    let caret_re = Regex::new(r"^\s+\|").unwrap();

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    for line in stderr.lines() {
        if let Some(caps) = header_re.captures(line) {
            let location = caps.get(1).unwrap().as_str().to_string();
            let level_str = caps.get(2).unwrap().as_str();
            let message = caps.get(3).unwrap().as_str().to_string();

            let level = match level_str {
                "error" | "fatal error" => {
                    error_count += 1;
                    DiagnosticLevel::Error
                }
                "warning" => {
                    warning_count += 1;
                    DiagnosticLevel::Warning
                }
                _ => DiagnosticLevel::Note,
            };

            diagnostics.push(Diagnostic {
                location: Some(location),
                level,
                message,
                context_lines: Vec::new(),
            });
        } else if !diagnostics.is_empty()
            && (context_re.is_match(line) || caret_re.is_match(line))
        {
            diagnostics.last_mut().unwrap().context_lines.push(line.to_string());
        }
        // Skip lines that don't match either pattern (linker summaries, etc.)
    }

    if diagnostics.is_empty() {
        ParsedOutput::Raw(stderr.to_string())
    } else {
        ParsedOutput::Structured {
            diagnostics,
            error_count,
            warning_count,
        }
    }
}

/// Render parsed compiler output into styled ratatui Lines.
pub fn render_diagnostics<'a>(parsed: &ParsedOutput, theme: &Theme) -> Vec<Line<'a>> {
    match parsed {
        ParsedOutput::Structured {
            diagnostics,
            error_count,
            warning_count,
        } => render_structured(diagnostics, *error_count, *warning_count, theme),
        ParsedOutput::Raw(text) => render_raw(text, theme),
    }
}

fn render_structured<'a>(
    diagnostics: &[Diagnostic],
    error_count: usize,
    warning_count: usize,
    theme: &Theme,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    // Summary line
    let summary = format!("  {} error(s), {} warning(s)", error_count, warning_count);
    lines.push(Line::from(Span::styled(
        summary,
        Style::default().fg(theme.muted),
    )));
    lines.push(Line::from(""));

    for (i, diag) in diagnostics.iter().enumerate() {
        // Location
        if let Some(ref loc) = diag.location {
            lines.push(Line::from(Span::styled(
                format!("  {}", loc),
                Style::default().fg(theme.muted),
            )));
        }

        // Level + message on one line
        let (level_str, level_color) = match diag.level {
            DiagnosticLevel::Error => ("error", theme.error),
            DiagnosticLevel::Warning => ("warning", theme.diff_actual),
            DiagnosticLevel::Note => ("note", theme.muted),
        };

        let level_style = if diag.level == DiagnosticLevel::Error {
            Style::default().fg(level_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(level_color)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {}", level_str), level_style),
            Span::styled(
                format!(": {}", diag.message),
                Style::default().fg(theme.body_text),
            ),
        ]));

        // Source context lines
        let context_color = match diag.level {
            DiagnosticLevel::Error => theme.error,
            DiagnosticLevel::Warning => theme.diff_actual,
            DiagnosticLevel::Note => theme.muted,
        };

        for ctx in &diag.context_lines {
            // Source lines (with digit|) in code color, caret lines in severity color
            if ctx.trim_start().starts_with('|') {
                // Caret/underline line
                lines.push(Line::from(Span::styled(
                    format!("  {}", ctx),
                    Style::default().fg(context_color),
                )));
            } else {
                // Source code line
                lines.push(Line::from(Span::styled(
                    format!("  {}", ctx),
                    Style::default().fg(theme.code),
                )));
            }
        }

        // Blank line between diagnostics (not after last)
        if i < diagnostics.len() - 1 {
            lines.push(Line::from(""));
        }
    }

    lines
}

fn render_raw<'a>(text: &str, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled(
        "  Error output:",
        Style::default().add_modifier(Modifier::BOLD),
    )));
    for line in text.lines() {
        lines.push(Line::from(Span::styled(
            format!("  {}", line),
            Style::default().fg(theme.error),
        )));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_parse_gcc_error() {
        let stderr = "main.cpp:5:12: error: expected ';' after expression\n    5 |     int x = 10\n      |            ^\n";
        let parsed = parse_compiler_output(stderr);
        match parsed {
            ParsedOutput::Structured {
                diagnostics,
                error_count,
                warning_count,
            } => {
                assert_eq!(error_count, 1);
                assert_eq!(warning_count, 0);
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
                assert_eq!(diagnostics[0].location.as_deref(), Some("main.cpp:5:12"));
                assert_eq!(diagnostics[0].message, "expected ';' after expression");
                assert_eq!(diagnostics[0].context_lines.len(), 2);
            }
            ParsedOutput::Raw(_) => panic!("expected Structured"),
        }
    }

    #[test]
    fn test_parse_multiple_diagnostics() {
        let stderr = "\
main.cpp:3:5: warning: unused variable 'x' [-Wunused-variable]
    3 |     int x = 10;
      |     ^~~
main.cpp:7:1: error: expected '}' at end of input
    7 | }
      | ^
";
        let parsed = parse_compiler_output(stderr);
        match parsed {
            ParsedOutput::Structured {
                diagnostics,
                error_count,
                warning_count,
            } => {
                assert_eq!(error_count, 1);
                assert_eq!(warning_count, 1);
                assert_eq!(diagnostics.len(), 2);
                assert_eq!(diagnostics[0].level, DiagnosticLevel::Warning);
                assert_eq!(diagnostics[1].level, DiagnosticLevel::Error);
            }
            ParsedOutput::Raw(_) => panic!("expected Structured"),
        }
    }

    #[test]
    fn test_parse_fatal_error() {
        let stderr = "main.cpp:1:10: fatal error: nosuchheader.h: No such file or directory\n";
        let parsed = parse_compiler_output(stderr);
        match parsed {
            ParsedOutput::Structured {
                error_count,
                diagnostics,
                ..
            } => {
                assert_eq!(error_count, 1);
                assert_eq!(diagnostics[0].level, DiagnosticLevel::Error);
            }
            ParsedOutput::Raw(_) => panic!("expected Structured"),
        }
    }

    #[test]
    fn test_parse_note() {
        let stderr = "\
main.cpp:5:12: error: use of undeclared identifier 'foo'
    5 |     foo();
      |     ^
main.cpp:2:6: note: did you mean 'bar'?
    2 | void bar() {}
      |      ^
";
        let parsed = parse_compiler_output(stderr);
        match parsed {
            ParsedOutput::Structured {
                diagnostics,
                error_count,
                ..
            } => {
                assert_eq!(error_count, 1);
                assert_eq!(diagnostics.len(), 2);
                assert_eq!(diagnostics[1].level, DiagnosticLevel::Note);
            }
            ParsedOutput::Raw(_) => panic!("expected Structured"),
        }
    }

    #[test]
    fn test_parse_unrecognized_format_falls_back_to_raw() {
        let stderr = "error[E0308]: mismatched types\n --> src/main.rs:5:12\n";
        let parsed = parse_compiler_output(stderr);
        match parsed {
            ParsedOutput::Raw(text) => assert_eq!(text, stderr),
            ParsedOutput::Structured { .. } => panic!("expected Raw fallback for rustc output"),
        }
    }

    #[test]
    fn test_parse_empty_input() {
        let parsed = parse_compiler_output("");
        match parsed {
            ParsedOutput::Raw(text) => assert_eq!(text, ""),
            ParsedOutput::Structured { .. } => panic!("expected Raw for empty input"),
        }
    }

    #[test]
    fn test_render_structured_has_summary() {
        let stderr = "main.cpp:5:12: error: expected ';'\n";
        let parsed = parse_compiler_output(stderr);
        let lines = render_diagnostics(&parsed, &test_theme());
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first.contains("1 error(s)"));
        assert!(first.contains("0 warning(s)"));
    }

    #[test]
    fn test_render_raw_shows_error_header() {
        let stderr = "some random compiler output\n";
        let parsed = parse_compiler_output(stderr);
        let lines = render_diagnostics(&parsed, &test_theme());
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first.contains("Error output:"));
    }

    #[test]
    fn test_render_no_color_mode() {
        let stderr = "main.cpp:1:1: error: test\n";
        let parsed = parse_compiler_output(stderr);
        let theme = Theme::no_color();
        let lines = render_diagnostics(&parsed, &theme);
        // Should still produce lines without panic
        assert!(!lines.is_empty());
    }
}
