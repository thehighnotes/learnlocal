use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::state::progress::ProgressStore;
use crate::state::types::*;
use crate::ui::theme::Theme;

/// Stats computed for a completed course.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CourseStats {
    pub total_exercises: usize,
    pub completed_exercises: usize,
    pub skipped_exercises: usize,
    pub total_attempts: usize,
    pub first_try_count: usize,
    pub hint_free_count: usize,
    pub total_time_seconds: u64,
}

impl CourseStats {
    /// Compute stats from progress store for a given course.
    pub fn compute(
        store: &ProgressStore,
        course_id: &str,
        version: &str,
        _total_lessons: usize,
        total_exercises: usize,
    ) -> Self {
        let key = progress_key(course_id, version);
        let mut completed = 0usize;
        let mut skipped = 0usize;
        let mut total_attempts = 0usize;
        let mut first_try = 0usize;
        let mut hint_free = 0usize;
        let mut total_time = 0u64;

        if let Some(cp) = store.data.courses.get(&key) {
            for lp in cp.lessons.values() {
                for ep in lp.exercises.values() {
                    match ep.status {
                        ProgressStatus::Completed => completed += 1,
                        ProgressStatus::Skipped => skipped += 1,
                        _ => {}
                    }
                    total_attempts += ep.attempts.len();

                    // First try: completed with exactly 1 attempt
                    if ep.status == ProgressStatus::Completed && ep.attempts.len() == 1 {
                        first_try += 1;
                    }

                    // Hint-free: completed with 0 hints in all attempts
                    if ep.status == ProgressStatus::Completed
                        && ep.attempts.iter().all(|a| a.hints_revealed == 0)
                    {
                        hint_free += 1;
                    }

                    for attempt in &ep.attempts {
                        total_time += attempt.time_spent_seconds;
                    }
                }
            }
        }

        Self {
            total_exercises,
            completed_exercises: completed,
            skipped_exercises: skipped,
            total_attempts,
            first_try_count: first_try,
            hint_free_count: hint_free,
            total_time_seconds: total_time,
        }
    }
}

/// Format a duration in seconds to human-readable form.
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let m = seconds / 60;
        let s = seconds % 60;
        if s == 0 {
            format!("{}m", m)
        } else {
            format!("{}m {}s", m, s)
        }
    } else {
        let h = seconds / 3600;
        let m = (seconds % 3600) / 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h {}m", h, m)
        }
    }
}

/// Build a mini progress bar string: `[====---]` style.
pub fn mini_progress_bar(completed: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return format!("[{}]", "-".repeat(width));
    }
    let filled = (completed * width) / total;
    let empty = width - filled;
    format!(
        "[{}{}]",
        "\u{2550}".repeat(filled),   // ═
        "\u{2500}".repeat(empty),     // ─
    )
}

/// Per-course stats for the aggregate dashboard.
#[derive(Debug, Clone)]
pub struct PerCourseStats {
    pub name: String,
    pub exercises_done: usize,
    pub exercises_total: usize,
    pub time_seconds: u64,
    pub started_date: String,
    pub completed: bool,
}

/// Aggregate stats across all courses.
#[derive(Debug, Clone)]
pub struct AggregateStats {
    pub courses_started: usize,
    pub courses_completed: usize,
    pub exercises_completed: usize,
    pub exercises_total: usize,
    pub total_time_seconds: u64,
    pub first_try_count: usize,
    pub hint_free_count: usize,
    pub per_course: Vec<PerCourseStats>,
}

