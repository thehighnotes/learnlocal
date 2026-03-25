use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::community::types::{RegistryCourse, RegistrySource};
use crate::ui::theme::Theme;

/// Format a single course row for the browse list.
pub fn format_course_row(
    course: &RegistryCourse,
    is_selected: bool,
    is_installed: bool,
    width: u16,
    theme: &Theme,
) -> Line<'static> {
    let prefix = if is_installed { "\u{2713} " } else { "  " };
    let name = &course.name;
    let lang = &course.language_display;

    // Truncate name if needed
    let max_name = (width as usize).saturating_sub(6 + lang.len());
    let display_name = if name.len() > max_name {
        format!("{}...", &name[..max_name.saturating_sub(3)])
    } else {
        name.clone()
    };

    let label = format!(
        "{}{:<width$} {}",
        prefix,
        display_name,
        lang,
        width = max_name
    );

    let style = if is_selected {
        Style::default()
            .fg(theme.heading)
            .add_modifier(Modifier::BOLD)
    } else if is_installed {
        Style::default().fg(theme.success)
    } else {
        Style::default().fg(theme.body_text)
    };

    Line::from(Span::styled(label, style))
}

/// Format the detail panel for a selected course.
pub fn format_course_detail(
    course: &RegistryCourse,
    is_installed: bool,
    theme: &Theme,
    max_width: u16,
) -> Vec<Line<'static>> {
    let w = max_width as usize;
    let mut lines = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        format!(" {}", course.name),
        Style::default()
            .fg(theme.heading)
            .add_modifier(Modifier::BOLD),
    )));

    // Version + author
    lines.push(Line::from(vec![
        Span::styled(" v", Style::default().fg(theme.muted)),
        Span::styled(course.version.clone(), Style::default().fg(theme.body_text)),
        Span::styled("  by ", Style::default().fg(theme.muted)),
        Span::styled(course.author.clone(), Style::default().fg(theme.body_text)),
    ]));

    lines.push(Line::from(""));

    // Description (word-wrapped)
    let desc = &course.description;
    let wrap_width = w.saturating_sub(2);
    for line in word_wrap(desc, wrap_width) {
        lines.push(Line::from(Span::styled(
            format!(" {}", line),
            Style::default().fg(theme.body_text),
        )));
    }

    lines.push(Line::from(""));

    // Metadata
    lines.push(Line::from(vec![
        Span::styled(" Language: ", Style::default().fg(theme.muted)),
        Span::styled(
            course.language_display.clone(),
            Style::default().fg(theme.body_text),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled(" Lessons:  ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{}", course.lessons),
            Style::default().fg(theme.body_text),
        ),
        Span::styled(" | Exercises: ", Style::default().fg(theme.muted)),
        Span::styled(
            format!("{}", course.exercises),
            Style::default().fg(theme.body_text),
        ),
    ]));

    if let Some(ref license) = course.license {
        lines.push(Line::from(vec![
            Span::styled(" License:  ", Style::default().fg(theme.muted)),
            Span::styled(license.clone(), Style::default().fg(theme.body_text)),
        ]));
    }

    if !course.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(" Tags:     ", Style::default().fg(theme.muted)),
            Span::styled(course.tags.join(", "), Style::default().fg(theme.body_text)),
        ]));
    }

    if let Some(ref platform) = course.platform {
        lines.push(Line::from(vec![
            Span::styled(" Platform: ", Style::default().fg(theme.muted)),
            Span::styled(platform.clone(), Style::default().fg(theme.body_text)),
        ]));
    }

    if course.has_stages {
        lines.push(Line::from(Span::styled(
            " \u{2605} Has staged exercises",
            Style::default().fg(theme.muted),
        )));
    }

    // Rating
    if let Some(avg) = course.avg_rating {
        if avg > 0.0 {
            let stars = "\u{2605}"
                .repeat(avg.round() as usize)
                .chars()
                .chain("\u{2606}".repeat(5 - avg.round() as usize).chars())
                .collect::<String>();
            let count = course.review_count.unwrap_or(0);
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", stars), Style::default().fg(theme.warning)),
                Span::styled(
                    format!("{:.1} ({} ratings)", avg, count),
                    Style::default().fg(theme.muted),
                ),
            ]));
        }
    }

    // Downloads
    if let Some(downloads) = course.downloads {
        if downloads > 0 {
            lines.push(Line::from(vec![
                Span::styled(" Downloads: ", Style::default().fg(theme.muted)),
                Span::styled(
                    format!("{}", downloads),
                    Style::default().fg(theme.body_text),
                ),
            ]));
        }
    }

    // Fork lineage
    if let Some(ref fork) = course.forked_from {
        lines.push(Line::from(""));
        let fork_text = if let Some(ref author) = fork.author {
            format!(
                " Forked from {} v{} by {}",
                fork.id,
                fork.version.as_deref().unwrap_or("?"),
                author
            )
        } else {
            format!(
                " Forked from {} v{}",
                fork.id,
                fork.version.as_deref().unwrap_or("?")
            )
        };
        lines.push(Line::from(Span::styled(
            fork_text,
            Style::default().fg(theme.muted),
        )));
    }

    lines.push(Line::from(""));

    // Status
    if is_installed {
        lines.push(Line::from(Span::styled(
            " \u{2713} Installed",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            " Press [d] to download",
            Style::default().fg(theme.muted),
        )));
    }

    lines
}

/// Format the search bar line.
pub fn format_search_bar(
    query: &str,
    is_editing: bool,
    result_count: usize,
    source: &RegistrySource,
    theme: &Theme,
    width: u16,
) -> Line<'static> {
    let w = width as usize;
    let source_str = format!("[{}]", source);

    let search_label = if is_editing {
        format!(" Search: {}_ ", query)
    } else if query.is_empty() {
        " Search: (press / to search) ".to_string()
    } else {
        format!(" Search: {} ", query)
    };

    let count_str = format!(" {} courses ", result_count);

    // Calculate padding
    let used = search_label.len() + count_str.len() + source_str.len();
    let padding = w.saturating_sub(used);

    let spans = vec![
        Span::styled(
            search_label,
            if is_editing {
                Style::default()
                    .fg(theme.title_bar_fg)
                    .bg(theme.title_bar_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            },
        ),
        Span::styled("\u{2500}".repeat(padding), Style::default().fg(theme.muted)),
        Span::styled(count_str, Style::default().fg(theme.muted)),
        Span::styled(source_str, Style::default().fg(theme.muted)),
    ];

    if spans.is_empty() {
        return Line::from("");
    }

    Line::from(spans)
}

/// Simple word-wrap: break text into lines of at most `width` characters.
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_wrap_short() {
        let result = word_wrap("hello world", 50);
        assert_eq!(result, vec!["hello world"]);
    }

    #[test]
    fn test_word_wrap_break() {
        let result = word_wrap("hello world foo bar", 11);
        assert_eq!(result, vec!["hello world", "foo bar"]);
    }

    #[test]
    fn test_word_wrap_empty() {
        let result = word_wrap("", 50);
        assert_eq!(result, vec![""]);
    }
}
