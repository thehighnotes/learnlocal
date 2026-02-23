use pulldown_cmark::{Event, Options, Parser, Tag};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use super::theme::Theme;

/// Render a markdown string into styled ratatui Lines.
pub fn render_markdown(md: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(md, options);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();

    let mut in_code_block = false;
    let mut bold = false;
    let mut italic = false;
    let mut in_heading = false;
    let mut heading_level = 0u8;
    let mut in_blockquote = false;
    let mut list_depth = 0u32;
    let mut ordered_list_index: Option<u64> = None;

    // Table state
    let mut in_table = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading(level, _, _) => {
                    in_heading = true;
                    heading_level = level as u8;
                }
                Tag::Paragraph => {}
                Tag::CodeBlock(_) => {
                    in_code_block = true;
                    flush_line(&mut lines, &mut current_spans);
                }
                Tag::Strong => bold = true,
                Tag::Emphasis => italic = true,
                Tag::BlockQuote => {
                    in_blockquote = true;
                }
                Tag::List(start) => {
                    list_depth += 1;
                    ordered_list_index = start;
                }
                Tag::Item => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1) as usize);
                    let bullet = if let Some(ref mut idx) = ordered_list_index {
                        let s = format!("  {}{}. ", indent, idx);
                        *idx += 1;
                        s
                    } else {
                        format!("  {}\u{2022} ", indent)
                    };
                    current_spans.push(Span::styled(bullet, Style::default().fg(theme.body_text)));
                }
                Tag::Table(_alignments) => {
                    in_table = true;
                    table_rows.clear();
                }
                Tag::TableHead => {
                    current_row.clear();
                }
                Tag::TableRow => {
                    current_row.clear();
                }
                Tag::TableCell => {
                    current_cell.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                Tag::Heading(_, _, _) => {
                    in_heading = false;
                    flush_line(&mut lines, &mut current_spans);
                    // Add underline for H1 and H2
                    if heading_level <= 2 {
                        let underline_char = if heading_level == 1 { "\u{2550}" } else { "\u{2500}" };
                        let underline_len = 40;
                        lines.push(Line::from(Span::styled(
                            format!("  {}", underline_char.repeat(underline_len)),
                            Style::default().fg(if heading_level == 1 { theme.heading } else { theme.heading_h2 }),
                        )));
                    }
                    lines.push(Line::from(""));
                }
                Tag::Paragraph => {
                    flush_line(&mut lines, &mut current_spans);
                    lines.push(Line::from(""));
                }
                Tag::CodeBlock(_) => {
                    in_code_block = false;
                    lines.push(Line::from(""));
                }
                Tag::Strong => bold = false,
                Tag::Emphasis => italic = false,
                Tag::BlockQuote => {
                    in_blockquote = false;
                }
                Tag::List(_) => {
                    list_depth = list_depth.saturating_sub(1);
                    if list_depth == 0 {
                        ordered_list_index = None;
                    }
                    lines.push(Line::from(""));
                }
                Tag::Item => {
                    flush_line(&mut lines, &mut current_spans);
                }
                Tag::Table(_) => {
                    in_table = false;
                    render_table(&mut lines, &table_rows, theme);
                    table_rows.clear();
                    lines.push(Line::from(""));
                }
                Tag::TableHead => {
                    table_rows.push(current_row.clone());
                    current_row.clear();
                }
                Tag::TableRow => {
                    table_rows.push(current_row.clone());
                    current_row.clear();
                }
                Tag::TableCell => {
                    current_row.push(current_cell.clone());
                    current_cell.clear();
                }
                _ => {}
            },
            Event::Text(text) => {
                let text_str = text.to_string();

                if in_table {
                    current_cell.push_str(&text_str);
                } else if in_code_block {
                    // Render code block lines with left border gutter
                    for code_line in text_str.lines() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "  \u{2502} ".to_string(),
                                Style::default().fg(theme.code_border),
                            ),
                            Span::styled(
                                code_line.to_string(),
                                Style::default().fg(theme.code),
                            ),
                        ]));
                    }
                } else if in_heading {
                    let (color, mods) = match heading_level {
                        1 => (theme.heading, Modifier::BOLD | Modifier::UNDERLINED),
                        2 => (theme.heading_h2, Modifier::BOLD),
                        _ => (theme.heading_h3, Modifier::BOLD),
                    };
                    current_spans.push(Span::styled(
                        format!("  {}", text_str),
                        Style::default().fg(color).add_modifier(mods),
                    ));
                } else {
                    let mut style = Style::default().fg(theme.body_text);
                    if bold {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if italic {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if in_blockquote {
                        let quoted = format!("  \u{2502} {}", text_str);
                        current_spans.push(Span::styled(
                            quoted,
                            style.fg(theme.muted),
                        ));
                    } else {
                        current_spans.push(Span::styled(format!("  {}", text_str), style));
                    }
                }
            }
            Event::Code(code) => {
                if in_table {
                    current_cell.push_str(&format!("`{}`", code));
                } else {
                    current_spans.push(Span::styled(
                        format!(" {} ", code),
                        Style::default().fg(theme.keyword),
                    ));
                }
            }
            Event::HardBreak | Event::SoftBreak => {
                if in_table {
                    current_cell.push(' ');
                } else {
                    flush_line(&mut lines, &mut current_spans);
                }
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_spans);
                lines.push(Line::from(Span::styled(
                    format!("  {}", "\u{2500}".repeat(40)),
                    Style::default().fg(theme.muted),
                )));
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    flush_line(&mut lines, &mut current_spans);
    lines
}