impl AggregateStats {
    /// Compute aggregate stats from course summaries and progress store.
    pub fn compute(
        summaries: &[crate::ui::screens::CourseProgressSummary],
        store: &ProgressStore,
    ) -> Self {
        let mut courses_started = 0usize;
        let mut courses_completed = 0usize;
        let mut exercises_completed = 0usize;
        let mut exercises_total = 0usize;
        let mut total_time = 0u64;
        let mut first_try = 0usize;
        let mut hint_free = 0usize;
        let mut per_course = Vec::new();

        for summary in summaries {
            let info = &summary.info;
            let course_id = info.name.to_lowercase().replace(' ', "-");
            let key = crate::state::types::progress_key(&course_id, &info.version);

            let total_ex = summary.total_exercises;
            exercises_total += total_ex;

            let mut course_done = 0usize;
            let mut course_time = 0u64;
            let mut started_date = String::new();
            let mut is_completed = false;

            if let Some(cp) = store.data.courses.get(&key) {
                courses_started += 1;
                started_date = if cp.started_at.len() >= 10 {
                    cp.started_at[..10].to_string()
                } else {
                    cp.started_at.clone()
                };

                for lp in cp.lessons.values() {
                    for ep in lp.exercises.values() {
                        if ep.status == ProgressStatus::Completed {
                            course_done += 1;
                        }
                        if ep.status == ProgressStatus::Completed && ep.attempts.len() == 1 {
                            first_try += 1;
                        }
                        if ep.status == ProgressStatus::Completed
                            && ep.attempts.iter().all(|a| a.hints_revealed == 0)
                        {
                            hint_free += 1;
                        }
                        for attempt in &ep.attempts {
                            course_time += attempt.time_spent_seconds;
                        }
                    }
                }

                // Check if all lessons are completed
                let completed_lessons = cp.lessons.values()
                    .filter(|lp| lp.status == ProgressStatus::Completed)
                    .count();
                if completed_lessons >= info.lesson_count && info.lesson_count > 0 {
                    courses_completed += 1;
                    is_completed = true;
                }
            }

            exercises_completed += course_done;
            total_time += course_time;

            per_course.push(PerCourseStats {
                name: info.name.clone(),
                exercises_done: course_done,
                exercises_total: total_ex,
                time_seconds: course_time,
                started_date,
                completed: is_completed,
            });
        }

        Self {
            courses_started,
            courses_completed,
            exercises_completed,
            exercises_total,
            total_time_seconds: total_time,
            first_try_count: first_try,
            hint_free_count: hint_free,
            per_course,
        }
    }
}

/// Exercise success flash art (centered).
pub fn exercise_success_art(
    exercise_idx: usize,
    total_exercises: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let bar = mini_progress_bar(exercise_idx + 1, total_exercises, 10);
    let progress_str = format!("Exercise {}/{} {}", exercise_idx + 1, total_exercises, bar);
    let green = if theme.no_color { Color::Reset } else { theme.success };

    vec![
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "    \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            Style::default().fg(green),
        )),
        Line::from(Span::styled(
            "       \u{2713}  PASS",
            Style::default()
                .fg(green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}",
            Style::default().fg(green),
        )),
        Line::from(Span::styled(
            format!("       {}", progress_str),
            Style::default().fg(if theme.no_color { Color::Reset } else { theme.muted }),
        )),
        Line::from(""),
    ]
}

