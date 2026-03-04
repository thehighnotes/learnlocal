use ratatui::style::Style;
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;

/// Render a line-level diff between expected and actual output.
/// Matching lines get no marker. Expected-only lines get `- ` prefix (red).
/// Actual-only lines get `+ ` prefix (yellow).
pub fn render_output_diff<'a>(expected: &str, actual: &str, theme: &Theme) -> Vec<Line<'a>> {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    // Single-line case: character-level highlight of divergence point
    if expected_lines.len() == 1 && actual_lines.len() == 1 {
        return render_single_line_diff(expected_lines[0], actual_lines[0], theme);
    }

    // Multi-line: simple line-level diff using LCS
    let lcs = lcs_lines(&expected_lines, &actual_lines);
    let mut lines = Vec::new();

    let mut ei = 0;
    let mut ai = 0;
    let mut li = 0;

    while ei < expected_lines.len() || ai < actual_lines.len() {
        if li < lcs.len() && ei == lcs[li].0 && ai == lcs[li].1 {
            // Matching line
            lines.push(Line::from(format!("    {}", expected_lines[ei])));
            ei += 1;
            ai += 1;
            li += 1;
        } else {
            // Output expected-only lines before actual-only lines
            if ei < expected_lines.len() && (li >= lcs.len() || ei < lcs[li].0) {
                lines.push(Line::from(Span::styled(
                    format!("  - {}", expected_lines[ei]),
                    Style::default().fg(theme.diff_expected),
                )));
                ei += 1;
            } else if ai < actual_lines.len() && (li >= lcs.len() || ai < lcs[li].1) {
                lines.push(Line::from(Span::styled(
                    format!("  + {}", actual_lines[ai]),
                    Style::default().fg(theme.diff_actual),
                )));
                ai += 1;
            }
        }
    }

    lines
}

/// For single-line mismatches, show expected and actual with caret at divergence point.
fn render_single_line_diff<'a>(expected: &str, actual: &str, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        format!("  - {}", expected),
        Style::default().fg(theme.diff_expected),
    )));
    lines.push(Line::from(Span::styled(
        format!("  + {}", actual),
        Style::default().fg(theme.diff_actual),
    )));

    // Find first diverging position and show caret
    let diverge_pos = expected
        .chars()
        .zip(actual.chars())
        .position(|(a, b)| a != b)
        .unwrap_or(expected.len().min(actual.len()));

    // 4 chars for "  + " prefix, then diverge_pos chars into the string
    let caret_line = format!("  {}^", " ".repeat(diverge_pos + 2));
    lines.push(Line::from(Span::styled(
        caret_line,
        Style::default().fg(theme.diff_actual),
    )));

    lines
}

/// Compute LCS (Longest Common Subsequence) of line indices.
/// Returns vec of (expected_idx, actual_idx) pairs.
fn lcs_lines(expected: &[&str], actual: &[&str]) -> Vec<(usize, usize)> {
    let m = expected.len();
    let n = actual.len();

    // Build DP table
    let mut dp = vec![vec![0u32; n + 1]; m + 1];
    for i in 1..=m {
        for j in 1..=n {
            if expected[i - 1] == actual[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find the pairs
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if expected[i - 1] == actual[j - 1] {
            result.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn test_identical_strings() {
        let lines = render_output_diff("hello\nworld", "hello\nworld", &test_theme());
        // All matching lines, no - or + markers
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            assert!(!text.starts_with("  -"), "unexpected - marker: {}", text);
            assert!(!text.starts_with("  +"), "unexpected + marker: {}", text);
        }
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_single_line_mismatch() {
        let lines = render_output_diff("42", "43", &test_theme());
        assert_eq!(lines.len(), 3); // expected, actual, caret
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        let second: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first.contains("- 42"));
        assert!(second.contains("+ 43"));
    }

    #[test]
    fn test_multi_line_diff() {
        let expected = "line1\nline2\nline3";
        let actual = "line1\nchanged\nline3";
        let lines = render_output_diff(expected, actual, &test_theme());
        // Should show line1 matching, line2 as -, changed as +, line3 matching
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_empty_expected() {
        let lines = render_output_diff("", "output", &test_theme());
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_empty_actual() {
        let lines = render_output_diff("expected", "", &test_theme());
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_no_color_still_has_prefixes() {
        let theme = Theme::no_color();
        let lines = render_output_diff("42", "43", &theme);
        let first: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        let second: String = lines[1].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first.contains("- "));
        assert!(second.contains("+ "));
    }
}