fn flush_line(lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>) {
    if !spans.is_empty() {
        lines.push(Line::from(std::mem::take(spans)));
    }
}

/// Render a table with box-drawing characters.
/// First row is the header.
fn render_table(lines: &mut Vec<Line<'static>>, rows: &[Vec<String>], theme: &Theme) {
    if rows.is_empty() {
        return;
    }

    // Compute column widths
    let num_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return;
    }

    let mut col_widths = vec![0usize; num_cols];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < num_cols {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }
    // Ensure minimum column width of 3
    for w in &mut col_widths {
        *w = (*w).max(3);
    }

    let border_style = Style::default().fg(theme.table_border);
    let header_style = Style::default().fg(theme.heading).add_modifier(Modifier::BOLD);
    let cell_style = Style::default().fg(theme.body_text);

    // Top border: ┌──────┬──────┐
    let top = format!(
        "  \u{250C}{}\u{2510}",
        col_widths
            .iter()
            .map(|w| "\u{2500}".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("\u{252C}")
    );
    lines.push(Line::from(Span::styled(top, border_style)));

    for (row_idx, row) in rows.iter().enumerate() {
        // Data row: │ cell │ cell │
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::styled("  \u{2502}".to_string(), border_style));
        for (i, width) in col_widths.iter().enumerate() {
            let cell_text = row.get(i).map(|s| s.as_str()).unwrap_or("");
            let padded = format!(" {:<width$} ", cell_text, width = width);
            let style = if row_idx == 0 { header_style } else { cell_style };
            spans.push(Span::styled(padded, style));
            spans.push(Span::styled("\u{2502}".to_string(), border_style));
        }
        lines.push(Line::from(spans));

        // After header row: ├──────┼──────┤
        if row_idx == 0 && rows.len() > 1 {
            let sep = format!(
                "  \u{251C}{}\u{2524}",
                col_widths
                    .iter()
                    .map(|w| "\u{2500}".repeat(w + 2))
                    .collect::<Vec<_>>()
                    .join("\u{253C}")
            );
            lines.push(Line::from(Span::styled(sep, border_style)));
        }
    }

    // Bottom border: └──────┴──────┘
    let bottom = format!(
        "  \u{2514}{}\u{2518}",
        col_widths
            .iter()
            .map(|w| "\u{2500}".repeat(w + 2))
            .collect::<Vec<_>>()
            .join("\u{2534}")
    );
    lines.push(Line::from(Span::styled(bottom, border_style)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heading() {
        let theme = Theme::default();
        let lines = render_markdown("# Hello", &theme);
        assert!(!lines.is_empty());
        let first = &lines[0];
        let text: String = first.spans.iter().map(|s| s.content.to_string()).collect();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_heading_h2_has_underline() {
        let theme = Theme::default();
        let lines = render_markdown("## Section", &theme);
        // Should have: heading line, underline line, blank line
        assert!(lines.len() >= 2);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(all_text.contains("Section"));
        assert!(all_text.contains("\u{2500}")); // H2 underline
    }

    #[test]
    fn test_heading_h1_has_double_underline() {
        let theme = Theme::default();
        let lines = render_markdown("# Title", &theme);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(all_text.contains("Title"));
        assert!(all_text.contains("\u{2550}")); // H1 double underline
    }

    #[test]
    fn test_code_block() {
        let theme = Theme::default();
        let md = "```cpp\nint x = 5;\n```";
        let lines = render_markdown(md, &theme);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(all_text.contains("int x = 5;"));
        assert!(all_text.contains("\u{2502}")); // left border gutter
    }

    #[test]
    fn test_bold_text() {
        let theme = Theme::default();
        let lines = render_markdown("This is **bold** text", &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_list() {
        let theme = Theme::default();
        let md = "- item 1\n- item 2\n- item 3";
        let lines = render_markdown(md, &theme);
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_horizontal_rule() {
        let theme = Theme::default();
        let lines = render_markdown("---", &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_table_rendering() {
        let theme = Theme::default();
        let md = "| Type | Size |\n|------|------|\n| int | 4 bytes |\n| char | 1 byte |";
        let lines = render_markdown(md, &theme);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        // Should contain table border characters
        assert!(all_text.contains("\u{250C}")); // top-left corner
        assert!(all_text.contains("\u{2518}")); // bottom-right corner
        assert!(all_text.contains("Type"));
        assert!(all_text.contains("int"));
    }

    #[test]
    fn test_inline_code() {
        let theme = Theme::default();
        let lines = render_markdown("Use `int` for integers", &theme);
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.to_string())
            .collect::<Vec<_>>()
            .join("");
        assert!(all_text.contains("int"));
    }
}