/// Lesson complete art with progress bar.
pub fn lesson_complete_art(
    lesson_title: &str,
    lesson_idx: usize,
    total_lessons: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let green = if theme.no_color { Color::Reset } else { theme.success };
    let body = if theme.no_color { Color::Reset } else { theme.body_text };

    let bar_width = 10usize;
    let filled = if total_lessons > 0 {
        ((lesson_idx + 1) * bar_width) / total_lessons
    } else {
        0
    };
    let empty = bar_width - filled;
    let progress_bar = format!(
        "{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
    );
    let progress_text = format!("{}/{} lessons", lesson_idx + 1, total_lessons);

    // Truncate title if too long
    let title = if lesson_title.len() > 24 {
        format!("{}...", &lesson_title[..21])
    } else {
        lesson_title.to_string()
    };

    vec![
        Line::from(""),
        Line::from(Span::styled(
            "  \u{250C}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            Style::default().fg(green),
        )),
        Line::from(Span::styled(
            format!("  \u{2502}  \u{2605} Lesson Complete{:>9}\u{2502}", ""),
            Style::default().fg(green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  \u{2502}  {:<24}\u{2502}", title),
            Style::default().fg(body),
        )),
        Line::from(Span::styled(
            format!("  \u{2502}  {} {:>4} \u{2502}", progress_bar, progress_text),
            Style::default().fg(body),
        )),
        Line::from(Span::styled(
            "  \u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}",
            Style::default().fg(green),
        )),
        Line::from(""),
    ]
}

/// Course complete art with big celebration and stats.
pub fn course_complete_art(
    course_name: &str,
    stats: &CourseStats,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let green = if theme.no_color { Color::Reset } else { theme.success };
    let cyan = if theme.no_color { Color::Reset } else { Color::Cyan };
    let body = if theme.no_color { Color::Reset } else { theme.body_text };
    let muted = if theme.no_color { Color::Reset } else { theme.muted };

    let star_line = if theme.no_color {
        "     * * *  COMPLETE  * * *"
    } else {
        "     \u{2605} \u{2605} \u{2605}  COMPLETE  \u{2605} \u{2605} \u{2605}"
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  \u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}",
            Style::default().fg(green),
        )),
        Line::from(Span::styled(
            "  \u{2551}                              \u{2551}",
            Style::default().fg(green),
        )),
        Line::from(Span::styled(
            format!("  \u{2551}{:^30}\u{2551}", star_line),
            Style::default().fg(green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "  \u{2551}                              \u{2551}",
            Style::default().fg(green),
        )),
    ];

    // Course name - center it within the box
    let name_display = if course_name.len() > 28 {
        format!("{}...", &course_name[..25])
    } else {
        course_name.to_string()
    };
    lines.push(Line::from(Span::styled(
        format!("  \u{2551}  {:<28}\u{2551}", name_display),
        Style::default().fg(cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "  \u{2551}                              \u{2551}",
        Style::default().fg(green),
    )));
    lines.push(Line::from(Span::styled(
        "  \u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}",
        Style::default().fg(green),
    )));

    // Stats section
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Your Journey:",
        Style::default().fg(body).add_modifier(Modifier::BOLD),
    )));

    lines.push(Line::from(Span::styled(
        format!("    {} exercises completed", stats.completed_exercises),
        Style::default().fg(body),
    )));
    if stats.skipped_exercises > 0 {
        lines.push(Line::from(Span::styled(
            format!("    {} skipped", stats.skipped_exercises),
            Style::default().fg(muted),
        )));
    }
    lines.push(Line::from(Span::styled(
        format!("    {} on first try", stats.first_try_count),
        Style::default().fg(body),
    )));
    lines.push(Line::from(Span::styled(
        format!("    {} without hints", stats.hint_free_count),
        Style::default().fg(body),
    )));
    lines.push(Line::from(Span::styled(
        format!("    Total time: {}", format_duration(stats.total_time_seconds)),
        Style::default().fg(body),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Press [Esc] to return home",
        Style::default().fg(muted),
    )));
    lines.push(Line::from(""));

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(30), "30s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(125), "2m 5s");
        assert_eq!(format_duration(120), "2m");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3661), "1h 1m");
        assert_eq!(format_duration(7200), "2h");
    }

    #[test]
    fn test_mini_progress_bar() {
        let bar = mini_progress_bar(3, 10, 10);
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        // 3/10 of 10 width = 3 filled, 7 empty
        assert_eq!(bar.chars().count(), 12); // [ + 10 chars + ]
    }

    #[test]
    fn test_mini_progress_bar_zero() {
        let bar = mini_progress_bar(0, 0, 5);
        assert_eq!(bar, "[-----]");
    }

    #[test]
    fn test_exercise_success_art_has_content() {
        let theme = Theme::new();
        let lines = exercise_success_art(2, 7, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_lesson_complete_art_has_content() {
        let theme = Theme::new();
        let lines = lesson_complete_art("Variables and Types", 3, 8, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_course_complete_art_has_content() {
        let theme = Theme::new();
        let stats = CourseStats {
            total_exercises: 55,
            completed_exercises: 55,
            skipped_exercises: 0,
            total_attempts: 72,
            first_try_count: 40,
            hint_free_count: 45,
            total_time_seconds: 9240,
        };
        let lines = course_complete_art("C++ Fundamentals", &stats, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_course_stats_empty_store() {
        let store = ProgressStore::empty();
        let stats = CourseStats::compute(&store, "test", "1.0.0", 8, 55);
        assert_eq!(stats.completed_exercises, 0);
        assert_eq!(stats.total_exercises, 55);
    }

    #[test]
    fn test_no_color_mode() {
        let theme = Theme::no_color();
        let lines = exercise_success_art(0, 5, &theme);
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_aggregate_stats_empty_store() {
        let store = ProgressStore::empty();
        let summaries: Vec<crate::ui::screens::CourseProgressSummary> = Vec::new();
        let stats = AggregateStats::compute(&summaries, &store);
        assert_eq!(stats.courses_started, 0);
        assert_eq!(stats.courses_completed, 0);
        assert_eq!(stats.exercises_completed, 0);
        assert_eq!(stats.exercises_total, 0);
        assert_eq!(stats.total_time_seconds, 0);
        assert_eq!(stats.first_try_count, 0);
        assert_eq!(stats.hint_free_count, 0);
        assert!(stats.per_course.is_empty());
    }

    #[test]
    fn test_aggregate_stats_populated() {
        use crate::ui::screens::{CourseProgressSummary, CourseStatus};
        use crate::course::types::CourseInfo;
        use crate::state::types::*;
        use std::collections::HashMap;
        use std::path::PathBuf;

        let mut store = ProgressStore::empty();

        // Add a course with some exercises completed
        let mut lessons = HashMap::new();
        let mut exercises = HashMap::new();
        exercises.insert("ex1".to_string(), ExerciseProgress {
            status: ProgressStatus::Completed,
            attempts: vec![AttemptRecord {
                timestamp: "2026-01-15T10:00:00Z".to_string(),
                time_spent_seconds: 60,
                compile_success: true,
                run_exit_code: Some(0),
                output_matched: Some(true),
                hints_revealed: 0,
            }],
        });
        exercises.insert("ex2".to_string(), ExerciseProgress {
            status: ProgressStatus::Completed,
            attempts: vec![
                AttemptRecord {
                    timestamp: "2026-01-15T10:01:00Z".to_string(),
                    time_spent_seconds: 30,
                    compile_success: false,
                    run_exit_code: None,
                    output_matched: None,
                    hints_revealed: 1,
                },
                AttemptRecord {
                    timestamp: "2026-01-15T10:02:00Z".to_string(),
                    time_spent_seconds: 45,
                    compile_success: true,
                    run_exit_code: Some(0),
                    output_matched: Some(true),
                    hints_revealed: 0,
                },
            ],
        });
        lessons.insert("lesson-1".to_string(), LessonProgress {
            status: ProgressStatus::Completed,
            completed_at: Some("2026-01-15T10:05:00Z".to_string()),
            exercises,
        });

        store.data.courses.insert("test-course@1".to_string(), CourseProgress {
            course_version: "1.0.0".to_string(),
            started_at: "2026-01-15T09:00:00Z".to_string(),
            last_activity: "2026-01-15T10:05:00Z".to_string(),
            lessons,
        });

        let summaries = vec![CourseProgressSummary {
            info: CourseInfo {
                dir_name: "test-course".to_string(),
                name: "Test Course".to_string(),
                description: "A test".to_string(),
                author: "Tester".to_string(),
                version: "1.0.0".to_string(),
                license: None,
                platform: None,
                language_name: "Rust".to_string(),
                lesson_count: 1,
                lesson_ids: vec!["lesson-1".to_string()],
                lesson_titles: vec!["Lesson 1".to_string()],
                source_dir: PathBuf::new(),
                step_commands: Vec::new(),
                env_commands: Vec::new(),
                estimated_minutes_per_lesson: None,
                total_exercise_count: Some(2),
                provision: crate::course::types::Provision::default(),
            },
            status: CourseStatus::Completed,
            completed_lessons: 1,
            total_lessons: 1,
            completed_exercises: 2,
            total_exercises: 2,
        }];

        let stats = AggregateStats::compute(&summaries, &store);
        assert_eq!(stats.courses_started, 1);
        assert_eq!(stats.courses_completed, 1);
        assert_eq!(stats.exercises_completed, 2);
        assert_eq!(stats.exercises_total, 2);
        assert_eq!(stats.total_time_seconds, 135); // 60 + 30 + 45
        assert_eq!(stats.first_try_count, 1); // ex1 completed in 1 attempt
        assert_eq!(stats.hint_free_count, 1); // ex1 had 0 hints; ex2 had hint in first attempt
        assert_eq!(stats.per_course.len(), 1);
        assert_eq!(stats.per_course[0].name, "Test Course");
        assert!(stats.per_course[0].completed);
        assert_eq!(stats.per_course[0].started_date, "2026-01-15");
    }
}
