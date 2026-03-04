use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::course::types::{
    Course, EnvironmentSpec, Exercise, ExerciseFile, ExerciseType, StateAssertion, ValidationMethod,
};
use crate::error::Result;
use crate::exec::environment;
use crate::exec::runner::{self, ExecutionResult};
use crate::exec::sandbox::{Sandbox, SandboxLevel};
use crate::exec::validate;
use crate::state::progress::ProgressStore;
use crate::state::signals::SessionState;
use crate::state::types::*;
use crate::ui::celebration::{self, CourseStats};
use crate::ui::inline_editor::InlineEditorState;
use crate::ui::markdown;
use crate::ui::screens::CourseAction;
use crate::ui::shell::{ShellHistoryEntry, ShellState};
use crate::ui::terminal;
use crate::ui::theme::Theme;
use crate::ui::watch::WatchState;

#[cfg(feature = "llm")]
use crate::llm::channel::{LlmChannel, LlmEvent, LlmRequest};
#[cfg(feature = "llm")]
use crate::llm::chat::{ChatMessage, ChatRole, ChatState};
#[cfg(feature = "llm")]
use crate::llm::config::LlmConfig;
#[cfg(feature = "llm")]
use crate::llm::context::LlmContext;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AppState {
    LessonContent,
    ExercisePrompt,
    Editing,
    Executing,
    RunResult,
    ResultSuccess,
    ResultFail,
    LessonRecap,
    CourseComplete,
    Watching,
    Sandbox,
    Shell,
}

#[derive(Debug, Clone)]
pub enum FailureDetail {
    Plain(String),
    OutputMismatch {
        expected: String,
        actual: String,
    },
    RegexMismatch {
        pattern: String,
        actual: String,
    },
    StateAssertionFailed {
        results: Vec<crate::exec::environment::AssertionResult>,
    },
    InfrastructureFailed {
        phase: String,
        detail: String,
    },
}

pub struct CourseApp {
    pub course: Course,
    pub current_lesson_idx: usize,
    pub current_exercise_idx: usize,
    pub state: AppState,
    pub session: SessionState,
    pub sandbox_level: SandboxLevel,
    pub scroll_offset: u16,
    pub content_line_count: u16,
    pub viewport_height: u16,
    pub last_error: Option<String>,
    pub last_step_name: Option<String>,
    pub failure_detail: Option<FailureDetail>,
    pub last_run_output: Option<runner::RunOutput>,
    pub teardown_warnings: Vec<String>,
    pub animation_start: Option<Instant>,
    pub show_help: bool,
    pub help_scroll_offset: u16,
    pub inline_editor: Option<InlineEditorState>,
    pub editing: bool,
    pub last_input_time: Instant,
    pub shown_quickstart: bool,
    pub course_complete_stats: Option<CourseStats>,
    pub watch_state: Option<WatchState>,
    // Sandbox fields
    pub sandbox_code: Vec<crate::course::types::ExerciseFile>,
    pub sandbox_last_output: Option<runner::RunOutput>,
    pub sandbox_lesson_idx: usize,
    pub sandbox_editing: bool,
    pub sandbox_watching: bool,
    // Shell mode state (command exercises)
    pub shell_state: Option<ShellState>,
    // Assertion results from last execution (for post-run checklist)
    pub last_assertion_results: Option<Vec<crate::exec::environment::AssertionResult>>,
    // Progressive reveal fields
    pub reveal_sections: Vec<String>,
    pub revealed_count: usize,
    pub reveal_lesson_idx: Option<usize>,
    pub focused_section: usize,
    pub section_line_offsets: Vec<u16>,
    // LLM fields
    #[cfg(feature = "llm")]
    pub ai_enabled: bool,
    #[cfg(feature = "llm")]
    pub llm_channel: Option<LlmChannel>,
    #[cfg(feature = "llm")]
    pub llm_config: Option<LlmConfig>,
    #[cfg(feature = "llm")]
    pub chat_state: Option<ChatState>,
    #[cfg(feature = "llm")]
    pub chat_visible: bool,
    #[cfg(feature = "llm")]
    pub ai_status: String,
}

impl CourseApp {
    pub fn new(
        course: Course,
        progress_store: &ProgressStore,
        start_lesson: Option<&str>,
        start_lesson_idx: Option<usize>,
    ) -> Self {
        let lesson_idx = if let Some(lesson_id) = start_lesson {
            course
                .loaded_lessons
                .iter()
                .position(|l| l.id == lesson_id)
                .unwrap_or(0)
        } else if let Some(idx) = start_lesson_idx {
            idx.min(course.loaded_lessons.len().saturating_sub(1))
        } else {
            find_resume_lesson(&course, progress_store)
        };

        let exercise_idx = find_resume_exercise(&course, progress_store, lesson_idx);

        let mut starter_files = if !course.loaded_lessons.is_empty()
            && !course.loaded_lessons[lesson_idx]
                .loaded_exercises
                .is_empty()
        {
            let ex = &course.loaded_lessons[lesson_idx].loaded_exercises[exercise_idx];
            ex.get_starter_files(&course.language.extension)
        } else {
            vec![]
        };

        // Load draft files if they exist (persisted edits from a previous session)
        if !course.loaded_lessons.is_empty()
            && !course.loaded_lessons[lesson_idx]
                .loaded_exercises
                .is_empty()
        {
            let course_id = course.name.to_lowercase().replace(' ', "-");
            let lesson_id = &course.loaded_lessons[lesson_idx].id;
            let exercise_id = &course.loaded_lessons[lesson_idx].loaded_exercises[exercise_idx].id;
            if let Ok(dir) = crate::state::sandbox::draft_dir(
                &course_id,
                &course.version,
                lesson_id,
                exercise_id,
            ) {
                if let Ok(drafts) = crate::state::sandbox::load_draft_files(&dir) {
                    if !drafts.is_empty() {
                        for (name, content) in &drafts {
                            if let Some(f) = starter_files
                                .iter_mut()
                                .find(|f| f.editable && f.name == *name)
                            {
                                f.content = content.clone();
                            }
                        }
                    }
                }
            }
        }

        Self {
            course,
            current_lesson_idx: lesson_idx,
            current_exercise_idx: exercise_idx,
            state: AppState::LessonContent,
            session: SessionState::new(starter_files),
            sandbox_level: SandboxLevel::Basic,
            scroll_offset: 0,
            content_line_count: 0,
            viewport_height: 0,
            last_error: None,
            last_step_name: None,
            failure_detail: None,
            last_run_output: None,
            teardown_warnings: Vec::new(),
            animation_start: None,
            show_help: false,
            help_scroll_offset: 0,
            inline_editor: None,
            editing: false,
            last_input_time: Instant::now(),
            shown_quickstart: false,
            course_complete_stats: None,
            watch_state: None,
            sandbox_code: Vec::new(),
            sandbox_last_output: None,
            sandbox_lesson_idx: 0,
            sandbox_editing: false,
            sandbox_watching: false,
            shell_state: None,
            last_assertion_results: None,
            reveal_sections: Vec::new(),
            revealed_count: 0,
            reveal_lesson_idx: None,
            focused_section: 0,
            section_line_offsets: Vec::new(),
            #[cfg(feature = "llm")]
            ai_enabled: false,
            #[cfg(feature = "llm")]
            llm_channel: None,
            #[cfg(feature = "llm")]
            llm_config: None,
            #[cfg(feature = "llm")]
            chat_state: None,
            #[cfg(feature = "llm")]
            chat_visible: false,
            #[cfg(feature = "llm")]
            ai_status: "off".to_string(),
        }
    }

    #[cfg(feature = "llm")]
    pub fn enable_ai(&mut self, channel: LlmChannel, config: LlmConfig) {
        self.ai_enabled = true;
        self.llm_channel = Some(channel);
        self.llm_config = Some(config);
        self.chat_state = Some(ChatState::new());
        self.ai_status = "connecting...".to_string();
    }

    pub fn current_lesson(&self) -> Option<&crate::course::types::Lesson> {
        self.course.loaded_lessons.get(self.current_lesson_idx)
    }

    /// Check if the user's code differs from the exercise starter.
    pub fn is_code_modified(&self) -> bool {
        let ext = &self.course.language.extension;
        if let Some(exercise) = self.current_exercise() {
            let starters = exercise.get_starter_files(ext);
            for current in &self.session.current_code {
                if !current.editable {
                    continue;
                }
                if let Some(starter) = starters.iter().find(|s| s.name == current.name) {
                    if current.content != starter.content {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn current_exercise(&self) -> Option<&Exercise> {
        self.current_lesson()
            .and_then(|l| l.loaded_exercises.get(self.current_exercise_idx))
    }

    // --- Rendering ---

    pub fn render(&mut self, frame: &mut ratatui::Frame, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status bar
                Constraint::Min(1),    // content
                Constraint::Length(1), // keybinding bar
            ])
            .split(frame.size());

        self.viewport_height = chunks[1].height;
        self.render_content(frame, chunks[1], theme);
        self.render_status_bar(frame, chunks[0], theme);

        if self.show_help {
            self.render_help_overlay(frame, chunks[1], theme);
            let bar = Paragraph::new(Line::from(Span::styled(
                " Press [?] or [Esc] to close help",
                Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(ratatui::style::Color::White),
            )));
            frame.render_widget(bar, chunks[2]);
        } else {
            self.render_key_bar(frame, chunks[2], theme);
        }

        // Render scroll indicators over the content area
        self.render_scroll_indicators(frame, chunks[1]);
    }

    fn render_status_bar(&self, frame: &mut ratatui::Frame, area: Rect, _theme: &Theme) {
        let lesson_info = self
            .current_lesson()
            .map(|l| l.title.clone())
            .unwrap_or_default();

        let lesson_num = format!(
            "Lesson {}/{}",
            self.current_lesson_idx + 1,
            self.course.loaded_lessons.len()
        );

        let exercise_info = if self.state != AppState::LessonContent
            && self.state != AppState::LessonRecap
            && self.state != AppState::CourseComplete
            && self.state != AppState::Watching
            && self.state != AppState::Sandbox
        {
            let total = self
                .current_lesson()
                .map(|l| l.loaded_exercises.len())
                .unwrap_or(0);
            let bar = celebration::mini_progress_bar(self.current_exercise_idx + 1, total, 7);
            format!(" | Ex {}/{} {}", self.current_exercise_idx + 1, total, bar)
        } else {
            String::new()
        };

        #[cfg(feature = "llm")]
        let ai_info = if self.ai_enabled {
            format!(" | AI: {}", self.ai_status)
        } else {
            String::new()
        };
        #[cfg(not(feature = "llm"))]
        let ai_info = String::new();

        let scroll_info =
            if self.content_line_count > self.viewport_height && self.viewport_height > 0 {
                let current_page = (self.scroll_offset / self.viewport_height) + 1;
                let total_pages = self.content_line_count.div_ceil(self.viewport_height);
                format!(" | {}/{}", current_page, total_pages)
            } else {
                String::new()
            };

        let status = format!(
            " LearnLocal | {} | {} | {}{}{}{}",
            self.course.name, lesson_num, lesson_info, exercise_info, scroll_info, ai_info
        );

        let bar = Paragraph::new(Line::from(Span::styled(
            status,
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(ratatui::style::Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(bar, area);
    }

    /// Render a scrollable paragraph, tracking total line count.
    fn render_scrollable(
        &mut self,
        frame: &mut ratatui::Frame,
        area: Rect,
        lines: Vec<Line<'static>>,
    ) {
        self.content_line_count = lines.len() as u16;
        // Clamp scroll offset so we don't scroll past content
        let max_scroll = self.content_line_count.saturating_sub(area.height);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));
        frame.render_widget(paragraph, area);
    }

    fn render_scroll_indicators(&self, frame: &mut ratatui::Frame, area: Rect) {
        if self.content_line_count <= area.height {
            return; // No overflow, nothing to show
        }

        let indicator_style = Style::default()
            .fg(ratatui::style::Color::Yellow)
            .add_modifier(Modifier::BOLD);

        // Show ▲ at top-right when scrolled down
        if self.scroll_offset > 0 {
            let indicator = Paragraph::new(Line::from(Span::styled(" ▲ more ", indicator_style)));
            let r = Rect::new(
                area.x + area.width.saturating_sub(9),
                area.y,
                9.min(area.width),
                1,
            );
            frame.render_widget(indicator, r);
        }

        // Show ▼ at bottom-right when more content below
        let max_scroll = self.content_line_count.saturating_sub(area.height);
        if self.scroll_offset < max_scroll {
            let indicator = Paragraph::new(Line::from(Span::styled(" ▼ more ", indicator_style)));
            let r = Rect::new(
                area.x + area.width.saturating_sub(9),
                area.y + area.height.saturating_sub(1),
                9.min(area.width),
                1,
            );
            frame.render_widget(indicator, r);
        }
    }

    fn render_content(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        #[cfg(feature = "llm")]
        let (exercise_area, chat_area) = if self.chat_visible {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };
        #[cfg(not(feature = "llm"))]
        let exercise_area = area;

        match self.state {
            AppState::LessonContent => self.render_lesson_content(frame, exercise_area, theme),
            AppState::ExercisePrompt => {
                if !self.shown_quickstart
                    && self.current_lesson_idx == 0
                    && self.current_exercise_idx == 0
                {
                    // Split: banner on top, exercise below
                    let banner_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(10), Constraint::Min(1)])
                        .split(exercise_area);
                    self.render_quickstart_banner(frame, banner_chunks[0], theme);
                    self.render_exercise_prompt(frame, banner_chunks[1], theme);
                } else {
                    self.render_exercise_prompt(frame, exercise_area, theme);
                }
            }
            AppState::Editing => self.render_exercise_prompt(frame, exercise_area, theme),
            AppState::Executing => self.render_executing(frame, exercise_area, theme),
            AppState::RunResult => self.render_run_result(frame, exercise_area, theme),
            AppState::ResultSuccess => self.render_result_success(frame, exercise_area, theme),
            AppState::ResultFail => self.render_result_fail(frame, exercise_area, theme),
            AppState::LessonRecap => self.render_lesson_recap(frame, exercise_area, theme),
            AppState::CourseComplete => self.render_course_complete(frame, exercise_area, theme),
            AppState::Watching => self.render_watching(frame, exercise_area, theme),
            AppState::Sandbox => self.render_sandbox(frame, exercise_area, theme),
            AppState::Shell => self.render_shell(frame, exercise_area, theme),
        }

        #[cfg(feature = "llm")]
        if let Some(chat_area) = chat_area {
            self.render_chat_panel(frame, chat_area, theme);
        }
    }

    fn render_lesson_content(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let md = self
            .current_lesson()
            .map(|l| l.content_markdown.clone())
            .unwrap_or_else(|| "No content".to_string());

        // Lazy recompute: if lesson changed, re-split and reset reveal
        if self.reveal_lesson_idx != Some(self.current_lesson_idx) {
            self.reveal_sections = crate::course::loader::split_display_sections(&md);
            self.revealed_count = 1.min(self.reveal_sections.len());
            self.reveal_lesson_idx = Some(self.current_lesson_idx);
            self.focused_section = 0;
            self.section_line_offsets = Vec::new();
            self.scroll_offset = 0;
        }

        let dim_color = theme.muted;
        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut offsets: Vec<u16> = Vec::new();

        if self.reveal_sections.is_empty() {
            // No sections — render full markdown as-is
            lines = markdown::render_markdown(&md, theme);
        } else {
            for i in 0..self.revealed_count {
                offsets.push(lines.len() as u16);
                let mut section_lines = markdown::render_markdown(&self.reveal_sections[i], theme);
                if i != self.focused_section {
                    dim_lines(&mut section_lines, dim_color);
                }
                lines.extend(section_lines);
                // Blank separator between sections
                if i + 1 < self.revealed_count {
                    lines.push(Line::from(""));
                }
            }
        }

        // Animated "Space to continue" indicator if more sections remain
        if self.revealed_count < self.reveal_sections.len() {
            let epoch_ms = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as usize;
            let dot_frames = ["   ", ".  ", ".. ", "..."];
            let dots = dot_frames[(epoch_ms / 500) % dot_frames.len()];

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  \u{25bc} Space to continue reading{}", dots),
                Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
        }

        self.section_line_offsets = offsets;
        self.render_scrollable(frame, area, lines);
    }

    /// Render workspace panel lines showing environment setup (files, dirs, services, etc.)
    fn render_workspace_lines(env: &EnvironmentSpec, theme: &Theme) -> Vec<Line<'static>> {
        let has_content = !env.files.is_empty()
            || !env.dirs.is_empty()
            || !env.symlinks.is_empty()
            || !env.services.is_empty()
            || env.ports > 0
            || !env.setup.is_empty();

        if !has_content {
            return Vec::new();
        }

        let border_color = theme.code_border;
        let label_color = theme.keyword;
        let content_color = theme.body_text;
        let muted = theme.muted;

        let mut lines: Vec<Line<'static>> = Vec::new();

        // Top border
        let header = " WORKSPACE ";
        let fill = 46_usize.saturating_sub(header.len());
        lines.push(Line::from(Span::styled(
            format!(
                "  \u{250C}\u{2500}{}{}\u{2510}",
                header,
                "\u{2500}".repeat(fill)
            ),
            Style::default().fg(border_color),
        )));

        // Files
        for file in &env.files {
            let perm_info = file
                .permissions
                .as_ref()
                .map(|p| format!(" ({})", p))
                .unwrap_or_default();
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("\u{1F4C4} {}{}", file.path, perm_info),
                    Style::default().fg(label_color),
                ),
            ]));
            // Show first few lines of file content (max 4)
            let content_lines: Vec<&str> = file.content.lines().take(4).collect();
            for cl in &content_lines {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2502}    ", Style::default().fg(border_color)),
                    Span::styled(cl.to_string(), Style::default().fg(content_color)),
                ]));
            }
            let total_lines = file.content.lines().count();
            if total_lines > 4 {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2502}    ", Style::default().fg(border_color)),
                    Span::styled(
                        format!("... ({} more lines)", total_lines - 4),
                        Style::default().fg(muted),
                    ),
                ]));
            }
        }

        // Directories
        for dir in &env.dirs {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("\u{1F4C1} {}/", dir),
                    Style::default().fg(label_color),
                ),
            ]));
        }

        // Symlinks
        for sym in &env.symlinks {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("\u{1F517} {} \u{2192} {}", sym.link, sym.target),
                    Style::default().fg(label_color),
                ),
            ]));
        }

        // Services
        for svc in &env.services {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("\u{1F50C} {} ({})", svc.name, svc.command),
                    Style::default().fg(label_color),
                ),
            ]));
        }

        // Ports
        if env.ports > 0 {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!(
                        "\u{1F310} {} dynamic port{}",
                        env.ports,
                        if env.ports > 1 { "s" } else { "" }
                    ),
                    Style::default().fg(label_color),
                ),
            ]));
        }

        // Setup commands (summary)
        if !env.setup.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!(
                        "\u{2699}\u{FE0F}  {} setup step{}",
                        env.setup.len(),
                        if env.setup.len() > 1 { "s" } else { "" }
                    ),
                    Style::default().fg(muted),
                ),
            ]));
        }

        // Bottom border
        lines.push(Line::from(Span::styled(
            format!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(48)),
            Style::default().fg(border_color),
        )));
        lines.push(Line::from(""));

        lines
    }

    /// Render pre-run assertion checklist lines (unchecked ○ items).
    fn render_assertion_checklist_lines(
        assertions: &[StateAssertion],
        theme: &Theme,
    ) -> Vec<Line<'static>> {
        if assertions.is_empty() {
            return Vec::new();
        }

        let border_color = theme.code_border;
        let mut lines: Vec<Line<'static>> = Vec::new();

        let header = " EXPECTED ";
        let fill = 46_usize.saturating_sub(header.len());
        lines.push(Line::from(Span::styled(
            format!(
                "  \u{250C}\u{2500}{}{}\u{2510}",
                header,
                "\u{2500}".repeat(fill)
            ),
            Style::default().fg(border_color),
        )));

        for assertion in assertions {
            let desc = Self::assertion_description(assertion);
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("\u{25CB} {}", desc),
                    Style::default().fg(theme.body_text),
                ),
            ]));
        }

        lines.push(Line::from(Span::styled(
            format!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(48)),
            Style::default().fg(border_color),
        )));
        lines.push(Line::from(""));

        lines
    }

    /// Render post-run assertion results (✔/✘ items).
    fn render_assertion_results_lines(
        results: &[crate::exec::environment::AssertionResult],
        theme: &Theme,
    ) -> Vec<Line<'static>> {
        if results.is_empty() {
            return Vec::new();
        }

        let border_color = theme.code_border;
        let all_passed = results.iter().all(|r| r.passed);
        let header = if all_passed {
            " RESULTS \u{2714} "
        } else {
            " RESULTS "
        };
        let header_color = if all_passed {
            theme.success
        } else {
            border_color
        };
        let fill = 46_usize.saturating_sub(header.len());

        let mut lines: Vec<Line<'static>> = Vec::new();

        lines.push(Line::from(Span::styled(
            format!(
                "  \u{250C}\u{2500}{}{}\u{2510}",
                header,
                "\u{2500}".repeat(fill)
            ),
            Style::default().fg(header_color),
        )));

        for r in results {
            let (icon, color) = if r.passed {
                ("\u{2714}", theme.success)
            } else {
                ("\u{2718}", theme.error)
            };
            lines.push(Line::from(vec![
                Span::styled("  \u{2502}  ", Style::default().fg(border_color)),
                Span::styled(
                    format!("{} {}", icon, r.description),
                    Style::default().fg(color),
                ),
            ]));
            if !r.passed {
                lines.push(Line::from(vec![
                    Span::styled("  \u{2502}    ", Style::default().fg(border_color)),
                    Span::styled(
                        format!("\u{2192} {}", r.detail),
                        Style::default().fg(theme.muted),
                    ),
                ]));
            }
        }

        lines.push(Line::from(Span::styled(
            format!("  \u{2514}{}\u{2518}", "\u{2500}".repeat(48)),
            Style::default().fg(border_color),
        )));
        lines.push(Line::from(""));

        lines
    }

    /// Map a StateAssertion to a human-readable description.
    fn assertion_description(assertion: &StateAssertion) -> String {
        match assertion {
            StateAssertion::FileExists(p) => format!("{} exists", p),
            StateAssertion::DirExists(p) => format!("{}/ exists", p),
            StateAssertion::FileNotExists(p) => format!("{} does not exist", p),
            StateAssertion::DirNotExists(p) => format!("{}/ does not exist", p),
            StateAssertion::FileContains(c) => {
                let preview = if c.content.len() > 30 {
                    format!("{}...", &c.content[..27])
                } else {
                    c.content.clone()
                };
                format!("{} contains \"{}\"", c.path, preview)
            }
            StateAssertion::FileMatches(c) => format!("{} matches /{}/", c.path, c.pattern),
            StateAssertion::FileEquals(c) => {
                let preview = if c.content.len() > 30 {
                    format!("{}...", &c.content[..27])
                } else {
                    c.content.clone()
                };
                format!("{} equals \"{}\"", c.path, preview)
            }
            StateAssertion::Permissions(c) => format!("{} has permissions {}", c.path, c.mode),
            StateAssertion::Symlink(c) => format!("{} \u{2192} {}", c.path, c.target),
            StateAssertion::FileCount(c) => format!("{}/ has {} entries", c.path, c.count),
            StateAssertion::DirEmpty(p) => format!("{}/ is empty", p),
        }
    }

    fn render_exercise_prompt(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let exercise = match self.current_exercise() {
            Some(e) => e,
            None => return,
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        let mut scroll_to_cursor: Option<(u16, u16)> = None; // (cursor_abs_line, viewport_height)

        // Show relevant lesson section above the exercise prompt
        if let Some(lesson) = self.current_lesson() {
            if let Some(section) = lesson.content_sections.get(self.current_exercise_idx) {
                let section_lines = markdown::render_markdown(section, theme);
                lines.extend(section_lines);
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    Style::default().fg(theme.muted),
                )));
                lines.push(Line::from(""));
            }
        }

        let modified = self.is_code_modified();
        let mut title_spans = vec![Span::styled(
            format!("  Exercise: {}", exercise.title),
            Style::default()
                .fg(theme.prompt)
                .add_modifier(Modifier::BOLD),
        )];
        if modified {
            title_spans.push(Span::styled(
                "  (modified)",
                Style::default().fg(Color::Yellow),
            ));
        }
        lines.push(Line::from(title_spans));
        lines.push(Line::from(""));

        let (type_str, type_color) = match exercise.exercise_type {
            ExerciseType::Write => ("[WRITE]", theme.success),
            ExerciseType::Fix => ("[FIX]", Color::Yellow),
            ExerciseType::FillBlank => ("[FILL BLANK]", Color::Cyan),
            ExerciseType::MultipleChoice => ("[CHOICE]", Color::Magenta),
            ExerciseType::Predict => ("[PREDICT]", theme.prompt),
            ExerciseType::Command => ("[COMMAND]", Color::Blue),
        };
        if exercise.golf {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", type_str), Style::default().fg(type_color)),
                Span::styled(
                    "  [GOLF]",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                format!("  {}", type_str),
                Style::default().fg(type_color),
            )));
        }
        lines.push(Line::from(""));

        for line in exercise.prompt.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(theme.body_text),
            )));
        }
        lines.push(Line::from(""));

        // Workspace panel: show environment setup (files, dirs, services)
        if let Some(ref env) = exercise.environment {
            let workspace_lines = Self::render_workspace_lines(env, theme);
            lines.extend(workspace_lines);
        }

        // Assertion checklist: show pre-run expectations or post-run results
        if exercise.validation.method == ValidationMethod::State {
            if let Some(ref results) = self.last_assertion_results {
                // Post-run: show results with ✔/✘
                let result_lines = Self::render_assertion_results_lines(results, theme);
                lines.extend(result_lines);
            } else if let Some(ref assertions) = exercise.validation.assertions {
                // Pre-run: show unchecked ○ expectations
                if exercise.exercise_type != ExerciseType::Predict {
                    let checklist_lines = Self::render_assertion_checklist_lines(assertions, theme);
                    lines.extend(checklist_lines);
                }
            }
        }

        if !self.session.current_code.is_empty() && exercise.exercise_type != ExerciseType::Command
        {
            // Adaptive code box: use area width minus indent/margin
            let box_w = (area.width as usize).saturating_sub(4).max(20);
            let code_w = box_w.saturating_sub(8); // │ + space + 3 digits + 2 spaces + │

            // Determine if we're in inline edit mode for this file
            let is_editing = self.editing && self.inline_editor.is_some();
            let border_color = if is_editing || modified {
                Color::Yellow
            } else {
                theme.code_border
            };

            for (file_i, file) in self.session.current_code.iter().enumerate() {
                // File label for multi-file exercises
                if self.session.current_code.len() > 1 {
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  File: {} {}",
                            file.name,
                            if file.editable {
                                "(editable)"
                            } else {
                                "(read-only)"
                            }
                        ),
                        Style::default().fg(theme.muted),
                    )));
                }

                // Check if this file is the one being edited
                let editing_this_file = is_editing
                    && self
                        .inline_editor
                        .as_ref()
                        .is_some_and(|e| e.file_idx == file_i);

                if editing_this_file {
                    let editor = self.inline_editor.as_ref().unwrap();
                    // Top border with editing indicator
                    let label = format!(" {} [editing] ", file.name);
                    let border_fill = box_w.saturating_sub(2).saturating_sub(label.len());
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  \u{250C}\u{2500}{}{}\u{2510}",
                            label,
                            "\u{2500}".repeat(border_fill)
                        ),
                        Style::default().fg(border_color),
                    )));
                    // Track where the code starts for auto-scroll
                    let code_box_start = lines.len() as u16;
                    // Render editor lines with cursor
                    for (i, editor_line) in editor.lines.iter().enumerate() {
                        let is_cursor_line = i == editor.cursor_line;
                        let num_style = if is_cursor_line {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };

                        let prefix = format!("  \u{2502} {:3}  ", i + 1);

                        if is_cursor_line {
                            let (before, cursor_char, after) =
                                crate::ui::inline_editor::split_at_cursor(
                                    editor_line,
                                    editor.cursor_col,
                                );
                            // Pad the after portion so the right border aligns
                            let used = before.len() + cursor_char.len() + after.len();
                            let pad = code_w.saturating_sub(used);
                            let after_padded = format!("{}{}", after, " ".repeat(pad));
                            lines.push(Line::from(vec![
                                Span::styled(prefix, num_style),
                                Span::styled(before, Style::default().fg(theme.code)),
                                Span::styled(
                                    cursor_char,
                                    Style::default().fg(Color::Black).bg(Color::White),
                                ),
                                Span::styled(after_padded, Style::default().fg(theme.code)),
                                Span::styled("\u{2502}", Style::default().fg(border_color)),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled(prefix, num_style),
                                Span::styled(
                                    format!("{:<width$}", editor_line, width = code_w),
                                    Style::default().fg(theme.code),
                                ),
                                Span::styled("\u{2502}", Style::default().fg(border_color)),
                            ]));
                        }
                    }
                    // Status line inside bottom border
                    let status = format!(
                        " Ln {}, Col {} ",
                        editor.cursor_line + 1,
                        editor.cursor_col + 1,
                    );
                    let status_fill = box_w.saturating_sub(2).saturating_sub(status.len());
                    lines.push(Line::from(vec![
                        Span::styled("  \u{2514}", Style::default().fg(border_color)),
                        Span::styled(status, Style::default().fg(Color::Black).bg(Color::Cyan)),
                        Span::styled(
                            "\u{2500}".repeat(status_fill),
                            Style::default().fg(border_color),
                        ),
                        Span::styled("\u{2518}", Style::default().fg(border_color)),
                    ]));
                    lines.push(Line::from(""));

                    // Defer scroll adjustment (can't mutate self while exercise is borrowed)
                    let cursor_abs_line = code_box_start + editor.cursor_line as u16;
                    scroll_to_cursor = Some((cursor_abs_line, area.height.saturating_sub(2)));
                } else {
                    // Static code box (read-only display)
                    let label = format!(" {} ", file.name);
                    let border_fill = box_w.saturating_sub(2).saturating_sub(label.len());
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  \u{250C}\u{2500}{}{}\u{2510}",
                            label,
                            "\u{2500}".repeat(border_fill)
                        ),
                        Style::default().fg(border_color),
                    )));
                    for (i, code_line) in file.content.lines().enumerate() {
                        let prefix = format!("  \u{2502} {:3}  ", i + 1);
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(border_color)),
                            Span::styled(
                                format!("{:<width$}", code_line, width = code_w),
                                Style::default().fg(theme.code),
                            ),
                            Span::styled("\u{2502}", Style::default().fg(border_color)),
                        ]));
                    }
                    lines.push(Line::from(Span::styled(
                        format!(
                            "  \u{2514}{}\u{2518}",
                            "\u{2500}".repeat(box_w.saturating_sub(2))
                        ),
                        Style::default().fg(border_color),
                    )));
                    lines.push(Line::from(""));
                }
            }
        }

        if self.session.hints_revealed > 0 {
            lines.push(Line::from(Span::styled(
                "  Hints:",
                Style::default()
                    .fg(theme.keyword)
                    .add_modifier(Modifier::BOLD),
            )));
            for (i, hint) in exercise.hints.iter().enumerate() {
                if i < self.session.hints_revealed {
                    lines.push(Line::from(format!("  {}. {}", i + 1, hint)));
                }
            }
            lines.push(Line::from(""));
        }

        self.render_scrollable(frame, area, lines);

        // Apply deferred scroll adjustment for editing cursor visibility
        if let Some((cursor_abs_line, viewport)) = scroll_to_cursor {
            if cursor_abs_line >= self.scroll_offset + viewport {
                self.scroll_offset = cursor_abs_line.saturating_sub(viewport / 2);
            } else if cursor_abs_line < self.scroll_offset {
                self.scroll_offset = cursor_abs_line.saturating_sub(2);
            }
        }
    }

    fn render_quickstart_banner(&self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let key_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(theme.body_text);
        let dim_style = Style::default().fg(theme.muted);

        let lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  [e]", key_style),
                Span::styled(" Edit your code    ", text_style),
                Span::styled("[Enter]", key_style),
                Span::styled(" Run it", text_style),
            ]),
            Line::from(vec![
                Span::styled("  [t]", key_style),
                Span::styled(" Submit answer     ", text_style),
                Span::styled("[h]", key_style),
                Span::styled("     Get a hint", text_style),
            ]),
            Line::from(vec![
                Span::styled("  [?]", key_style),
                Span::styled(" All shortcuts    ", text_style),
                Span::styled("[Esc]", key_style),
                Span::styled("   Back to home", text_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Press any key to start", dim_style)),
        ];

        let banner = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Quick Start ")
                    .title_alignment(Alignment::Left)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(banner, area);
    }

    fn render_executing(&self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Running...",
                Style::default().fg(theme.prompt),
            )),
        ]);
        frame.render_widget(paragraph, area);
    }

    fn render_run_result(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(""));

        if let Some(ref output) = self.last_run_output {
            if output.timed_out {
                lines.push(Line::from(Span::styled(
                    "  Execution timed out",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if !output.success {
                if let Some(ref step) = output.step_failed {
                    lines.push(Line::from(Span::styled(
                        format!("  Failed at: {}", step),
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )));
                }
                lines.push(Line::from(""));
                let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
            } else {
                lines.push(Line::from(Span::styled(
                    "  Program output:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                if output.stdout.trim().is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  (no output)",
                        Style::default().fg(theme.muted),
                    )));
                } else {
                    for line in output.stdout.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(theme.body_text),
                        )));
                    }
                }
                if !output.stderr.trim().is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  Warnings:",
                        Style::default()
                            .fg(theme.keyword)
                            .add_modifier(Modifier::BOLD),
                    )));
                    let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                    lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
                }
            }
        }

        // Show teardown warnings if any
        if !self.teardown_warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ⚠ Teardown warnings:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for warning in &self.teardown_warnings {
                lines.push(Line::from(Span::styled(
                    format!("    {}", warning),
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(
            "  Press [Enter] to go back, [t] to test, [e] to edit",
        ));

        self.render_scrollable(frame, area, lines);
    }

    fn render_result_success(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let total_exercises = self
            .current_lesson()
            .map(|l| l.loaded_exercises.len())
            .unwrap_or(0);

        if let Some(start) = self.animation_start {
            if start.elapsed() < Duration::from_millis(400) {
                let lines = celebration::exercise_success_art(
                    self.current_exercise_idx,
                    total_exercises,
                    theme,
                );
                self.content_line_count = 0;
                let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                frame.render_widget(paragraph, area);
                return;
            }
        }

        let exercise = self.current_exercise();
        let mut lines =
            celebration::exercise_success_art(self.current_exercise_idx, total_exercises, theme);

        if let Some(ex) = exercise {
            if let Some(ref explanation) = ex.explanation {
                lines.push(Line::from(Span::styled(
                    "  Explanation:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                for line in explanation.lines() {
                    lines.push(Line::from(format!("  {}", line)));
                }
            }
        }

        // Show assertion results on success (all passed)
        if let Some(ref results) = self.last_assertion_results {
            lines.push(Line::from(""));
            let result_lines = Self::render_assertion_results_lines(results, theme);
            lines.extend(result_lines);
        }

        // Code golf scoring
        if let Some(ex) = self.current_exercise() {
            if ex.golf {
                let student_chars: usize = self
                    .session
                    .current_code
                    .iter()
                    .filter(|f| f.editable)
                    .map(|f| f.content.trim().len())
                    .sum();

                let par_chars: Option<usize> = ex.solution.as_ref().map(|s| s.trim().len());

                lines.push(Line::from(""));
                let mut score_spans = vec![
                    Span::styled("  Your solution: ", Style::default().fg(theme.muted)),
                    Span::styled(
                        format!("{} chars", student_chars),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ];

                if let Some(par) = par_chars {
                    score_spans.push(Span::styled(
                        format!("  |  Par: {} chars", par),
                        Style::default().fg(theme.muted),
                    ));

                    if student_chars <= par {
                        lines.push(Line::from(score_spans));
                        lines.push(Line::from(Span::styled(
                            "  At or under par!",
                            Style::default()
                                .fg(theme.success)
                                .add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        let over = student_chars - par;
                        score_spans.push(Span::styled(
                            format!("  (+{})", over),
                            Style::default().fg(Color::Red),
                        ));
                        lines.push(Line::from(score_spans));
                    }
                } else {
                    lines.push(Line::from(score_spans));
                }
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from("  Press Enter to continue"));

        self.render_scrollable(frame, area, lines);
    }

    fn render_result_fail(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        if let Some(start) = self.animation_start {
            if start.elapsed() < Duration::from_millis(200) {
                let lines = vec![
                    Line::from(""),
                    Line::from(""),
                    Line::from(""),
                    Line::from(Span::styled(
                        "\u{2718}",
                        Style::default()
                            .fg(ratatui::style::Color::White)
                            .bg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Not quite right",
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )),
                ];
                self.content_line_count = 0;
                let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
                frame.render_widget(paragraph, area);
                return;
            }
        }

        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  \u{2718} Not quite right",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        if let Some(ref step_name) = self.last_step_name {
            lines.push(Line::from(format!("  Failed at step: {}", step_name)));
        }

        match &self.failure_detail {
            Some(FailureDetail::OutputMismatch { expected, actual }) => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Output diff:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.extend(crate::ui::diff::render_output_diff(expected, actual, theme));
            }
            Some(FailureDetail::RegexMismatch { pattern, actual }) => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Pattern mismatch:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  Pattern: /{}/", pattern),
                    Style::default().fg(theme.diff_expected),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  Actual:  \"{}\"", actual),
                    Style::default().fg(theme.diff_actual),
                )));
            }
            Some(FailureDetail::StateAssertionFailed { results }) => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  State assertions:",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                for r in results {
                    let icon = if r.passed { "\u{2714}" } else { "\u{2718}" };
                    let color = if r.passed { theme.success } else { theme.error };
                    lines.push(Line::from(Span::styled(
                        format!("  {} {}", icon, r.description),
                        Style::default().fg(color),
                    )));
                    if !r.passed {
                        lines.push(Line::from(Span::styled(
                            format!("    {}", r.detail),
                            Style::default().fg(theme.muted),
                        )));
                    }
                }
            }
            Some(FailureDetail::InfrastructureFailed { phase, detail }) => {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Infrastructure error — this is a course setup problem, not your code",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  Phase: {}", phase),
                    Style::default().fg(theme.body_text),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  {}", detail),
                    Style::default().fg(theme.diff_actual),
                )));
            }
            Some(FailureDetail::Plain(msg)) => {
                lines.push(Line::from(""));
                let parsed = crate::ui::diagnostics::parse_compiler_output(msg);
                lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
            }
            None => {
                if let Some(ref error) = self.last_error {
                    lines.push(Line::from(""));
                    let parsed = crate::ui::diagnostics::parse_compiler_output(error);
                    lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
                }
            }
        }

        // Show teardown warnings if any
        if !self.teardown_warnings.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ⚠ Teardown warnings:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            for warning in &self.teardown_warnings {
                lines.push(Line::from(Span::styled(
                    format!("    {}", warning),
                    Style::default().fg(Color::Yellow),
                )));
            }
        }

        let hints_left = self
            .current_exercise()
            .map(|e| e.hints.len().saturating_sub(self.session.hints_revealed))
            .unwrap_or(0);
        if hints_left > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(format!(
                "  Press [h] for a hint ({} remaining)",
                hints_left
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(
            "  Press [e] to edit, [Enter] to run, [t] to test, [s] to skip",
        ));

        self.render_scrollable(frame, area, lines);
    }

    fn render_lesson_recap(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let lesson = match self.current_lesson() {
            Some(l) => l,
            None => return,
        };

        let total_lessons = self.course.loaded_lessons.len();
        let mut lines = celebration::lesson_complete_art(
            &lesson.title,
            self.current_lesson_idx,
            total_lessons,
            theme,
        );

        if let Some(ref recap) = lesson.recap {
            for line in recap.lines() {
                lines.push(Line::from(format!("  {}", line)));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from("  Press Enter to continue to the next lesson"));

        // Sandbox prompt
        let course_id = self.course_id();
        let has_sandbox =
            crate::state::sandbox::has_sandbox_files(&course_id, &self.course.version, &lesson.id);
        let sandbox_text = if has_sandbox {
            "  Press [s] to resume sandbox"
        } else {
            "  Press [s] to enter sandbox"
        };
        lines.push(Line::from(Span::styled(
            sandbox_text.to_string(),
            Style::default().fg(theme.muted),
        )));

        self.render_scrollable(frame, area, lines);
    }

    fn render_course_complete(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let stats = self.course_complete_stats.clone().unwrap_or(CourseStats {
            total_exercises: 0,
            completed_exercises: 0,
            skipped_exercises: 0,
            total_attempts: 0,
            first_try_count: 0,
            hint_free_count: 0,
            total_time_seconds: 0,
        });

        let lines = celebration::course_complete_art(&self.course.name, &stats, theme);

        self.render_scrollable(frame, area, lines);
    }

    fn render_watching(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let exercise = match self.current_exercise() {
            Some(e) => e,
            None => return,
        };

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(""));

        // Watch mode badge
        let epoch_ms = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as usize;
        let dot_frames = ["   ", ".  ", ".. ", "..."];
        let dots = dot_frames[(epoch_ms / 500) % dot_frames.len()];

        lines.push(Line::from(vec![
            Span::styled(
                "  WATCH MODE  ",
                Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  Watching for changes{}", dots),
                Style::default().fg(Color::Yellow),
            ),
        ]));
        lines.push(Line::from(""));

        // Exercise title
        lines.push(Line::from(Span::styled(
            format!("  Exercise: {}", exercise.title),
            Style::default()
                .fg(theme.prompt)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Auto-test status
        let auto_test = self
            .watch_state
            .as_ref()
            .map(|w| w.auto_test)
            .unwrap_or(false);
        lines.push(Line::from(Span::styled(
            format!(
                "  Auto-Test: {}",
                if auto_test {
                    "ON (will grade on save)"
                } else {
                    "OFF (run only)"
                }
            ),
            Style::default().fg(theme.muted),
        )));
        lines.push(Line::from(""));

        // Last run output
        if let Some(ref ws) = self.watch_state {
            if let Some(ref output) = ws.last_watch_output {
                if output.timed_out {
                    lines.push(Line::from(Span::styled(
                        "  Execution timed out",
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )));
                } else if !output.success {
                    lines.push(Line::from(Span::styled(
                        "  Build/Run Failed:",
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(""));
                    let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                    lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
                } else {
                    lines.push(Line::from(Span::styled(
                        "  Output:",
                        Style::default().add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(""));
                    if output.stdout.trim().is_empty() {
                        lines.push(Line::from(Span::styled(
                            "  (no output)",
                            Style::default().fg(theme.muted),
                        )));
                    } else {
                        for line in output.stdout.lines() {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", line),
                                Style::default().fg(theme.body_text),
                            )));
                        }
                    }
                    if !output.stderr.trim().is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "  Warnings:",
                            Style::default()
                                .fg(theme.keyword)
                                .add_modifier(Modifier::BOLD),
                        )));
                        let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                        lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
                    }
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "  Waiting for first save...",
                    Style::default().fg(theme.muted),
                )));
            }
        }

        self.render_scrollable(frame, area, lines);
    }

    fn render_key_bar(&self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let keys = match self.state {
            AppState::LessonContent => {
                let mut s = String::new();
                if self.revealed_count < self.reveal_sections.len() {
                    s.push_str("[Space] More  ");
                }
                s.push_str("[Enter] Exercises  [\u{2190}/\u{2192}] Lessons  [Esc] Home  [?] Help");
                s
            }
            AppState::ExercisePrompt => {
                let is_cmd = self
                    .current_exercise()
                    .is_some_and(|e| e.exercise_type == ExerciseType::Command);
                if is_cmd {
                    "[Enter] Shell  [h] Hint  [s] Skip  [Esc] Home".to_string()
                } else if self.editing {
                    "[e] Stop editing  [Ctrl+S] Save  [Enter] Run  [t] Test".to_string()
                } else {
                    "[e] Edit  [Enter] Run  [t] Test  [Esc] Home  [?] Help".to_string()
                }
            }
            AppState::Editing => "[Waiting for editor...]".to_string(),
            AppState::Executing => "[Running...]".to_string(),
            AppState::RunResult => {
                "[Enter] Back  [e] Edit  [t] Test  [Esc] Home  [?] Help".to_string()
            }
            AppState::ResultSuccess => "[Enter] Continue  [Esc] Home  [q] Quit".to_string(),
            AppState::ResultFail => {
                "[Enter] Back  [e] Edit  [t] Retest  [h] Hint  [Esc] Home  [?] Help".to_string()
            }
            AppState::LessonRecap => {
                "[Enter] Next lesson  [s] Sandbox  [Esc] Home  [q] Quit".to_string()
            }
            AppState::CourseComplete => "[Esc] Home  [q] Quit".to_string(),
            AppState::Watching => {
                let auto = self
                    .watch_state
                    .as_ref()
                    .map(|w| w.auto_test)
                    .unwrap_or(false);
                let auto_str = if auto { "ON" } else { "OFF" };
                format!("[Esc] Exit Watch  [t] Auto-Test: {}  [?] Help", auto_str)
            }
            AppState::Shell => {
                #[allow(unused_mut)]
                let mut s = "[Enter] Run  [Ctrl+H] Hint  [\u{2191}/\u{2193}] History  [Esc] Exit  [F1] Help".to_string();
                #[cfg(feature = "llm")]
                if self.ai_enabled && self.ai_status == "ready" {
                    s = "[Enter] Run  [Ctrl+A] AI  [Ctrl+H] Hint  [\u{2191}/\u{2193}] History  [Esc] Exit  [F1] Help".to_string();
                }
                s
            }
            AppState::Sandbox => {
                #[allow(unused_mut)]
                let mut s = "[e] Edit  [Enter] Run  [w] Watch  [Esc] Back".to_string();
                #[cfg(feature = "llm")]
                if self.ai_enabled && self.ai_status == "ready" {
                    s = "[e] Edit  [Enter] Run  [w] Watch  [a] AI  [Esc] Back".to_string();
                }
                s
            }
        };

        // Check for idle tip
        let idle_secs = self.last_input_time.elapsed().as_secs();
        let tip = if idle_secs >= 4 {
            let tips = get_tips_for_state(&self.state);
            if !tips.is_empty() {
                // Cycle every 5 seconds after the initial 4s delay
                let tip_cycle = ((idle_secs - 4) / 5) as usize % tips.len();
                Some(tips[tip_cycle])
            } else {
                None
            }
        } else {
            None
        };

        let mut spans = vec![Span::styled(
            format!(" {}", keys),
            Style::default()
                .fg(ratatui::style::Color::Black)
                .bg(ratatui::style::Color::White),
        )];

        if let Some(tip_text) = tip {
            spans.push(Span::styled(
                format!("  \u{2502} {}", tip_text),
                Style::default()
                    .fg(theme.muted)
                    .bg(ratatui::style::Color::White),
            ));
        }

        // Fill remaining width with white background
        let used: usize = spans.iter().map(|s| s.content.len()).sum();
        let remaining = (area.width as usize).saturating_sub(used);
        if remaining > 0 {
            spans.push(Span::styled(
                " ".repeat(remaining),
                Style::default().bg(ratatui::style::Color::White),
            ));
        }

        let bar = Paragraph::new(Line::from(spans));
        frame.render_widget(bar, area);
    }

    fn render_help_overlay(&self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        use ratatui::widgets::Clear;

        let key_style = Style::default().fg(Color::Cyan);
        let desc_style = Style::default().fg(theme.muted);

        #[allow(unused_mut)]
        let mut help_lines: Vec<Line<'_>> = vec![
            Line::from(Span::styled(
                " Keyboard Shortcuts",
                Style::default()
                    .fg(theme.heading)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Editing",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [e]          ", key_style),
                Span::raw("Inline editor"),
            ]),
            Line::from(Span::styled(
                "               Edit right in the TUI",
                desc_style,
            )),
            Line::from(vec![
                Span::styled("  [E]          ", key_style),
                Span::raw("External editor"),
            ]),
            Line::from(Span::styled(
                "               Opens $EDITOR (vim, VS Code)",
                desc_style,
            )),
            Line::from(vec![
                Span::styled("  [w]          ", key_style),
                Span::raw("Watch mode"),
            ]),
            Line::from(Span::styled(
                "               Auto-runs code on save",
                desc_style,
            )),
            Line::from(vec![
                Span::styled("  [W]          ", key_style),
                Span::raw("Watch + auto-test"),
            ]),
            Line::from(Span::styled(
                "               Auto-validates on save",
                desc_style,
            )),
            Line::from(vec![
                Span::styled("  [r]          ", key_style),
                Span::raw("Reset code"),
            ]),
            Line::from(Span::styled(
                "               Restore starter code",
                desc_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Running & Testing",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [Enter]      ", key_style),
                Span::raw("Run (ungraded)"),
            ]),
            Line::from(Span::styled(
                "               See output without grading",
                desc_style,
            )),
            Line::from(vec![
                Span::styled("  [t]          ", key_style),
                Span::raw("Test (graded)"),
            ]),
            Line::from(Span::styled(
                "               Validate your solution",
                desc_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Getting Help",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [h]          ", key_style),
                Span::raw("Reveal next hint"),
            ]),
            Line::from(Span::styled(
                "               Progressive, one at a time",
                desc_style,
            )),
        ];

        #[cfg(feature = "llm")]
        if self.ai_enabled {
            help_lines.push(Line::from(vec![
                Span::styled("  [a]          ", key_style),
                Span::raw("AI Chat"),
            ]));
            help_lines.push(Line::from(Span::styled(
                "               Ask questions anytime",
                desc_style,
            )));
            help_lines.push(Line::from(vec![
                Span::styled("  [w] / [x]    ", key_style),
                Span::raw("Why wrong / Explain error"),
            ]));
            help_lines.push(Line::from(Span::styled(
                "               After a failed test",
                desc_style,
            )));
        }

        help_lines.extend_from_slice(&[
            Line::from(""),
            Line::from(Span::styled(
                " Reading Lessons",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [Space]      ", key_style),
                Span::raw("Reveal next section"),
            ]),
            Line::from(vec![
                Span::styled("  [\u{2191}/\u{2193}]        ", key_style),
                Span::raw("Focus between sections"),
            ]),
            Line::from(vec![
                Span::styled("  [\u{2190}/\u{2192}]        ", key_style),
                Span::raw("Navigate between lessons"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                " Sandbox",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [s]          ", key_style),
                Span::raw("Enter sandbox (from recap)"),
            ]),
            Line::from(Span::styled(
                "               Free experimentation mode",
                desc_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                " Navigation",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  [PgUp/PgDn]  ", key_style),
                Span::raw("Scroll content"),
            ]),
            Line::from(vec![
                Span::styled("  [Home/End]   ", key_style),
                Span::raw("Jump to top/bottom"),
            ]),
            Line::from(vec![
                Span::styled("  [s]          ", key_style),
                Span::raw("Skip exercise"),
            ]),
            Line::from(vec![
                Span::styled("  [Esc]        ", key_style),
                Span::raw("Return to Home"),
            ]),
            Line::from(vec![
                Span::styled("  [q]          ", key_style),
                Span::raw("Quit"),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  [\u{2191}/\u{2193}] Scroll  [Esc] Close",
                desc_style,
            )),
        ]);

        let total_lines = help_lines.len() as u16;
        let help_width = 60u16;
        // Use available area but cap to content + border
        let help_height = (total_lines + 2).min(area.height);

        let y = area.y + area.height.saturating_sub(help_height) / 2;
        let x = area.x + area.width.saturating_sub(help_width) / 2;
        let overlay_area = Rect::new(x, y, help_width.min(area.width), help_height);

        // Apply scroll
        let inner_height = help_height.saturating_sub(2); // borders
        let max_scroll = total_lines.saturating_sub(inner_height);
        let scroll = self.help_scroll_offset.min(max_scroll);

        frame.render_widget(Clear, overlay_area);
        let help_panel = Paragraph::new(help_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.heading)),
            )
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));
        frame.render_widget(help_panel, overlay_area);
    }

    // --- Input handling ---

    pub fn handle_input(
        &mut self,
        key_event: KeyEvent,
        progress_store: &mut ProgressStore,
        config: &Config,
        sandbox_level: SandboxLevel,
    ) -> Result<CourseAction> {
        let key = key_event.code;

        // Reset tip rotation on any keypress
        self.last_input_time = Instant::now();

        // Quickstart banner: dismiss on any keypress, then fall through to normal handling
        if !self.shown_quickstart
            && self.state == AppState::ExercisePrompt
            && self.current_lesson_idx == 0
            && self.current_exercise_idx == 0
        {
            self.shown_quickstart = true;
            // Don't return — let the keypress fall through to the normal handler
        }

        // Help overlay toggle
        if self.show_help {
            match key {
                KeyCode::Esc | KeyCode::Char('?') => {
                    self.show_help = false;
                    self.help_scroll_offset = 0;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.help_scroll_offset = self.help_scroll_offset.saturating_add(10);
                }
                _ => {}
            }
            return Ok(CourseAction::Continue);
        }
        if key == KeyCode::Char('?') && self.state != AppState::Shell {
            self.show_help = true;
            self.help_scroll_offset = 0;
            return Ok(CourseAction::Continue);
        }

        // Global scroll keys (all scrollable states, but not when editing)
        let scrollable = !self.editing
            && matches!(
                self.state,
                AppState::LessonContent
                    | AppState::ExercisePrompt
                    | AppState::RunResult
                    | AppState::ResultSuccess
                    | AppState::ResultFail
                    | AppState::LessonRecap
                    | AppState::Watching
                    | AppState::Sandbox
            );
        if scrollable {
            let page = self.viewport_height.max(1);
            match key {
                KeyCode::PageDown => {
                    self.scroll_offset = self.scroll_offset.saturating_add(page);
                    return Ok(CourseAction::Continue);
                }
                KeyCode::PageUp => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(page);
                    return Ok(CourseAction::Continue);
                }
                KeyCode::Home => {
                    self.scroll_offset = 0;
                    return Ok(CourseAction::Continue);
                }
                KeyCode::End => {
                    self.scroll_offset = self.content_line_count;
                    return Ok(CourseAction::Continue);
                }
                _ => {}
            }
        }

        // Chat input routing
        #[cfg(feature = "llm")]
        if self.chat_visible {
            self.handle_chat_input(key, key_event.modifiers);
            return Ok(CourseAction::Continue);
        }

        match self.state {
            AppState::LessonContent => Ok(self.handle_lesson_content_input(key)),
            AppState::ExercisePrompt => self.handle_exercise_input(
                key,
                key_event.modifiers,
                progress_store,
                config,
                sandbox_level,
            ),
            AppState::RunResult => self.handle_run_result_input(
                key,
                key_event.modifiers,
                progress_store,
                config,
                sandbox_level,
            ),
            AppState::ResultSuccess => Ok(self.handle_success_input(key, progress_store)),
            AppState::ResultFail => self.handle_fail_input(
                key,
                key_event.modifiers,
                progress_store,
                config,
                sandbox_level,
            ),
            AppState::LessonRecap => Ok(self.handle_recap_input(key, progress_store)),
            AppState::Watching => {
                Ok(self.handle_watching_input(key, progress_store, sandbox_level))
            }
            AppState::Shell => self.handle_shell_input(key, key_event.modifiers, progress_store),
            AppState::Sandbox => {
                self.handle_sandbox_input(key, key_event.modifiers, config, sandbox_level)
            }
            AppState::CourseComplete => {
                if key == KeyCode::Char('q') {
                    Ok(CourseAction::Quit)
                } else if key == KeyCode::Esc {
                    Ok(CourseAction::GoHome)
                } else {
                    Ok(CourseAction::Continue)
                }
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn handle_lesson_content_input(&mut self, key: KeyCode) -> CourseAction {
        match key {
            KeyCode::Char('q') => CourseAction::Quit,
            KeyCode::Esc => CourseAction::GoHome,
            KeyCode::Char(' ') => {
                if self.revealed_count < self.reveal_sections.len() {
                    self.revealed_count += 1;
                    self.focused_section = self.revealed_count - 1;
                    // Auto-scroll to show the new section (near bottom of previous content)
                    self.scroll_offset = self.content_line_count.saturating_sub(4);
                }
                CourseAction::Continue
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a') if self.ai_enabled && self.ai_status == "ready" => {
                self.open_chat();
                CourseAction::Continue
            }
            KeyCode::Enter => {
                self.enter_exercise_state();
                CourseAction::Continue
            }
            KeyCode::Right => {
                self.next_lesson();
                CourseAction::Continue
            }
            KeyCode::Left => {
                self.prev_lesson();
                CourseAction::Continue
            }
            KeyCode::Up => {
                if self.focused_section > 0 {
                    self.focused_section -= 1;
                    if let Some(&offset) = self.section_line_offsets.get(self.focused_section) {
                        self.scroll_offset = offset;
                    }
                }
                CourseAction::Continue
            }
            KeyCode::Down => {
                if self.focused_section + 1 < self.revealed_count {
                    self.focused_section += 1;
                    if let Some(&offset) = self.section_line_offsets.get(self.focused_section) {
                        self.scroll_offset = offset;
                    }
                }
                CourseAction::Continue
            }
            _ => CourseAction::Continue,
        }
    }

    fn handle_exercise_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        progress_store: &mut ProgressStore,
        config: &Config,
        sandbox_level: SandboxLevel,
    ) -> Result<CourseAction> {
        // When in edit mode, route most keys to the inline editor
        if self.editing {
            return match key {
                KeyCode::Char('e') => {
                    // Toggle off edit mode
                    self.apply_inline_editor_to_session();
                    self.editing = false;
                    self.inline_editor = None;
                    Ok(CourseAction::Continue)
                }
                KeyCode::Esc => {
                    // Exit edit mode (save changes)
                    self.apply_inline_editor_to_session();
                    self.editing = false;
                    self.inline_editor = None;
                    Ok(CourseAction::Continue)
                }
                KeyCode::Enter if modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+Enter: save and run
                    self.apply_inline_editor_to_session();
                    self.editing = false;
                    self.inline_editor = None;
                    self.run_exercise(sandbox_level)?;
                    Ok(CourseAction::Continue)
                }
                KeyCode::Char('t') if modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+T: save and test
                    self.apply_inline_editor_to_session();
                    self.editing = false;
                    self.inline_editor = None;
                    self.submit_exercise(progress_store, sandbox_level)?;
                    Ok(CourseAction::Continue)
                }
                KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+S: save without exiting edit mode
                    self.apply_inline_editor_to_session();
                    Ok(CourseAction::Continue)
                }
                _ => {
                    // All other keys go to the editor
                    if let Some(ref mut editor) = self.inline_editor {
                        editor.handle_key(key, modifiers);
                    }
                    Ok(CourseAction::Continue)
                }
            };
        }

        // Command exercises that landed on ExercisePrompt (via Esc from shell):
        // re-enter shell mode on Enter/e/t instead of running code through the old path
        let is_command_exercise = self
            .current_exercise()
            .is_some_and(|e| e.exercise_type == ExerciseType::Command);
        if is_command_exercise {
            return match key {
                KeyCode::Char('q') => Ok(CourseAction::Quit),
                KeyCode::Esc => Ok(CourseAction::GoHome),
                KeyCode::Enter | KeyCode::Char('e') => {
                    self.enter_exercise_state();
                    Ok(CourseAction::Continue)
                }
                KeyCode::Char('h') => {
                    self.reveal_hint();
                    Ok(CourseAction::Continue)
                }
                KeyCode::Char('s') => {
                    self.skip_exercise(progress_store);
                    Ok(CourseAction::Continue)
                }
                _ => Ok(CourseAction::Continue),
            };
        }

        match key {
            KeyCode::Char('q') => Ok(CourseAction::Quit),
            KeyCode::Esc => Ok(CourseAction::GoHome),
            KeyCode::Char('E') if modifiers.contains(KeyModifiers::SHIFT) => {
                // If GUI editor, auto-enter watch mode instead of blocking
                let editor_cmd = crate::ui::editor::detect_editor(config.editor.as_deref());
                let editor_type = crate::ui::editor_detect::resolve_editor_type(
                    editor_cmd.as_deref(),
                    &config.editor_type,
                );
                if editor_type == crate::config::EditorType::Gui {
                    self.enter_watch_mode(config, false)?;
                } else {
                    self.launch_editor(config)?;
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('e') => {
                self.enter_inline_editor();
                Ok(CourseAction::Continue)
            }
            KeyCode::Enter => {
                self.run_exercise(sandbox_level)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('t') => {
                self.submit_exercise(progress_store, sandbox_level)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('h') => {
                self.reveal_hint();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('s') => {
                self.skip_exercise(progress_store);
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('r') => {
                self.reset_to_starter();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('w') => {
                self.enter_watch_mode(config, false)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('W') if modifiers.contains(KeyModifiers::SHIFT) => {
                self.enter_watch_mode(config, true)?;
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a') if self.ai_enabled && self.ai_status == "ready" => {
                self.open_chat();
                Ok(CourseAction::Continue)
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Ok(CourseAction::Continue)
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                Ok(CourseAction::Continue)
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn handle_success_input(
        &mut self,
        key: KeyCode,
        progress_store: &mut ProgressStore,
    ) -> CourseAction {
        self.animation_start = None;
        match key {
            KeyCode::Char('q') => CourseAction::Quit,
            KeyCode::Esc => CourseAction::GoHome,
            KeyCode::Enter => {
                self.advance_exercise(progress_store);
                CourseAction::Continue
            }
            _ => CourseAction::Continue,
        }
    }

    fn handle_fail_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        progress_store: &mut ProgressStore,
        config: &Config,
        sandbox_level: SandboxLevel,
    ) -> Result<CourseAction> {
        self.animation_start = None;
        match key {
            KeyCode::Char('q') => Ok(CourseAction::Quit),
            KeyCode::Esc => Ok(CourseAction::GoHome),
            KeyCode::Char('E') if modifiers.contains(KeyModifiers::SHIFT) => {
                self.launch_editor(config)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('e') => {
                self.enter_inline_editor();
                Ok(CourseAction::Continue)
            }
            KeyCode::Enter => {
                self.failure_detail = None;
                self.enter_exercise_state();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('t') => {
                self.submit_exercise(progress_store, sandbox_level)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('h') => {
                self.reveal_hint();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('r') => {
                self.reset_to_starter();
                self.failure_detail = None;
                self.enter_exercise_state();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('s') => {
                self.skip_exercise(progress_store);
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a') if self.ai_enabled && self.ai_status == "ready" => {
                self.open_chat();
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('w') if self.ai_enabled && self.ai_status == "ready" => {
                self.send_quick_action("Why is my solution wrong? What am I misunderstanding?");
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('x') if self.ai_enabled && self.ai_status == "ready" => {
                self.send_quick_action(
                    "Explain this error message. What does it mean and how do I fix it?",
                );
                Ok(CourseAction::Continue)
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn handle_recap_input(
        &mut self,
        key: KeyCode,
        progress_store: &mut ProgressStore,
    ) -> CourseAction {
        match key {
            KeyCode::Char('q') => CourseAction::Quit,
            KeyCode::Esc => CourseAction::GoHome,
            KeyCode::Enter => {
                if self.current_lesson_idx + 1 < self.course.loaded_lessons.len() {
                    self.current_lesson_idx += 1;
                    self.current_exercise_idx = 0;
                    self.scroll_offset = 0;
                    self.reset_session_for_current_exercise();
                    self.state = AppState::LessonContent;
                } else {
                    // Compute stats before entering CourseComplete
                    let course_id = self.course_id();
                    let total_exercises: usize = self
                        .course
                        .loaded_lessons
                        .iter()
                        .map(|l| l.loaded_exercises.len())
                        .sum();
                    self.course_complete_stats = Some(CourseStats::compute(
                        progress_store,
                        &course_id,
                        &self.course.version,
                        self.course.loaded_lessons.len(),
                        total_exercises,
                    ));
                    self.state = AppState::CourseComplete;
                    self.scroll_offset = 0;
                }
                CourseAction::Continue
            }
            KeyCode::Char('s') => {
                self.enter_sandbox(self.current_lesson_idx);
                CourseAction::Continue
            }
            _ => CourseAction::Continue,
        }
    }

    // --- Exercise operations ---

    fn launch_editor(&mut self, config: &Config) -> Result<()> {
        if self.current_exercise().is_none() {
            return Ok(());
        }

        terminal::leave_alternate_screen()?;

        let sandbox_dir = tempfile::tempdir()?;

        let editable_info: Vec<(String, String)> = self
            .session
            .current_code
            .iter()
            .filter(|f| f.editable)
            .map(|f| (f.name.clone(), f.content.clone()))
            .collect();

        for (name, content) in &editable_info {
            let path = sandbox_dir.path().join(name);
            std::fs::write(&path, content)?;
        }

        for (name, _) in &editable_info {
            let path = sandbox_dir.path().join(name);
            let new_content =
                crate::ui::editor::edit_file_with_config(&path, config.editor.as_deref())?;

            if let Some(f) = self
                .session
                .current_code
                .iter_mut()
                .find(|f| f.name == *name)
            {
                f.content = new_content;
            }
        }

        terminal::enter_alternate_screen()?;
        self.enter_exercise_state();

        Ok(())
    }

    fn enter_inline_editor(&mut self) {
        let is_command = self
            .current_exercise()
            .is_some_and(|e| e.exercise_type == ExerciseType::Command);
        if self.current_exercise().is_none() {
            return;
        }
        // Find the first editable file
        let file_idx = self
            .session
            .current_code
            .iter()
            .position(|f| f.editable)
            .unwrap_or(0);
        let content = &self.session.current_code[file_idx].content;
        self.inline_editor = Some(if is_command {
            InlineEditorState::new_command_mode(content, file_idx)
        } else {
            InlineEditorState::new(content, file_idx)
        });
        self.editing = true;
    }

    fn apply_inline_editor_to_session(&mut self) {
        if let Some(ref editor) = self.inline_editor {
            if self.sandbox_editing {
                if let Some(file) = self.sandbox_code.get_mut(editor.file_idx) {
                    file.content = editor.content();
                }
            } else if let Some(file) = self.session.current_code.get_mut(editor.file_idx) {
                file.content = editor.content();
            }
        }
    }

    // --- Shell mode (command exercises) ---

    fn enter_shell_mode(&mut self, sandbox_level: SandboxLevel) -> Result<()> {
        let exercise = match self.current_exercise() {
            Some(e) => e,
            None => return Ok(()),
        };

        let sandbox = Sandbox::new(&self.course.language.limits, sandbox_level)?;

        let mut shell = ShellState::new(sandbox);

        // Set up environment if present
        if let Some(ref env_spec) = exercise.environment {
            let main_file = exercise.get_main_file(&self.course.language.extension);
            let file_names: Vec<String> = exercise
                .get_starter_files(&self.course.language.extension)
                .iter()
                .map(|f| f.name.clone())
                .collect();

            let setup = environment::setup_environment(
                shell.sandbox.dir(),
                env_spec,
                &main_file,
                &file_names,
            )?;
            shell.env_vars = Some(setup.env_vars);
            shell.cwd_override = setup.cwd_override;

            // Run setup commands
            for step in &env_spec.setup {
                let output = environment::run_env_command(
                    &shell.sandbox,
                    step,
                    shell.env_vars.as_ref(),
                    shell.cwd_override.as_deref(),
                    self.course.language.limits.timeout_seconds,
                )?;
                if output.exit_code != 0 {
                    // Setup failed — still enter shell mode but show error in history
                    shell.history.push(ShellHistoryEntry {
                        command: format!("[setup: {}]", step.name),
                        stdout: output.stdout,
                        stderr: output.stderr,
                        exit_code: output.exit_code,
                        timed_out: output.timed_out,
                    });
                }
            }

            // Start background services
            shell.needs_loopback = !env_spec.services.is_empty();
            for svc in &env_spec.services {
                let svc_args: Vec<String> = svc
                    .args
                    .iter()
                    .map(|a| a.replace("{dir}", &shell.sandbox.dir().to_string_lossy()))
                    .collect();
                let mut child = shell.sandbox.spawn_service(
                    &svc.command,
                    &svc_args,
                    shell.env_vars.as_ref(),
                    shell.cwd_override.as_deref(),
                )?;
                match environment::wait_for_service_ready(&mut child, svc, shell.sandbox.dir()) {
                    Ok(reader_handles) => {
                        shell.drain_handles.extend(reader_handles);
                        let handles = runner::drain_service_pipes(
                            &mut child,
                            shell.sandbox.dir(),
                            svc.capture_stdout.as_deref(),
                            svc.capture_stderr.as_deref(),
                        );
                        shell.drain_handles.extend(handles);
                        shell.service_children.push((svc.name.clone(), child));
                    }
                    Err(e) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        shell.history.push(ShellHistoryEntry {
                            command: format!("[service: {}]", svc.name),
                            stdout: String::new(),
                            stderr: format!("{}", e),
                            exit_code: 1,
                            timed_out: false,
                        });
                    }
                }
            }
        }

        self.shell_state = Some(shell);
        self.state = AppState::Shell;
        self.scroll_offset = 0;
        Ok(())
    }

    fn exit_shell_mode(&mut self) {
        // Extract exercise data before borrowing shell_state mutably
        let teardown_info = self.current_exercise().and_then(|exercise| {
            exercise.environment.as_ref().map(|env_spec| {
                let main_file = exercise.get_main_file(&self.course.language.extension);
                let file_names: Vec<String> = exercise
                    .get_starter_files(&self.course.language.extension)
                    .iter()
                    .map(|f| f.name.clone())
                    .collect();
                let teardown = env_spec.teardown.clone();
                let timeout = self.course.language.limits.timeout_seconds;
                (main_file, file_names, teardown, timeout)
            })
        });

        if let Some(ref mut shell) = self.shell_state {
            // Run teardown commands (best-effort)
            if let Some((main_file, file_names, teardown, timeout)) = teardown_info {
                for step in &teardown {
                    let _ = environment::run_env_command_full(
                        &shell.sandbox,
                        step,
                        shell.env_vars.as_ref(),
                        shell.cwd_override.as_deref(),
                        timeout,
                        &main_file,
                        &file_names,
                    );
                }
            }

            // Kill service children
            for (_, child) in &mut shell.service_children {
                let _ = child.kill();
                let _ = child.wait();
            }

            // Join drain handles
            let handles = std::mem::take(&mut shell.drain_handles);
            for handle in handles {
                let _ = handle.join();
            }
        }

        self.shell_state = None;
    }

    fn shell_execute_and_validate(&mut self, progress_store: &mut ProgressStore) -> Result<()> {
        let cmd = match self.shell_state {
            Some(ref mut shell) => shell.take_input(),
            None => return Ok(()),
        };

        if cmd.trim().is_empty() {
            return Ok(());
        }

        // Execute the command
        let output = {
            let shell = self.shell_state.as_ref().unwrap();
            shell.sandbox.run_command_with_loopback(
                "sh",
                &["-c".to_string(), cmd.clone()],
                None,
                shell.env_vars.as_ref(),
                shell.cwd_override.as_deref(),
                shell.needs_loopback,
            )?
        };

        // Push to history
        let entry = ShellHistoryEntry {
            command: cmd,
            stdout: runner::clean_sandbox_paths(
                &output.stdout,
                self.shell_state.as_ref().unwrap().sandbox.dir(),
            ),
            stderr: runner::clean_sandbox_paths(
                &output.stderr,
                self.shell_state.as_ref().unwrap().sandbox.dir(),
            ),
            exit_code: output.exit_code,
            timed_out: output.timed_out,
        };
        self.shell_state.as_mut().unwrap().history.push(entry);

        // Run teardown commands that have capture_to (for state validation)
        if let Some(exercise) = self.current_exercise().cloned() {
            if let Some(ref env_spec) = exercise.environment {
                let main_file = exercise.get_main_file(&self.course.language.extension);
                let file_names: Vec<String> = exercise
                    .get_starter_files(&self.course.language.extension)
                    .iter()
                    .map(|f| f.name.clone())
                    .collect();

                let shell = self.shell_state.as_ref().unwrap();
                for step in &env_spec.teardown {
                    if step.capture_to.is_some() {
                        if let Ok(td_output) = environment::run_env_command_full(
                            &shell.sandbox,
                            step,
                            shell.env_vars.as_ref(),
                            shell.cwd_override.as_deref(),
                            self.course.language.limits.timeout_seconds,
                            &main_file,
                            &file_names,
                        ) {
                            if let Some(ref capture_path) = step.capture_to {
                                let _ = shell.sandbox.write_file(capture_path, &td_output.stdout);
                            }
                        }
                    }
                }
            }
        }

        // Auto-validate
        let mut success = false;

        if let Some(exercise) = self.current_exercise().cloned() {
            let shell = self.shell_state.as_ref().unwrap();

            if exercise.validation.method == ValidationMethod::State {
                if let Some(ref assertions) = exercise.validation.assertions {
                    let results = environment::validate_state(shell.sandbox.dir(), assertions);
                    let all_passed = results.iter().all(|r| r.passed);
                    self.last_assertion_results = Some(results);
                    if all_passed {
                        success = true;
                    }
                }
            } else if exercise.validation.method == ValidationMethod::Output
                || exercise.validation.method == ValidationMethod::Regex
            {
                // Validate against the last command's output
                let step_output = crate::exec::sandbox::StepOutput {
                    stdout: self
                        .shell_state
                        .as_ref()
                        .unwrap()
                        .history
                        .last()
                        .map(|e| e.stdout.clone())
                        .unwrap_or_default(),
                    stderr: self
                        .shell_state
                        .as_ref()
                        .unwrap()
                        .history
                        .last()
                        .map(|e| e.stderr.clone())
                        .unwrap_or_default(),
                    exit_code: self
                        .shell_state
                        .as_ref()
                        .unwrap()
                        .history
                        .last()
                        .map(|e| e.exit_code)
                        .unwrap_or(-1),
                    timed_out: false,
                };
                let vr = validate::validate_output(&exercise.validation, &step_output);
                if vr.is_success() {
                    success = true;
                }
            }
        }

        // Auto-scroll to bottom
        if let Some(ref mut shell) = self.shell_state {
            shell.scroll_to_bottom();
        }

        if success {
            self.mark_exercise_completed(progress_store);
            // Record attempt
            let time_spent = self.session.time_spent_seconds();
            let attempt = AttemptRecord {
                timestamp: chrono::Utc::now().to_rfc3339(),
                time_spent_seconds: time_spent,
                compile_success: true,
                run_exit_code: Some(0),
                output_matched: Some(true),
                hints_revealed: self.session.hints_revealed,
            };
            self.record_attempt(&attempt, progress_store);
            self.exit_shell_mode();
            self.state = AppState::ResultSuccess;
            self.scroll_offset = 0;
        }

        Ok(())
    }

    fn handle_shell_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        progress_store: &mut ProgressStore,
    ) -> Result<CourseAction> {
        // Help overlay in shell mode uses F1
        if let Some(ref mut shell) = self.shell_state {
            if shell.show_help {
                match key {
                    KeyCode::F(1) | KeyCode::Esc => {
                        shell.show_help = false;
                    }
                    _ => {}
                }
                return Ok(CourseAction::Continue);
            }
        }

        match key {
            KeyCode::Enter => {
                self.shell_execute_and_validate(progress_store)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Esc => {
                self.exit_shell_mode();
                self.state = AppState::ExercisePrompt;
                self.scroll_offset = 0;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('h') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.reveal_hint();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.clear_input();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::F(1) => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.show_help = true;
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Up => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.history_prev();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Down => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.history_next();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Left => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.move_left();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Right => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.move_right();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Backspace => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.backspace();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Delete => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.delete_char();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Home => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.cursor_col = 0;
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::End => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.cursor_col = shell.input.len();
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::PageUp => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.scroll_offset = shell
                        .scroll_offset
                        .saturating_sub(self.viewport_height.max(1));
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::PageDown => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.scroll_offset = shell
                        .scroll_offset
                        .saturating_add(self.viewport_height.max(1));
                }
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a')
                if modifiers.contains(KeyModifiers::CONTROL)
                    && self.ai_enabled
                    && self.ai_status == "ready" =>
            {
                self.open_chat();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char(c) => {
                if let Some(ref mut shell) = self.shell_state {
                    shell.insert_char(c);
                }
                Ok(CourseAction::Continue)
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn render_shell(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let exercise = match self.current_exercise() {
            Some(e) => e.clone(),
            None => return,
        };

        let mut lines: Vec<Line<'static>> = Vec::new();

        // Exercise title and type badge
        lines.push(Line::from(Span::styled(
            format!("  Exercise: {}", exercise.title),
            Style::default()
                .fg(theme.prompt)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  [COMMAND]",
            Style::default().fg(Color::Blue),
        )));
        lines.push(Line::from(""));

        // Exercise prompt
        for line in exercise.prompt.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line),
                Style::default().fg(theme.body_text),
            )));
        }
        lines.push(Line::from(""));

        // Workspace panel
        if let Some(ref env) = exercise.environment {
            let workspace_lines = Self::render_workspace_lines(env, theme);
            lines.extend(workspace_lines);
        }

        // Goals panel (assertion checklist or results)
        if exercise.validation.method == ValidationMethod::State {
            if let Some(ref results) = self.last_assertion_results {
                let result_lines = Self::render_assertion_results_lines(results, theme);
                lines.extend(result_lines);
            } else if let Some(ref assertions) = exercise.validation.assertions {
                let checklist_lines = Self::render_assertion_checklist_lines(assertions, theme);
                lines.extend(checklist_lines);
            }
        }

        // Hints
        if self.session.hints_revealed > 0 {
            lines.push(Line::from(Span::styled(
                "  Hints:",
                Style::default()
                    .fg(theme.keyword)
                    .add_modifier(Modifier::BOLD),
            )));
            for (i, hint) in exercise.hints.iter().enumerate() {
                if i < self.session.hints_revealed {
                    lines.push(Line::from(format!("  {}. {}", i + 1, hint)));
                }
            }
            lines.push(Line::from(""));
        }

        // Terminal header
        lines.push(Line::from(Span::styled(
            "  \u{2500}\u{2500}\u{2500} Terminal \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(theme.code_border),
        )));
        lines.push(Line::from(""));

        // History entries
        if let Some(ref shell) = self.shell_state {
            for entry in &shell.history {
                // Command line
                if entry.command.starts_with('[') {
                    // Setup/service info lines
                    lines.push(Line::from(Span::styled(
                        format!("  {}", entry.command),
                        Style::default().fg(theme.muted),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("  $ {}", entry.command),
                        Style::default().fg(Color::Green),
                    )));
                }
                // Stdout
                for out_line in entry.stdout.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", out_line),
                        Style::default().fg(theme.body_text),
                    )));
                }
                // Stderr
                for err_line in entry.stderr.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", err_line),
                        Style::default().fg(Color::Yellow),
                    )));
                }
                // Annotations
                if entry.timed_out {
                    lines.push(Line::from(Span::styled(
                        "  (timed out)",
                        Style::default().fg(theme.error),
                    )));
                } else if entry.exit_code != 0 && !entry.command.starts_with('[') {
                    lines.push(Line::from(Span::styled(
                        format!("  (exit {})", entry.exit_code),
                        Style::default().fg(theme.muted),
                    )));
                }
            }
        }

        // Current prompt line with cursor
        if let Some(ref shell) = self.shell_state {
            let (before, cursor_char, after) =
                crate::ui::inline_editor::split_at_cursor(&shell.input, shell.cursor_col);
            lines.push(Line::from(vec![
                Span::styled("  $ ", Style::default().fg(Color::Green)),
                Span::styled(before, Style::default().fg(theme.code)),
                Span::styled(
                    cursor_char,
                    Style::default().fg(Color::Black).bg(Color::White),
                ),
                Span::styled(after, Style::default().fg(theme.code)),
            ]));
        }

        lines.push(Line::from(""));

        // Help overlay
        if self.shell_state.as_ref().is_some_and(|s| s.show_help) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Shell Mode Help",
                Style::default()
                    .fg(theme.heading)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            let help_items = [
                ("Enter", "Execute command"),
                ("Up/Down", "Navigate command history"),
                ("Ctrl+C", "Clear current input"),
                ("Ctrl+H", "Reveal next hint"),
                ("Esc", "Exit shell mode"),
                ("PageUp/Down", "Scroll output"),
                ("F1", "Toggle this help"),
            ];
            for (key, desc) in &help_items {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:14}", key), Style::default().fg(Color::Cyan)),
                    Span::styled(*desc, Style::default().fg(theme.body_text)),
                ]));
            }
            lines.push(Line::from(""));
        }

        let total = lines.len() as u16;

        // Update shell's content_line_count
        if let Some(ref mut shell) = self.shell_state {
            shell.content_line_count = total;
            // Clamp scroll
            let max_scroll = total.saturating_sub(area.height);
            if shell.scroll_offset > max_scroll {
                shell.scroll_offset = max_scroll;
            }
        }

        let clamped_scroll = self.shell_state.as_ref().map_or(0, |s| s.scroll_offset);

        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::NONE))
            .wrap(Wrap { trim: false })
            .scroll((clamped_scroll, 0));
        frame.render_widget(paragraph, area);

        // Scroll indicators
        if total > area.height {
            let indicator_style = Style::default()
                .fg(ratatui::style::Color::Yellow)
                .add_modifier(Modifier::BOLD);

            if clamped_scroll > 0 {
                let indicator =
                    Paragraph::new(Line::from(Span::styled(" \u{25B2} more ", indicator_style)));
                let r = Rect::new(
                    area.x + area.width.saturating_sub(9),
                    area.y,
                    9.min(area.width),
                    1,
                );
                frame.render_widget(indicator, r);
            }

            let max_scroll = total.saturating_sub(area.height);
            if clamped_scroll < max_scroll {
                let indicator =
                    Paragraph::new(Line::from(Span::styled(" \u{25BC} more ", indicator_style)));
                let r = Rect::new(
                    area.x + area.width.saturating_sub(9),
                    area.y + area.height.saturating_sub(1),
                    9.min(area.width),
                    1,
                );
                frame.render_widget(indicator, r);
            }
        }
    }

    fn run_exercise(&mut self, sandbox_level: SandboxLevel) -> Result<()> {
        self.state = AppState::Executing;

        let mut output = runner::run_exercise_with_sandbox(
            &self.course,
            self.current_exercise().unwrap(),
            &self.session.current_code,
            sandbox_level,
        )?;

        if !output.success {
            self.last_error = Some(output.stderr.clone());
        }

        self.teardown_warnings = std::mem::take(&mut output.teardown_warnings);
        self.last_run_output = Some(output);
        self.state = AppState::RunResult;
        self.scroll_offset = 0;

        Ok(())
    }

    fn handle_run_result_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        progress_store: &mut ProgressStore,
        config: &Config,
        sandbox_level: SandboxLevel,
    ) -> Result<CourseAction> {
        match key {
            KeyCode::Char('q') => Ok(CourseAction::Quit),
            KeyCode::Esc => Ok(CourseAction::GoHome),
            KeyCode::Char('E') if modifiers.contains(KeyModifiers::SHIFT) => {
                self.launch_editor(config)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('e') => {
                self.enter_inline_editor();
                Ok(CourseAction::Continue)
            }
            KeyCode::Enter => {
                self.enter_exercise_state();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('t') => {
                self.submit_exercise(progress_store, sandbox_level)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('h') => {
                self.reveal_hint();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('r') => {
                self.reset_to_starter();
                self.enter_exercise_state();
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('s') => {
                self.skip_exercise(progress_store);
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a') if self.ai_enabled && self.ai_status == "ready" => {
                self.open_chat();
                Ok(CourseAction::Continue)
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Ok(CourseAction::Continue)
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                Ok(CourseAction::Continue)
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn submit_exercise(
        &mut self,
        progress_store: &mut ProgressStore,
        sandbox_level: SandboxLevel,
    ) -> Result<()> {
        self.state = AppState::Executing;

        let (result, teardown_warnings) = runner::execute_exercise_with_sandbox(
            &self.course,
            self.current_exercise().unwrap(),
            &self.session.current_code,
            sandbox_level,
        )?;

        let time_spent = self.session.time_spent_seconds();
        let (compile_success, run_exit_code, output_matched) = match &result {
            ExecutionResult::Success => (true, Some(0), Some(true)),
            ExecutionResult::CompileSuccess => (true, None, None),
            ExecutionResult::StepFailed { exit_code, .. } => (false, Some(*exit_code), None),
            ExecutionResult::ValidationFailed(_) => (true, Some(0), Some(false)),
            ExecutionResult::Timeout { .. } => (false, None, None),
            ExecutionResult::SetupFailed { exit_code, .. } => (false, Some(*exit_code), None),
            ExecutionResult::ServiceFailed { .. } => (false, None, None),
            ExecutionResult::Error(_) => (false, None, None),
        };

        let attempt = AttemptRecord {
            timestamp: chrono::Utc::now().to_rfc3339(),
            time_spent_seconds: time_spent,
            compile_success,
            run_exit_code,
            output_matched,
            hints_revealed: self.session.hints_revealed,
        };

        self.record_attempt(&attempt, progress_store);

        // Clear assertion results before processing new result
        self.last_assertion_results = None;

        match &result {
            ExecutionResult::Success | ExecutionResult::CompileSuccess => {
                self.mark_exercise_completed(progress_store);
                self.state = AppState::ResultSuccess;
            }
            ExecutionResult::StepFailed {
                step_name,
                stderr,
                exit_code: _,
            } => {
                self.last_step_name = Some(step_name.clone());
                self.last_error = Some(stderr.clone());
                self.failure_detail = Some(FailureDetail::Plain(stderr.clone()));
                self.state = AppState::ResultFail;
            }
            ExecutionResult::ValidationFailed(vr) => {
                self.last_step_name = None;
                match vr {
                    crate::exec::validate::ValidationResult::OutputMismatch {
                        expected,
                        actual,
                    } => {
                        self.last_error = Some(format!(
                            "Expected output: \"{}\"\nActual output:   \"{}\"",
                            expected, actual
                        ));
                        self.failure_detail = Some(FailureDetail::OutputMismatch {
                            expected: expected.clone(),
                            actual: actual.clone(),
                        });
                    }
                    crate::exec::validate::ValidationResult::RegexMismatch { pattern, actual } => {
                        self.last_error = Some(format!(
                            "Output \"{}\" didn't match pattern /{}/",
                            actual, pattern
                        ));
                        self.failure_detail = Some(FailureDetail::RegexMismatch {
                            pattern: pattern.clone(),
                            actual: actual.clone(),
                        });
                    }
                    crate::exec::validate::ValidationResult::StateAssertionFailed { results } => {
                        let failed_count = results.iter().filter(|r| !r.passed).count();
                        self.last_error = Some(format!("{} assertion(s) failed", failed_count));
                        self.last_assertion_results = Some(results.clone());
                        self.failure_detail = Some(FailureDetail::StateAssertionFailed {
                            results: results.clone(),
                        });
                    }
                    _ => {
                        self.last_error = Some("Validation failed".to_string());
                        self.failure_detail =
                            Some(FailureDetail::Plain("Validation failed".to_string()));
                    }
                }
                self.state = AppState::ResultFail;
            }
            ExecutionResult::Timeout { step_name } => {
                self.last_step_name = Some(step_name.clone());
                self.last_error = Some("Execution timed out".to_string());
                self.failure_detail = Some(FailureDetail::Plain("Execution timed out".to_string()));
                self.state = AppState::ResultFail;
            }
            ExecutionResult::SetupFailed {
                step_name,
                stderr,
                exit_code: _,
            } => {
                self.last_step_name = Some(step_name.clone());
                self.last_error = Some(stderr.clone());
                self.failure_detail = Some(FailureDetail::InfrastructureFailed {
                    phase: format!("setup: {}", step_name),
                    detail: stderr.clone(),
                });
                self.state = AppState::ResultFail;
            }
            ExecutionResult::ServiceFailed {
                service_name,
                error,
            } => {
                self.last_step_name = Some(service_name.clone());
                self.last_error = Some(error.clone());
                self.failure_detail = Some(FailureDetail::InfrastructureFailed {
                    phase: format!("service: {}", service_name),
                    detail: error.clone(),
                });
                self.state = AppState::ResultFail;
            }
            ExecutionResult::Error(msg) => {
                self.last_error = Some(msg.clone());
                self.failure_detail = Some(FailureDetail::Plain(msg.clone()));
                self.state = AppState::ResultFail;
            }
        }

        self.session.last_execution = Some(result);
        self.teardown_warnings = teardown_warnings;
        self.scroll_offset = 0;
        self.animation_start = Some(Instant::now());

        Ok(())
    }

    fn reveal_hint(&mut self) {
        let max_hints = self.current_exercise().map(|e| e.hints.len()).unwrap_or(0);
        if self.session.hints_revealed < max_hints {
            self.session.hints_revealed += 1;
        }
    }

    fn reset_to_starter(&mut self) {
        let ext = self.course.language.extension.clone();
        if let Some(exercise) = self.current_exercise() {
            let starter_files = exercise.get_starter_files(&ext);
            for starter in &starter_files {
                if let Some(f) = self
                    .session
                    .current_code
                    .iter_mut()
                    .find(|f| f.name == starter.name)
                {
                    f.content = starter.content.clone();
                }
            }
        }
        self.clear_current_draft();
    }

    fn skip_exercise(&mut self, progress_store: &mut ProgressStore) {
        self.mark_exercise_skipped(progress_store);
        self.advance_exercise(progress_store);
    }

    fn advance_exercise(&mut self, progress_store: &mut ProgressStore) {
        let total_exercises = self
            .current_lesson()
            .map(|l| l.loaded_exercises.len())
            .unwrap_or(0);

        if self.current_exercise_idx + 1 < total_exercises {
            self.current_exercise_idx += 1;
            self.reset_session_for_current_exercise();
            self.enter_exercise_state();
        } else {
            self.mark_lesson_completed(progress_store);
            self.state = AppState::LessonRecap;
        }
        self.scroll_offset = 0;
    }

    fn next_lesson(&mut self) {
        if self.current_lesson_idx + 1 < self.course.loaded_lessons.len() {
            self.current_lesson_idx += 1;
            self.current_exercise_idx = 0;
            self.scroll_offset = 0;
            self.reset_session_for_current_exercise();
        }
    }

    fn prev_lesson(&mut self) {
        if self.current_lesson_idx > 0 {
            self.current_lesson_idx -= 1;
            self.current_exercise_idx = 0;
            self.scroll_offset = 0;
            self.reset_session_for_current_exercise();
        }
    }

    fn reset_session_for_current_exercise(&mut self) {
        let ext = self.course.language.extension.clone();
        if let Some(exercise) = self.current_exercise() {
            let files = exercise.get_starter_files(&ext);
            self.session.reset_for_exercise(files);
        }

        // Try to load draft files (persisted edits from a previous session)
        let course_id = self.course_id();
        let lesson_id = self
            .current_lesson()
            .map(|l| l.id.clone())
            .unwrap_or_default();
        let exercise_id = self
            .current_exercise()
            .map(|e| e.id.clone())
            .unwrap_or_default();
        if !lesson_id.is_empty() && !exercise_id.is_empty() {
            if let Ok(dir) = crate::state::sandbox::draft_dir(
                &course_id,
                &self.course.version,
                &lesson_id,
                &exercise_id,
            ) {
                if let Ok(drafts) = crate::state::sandbox::load_draft_files(&dir) {
                    if !drafts.is_empty() {
                        for (name, content) in &drafts {
                            if let Some(f) = self
                                .session
                                .current_code
                                .iter_mut()
                                .find(|f| f.editable && f.name == *name)
                            {
                                f.content = content.clone();
                            }
                        }
                    }
                }
            }
        }

        self.editing = false;
        self.inline_editor = None;
        self.last_assertion_results = None;
        #[cfg(feature = "llm")]
        {
            if let Some(ref mut chat) = self.chat_state {
                chat.reset();
            }
            self.chat_visible = false;
        }
    }

    /// Transition to the correct exercise state: Shell for command exercises,
    /// ExercisePrompt for everything else. This is the ONLY correct way to
    /// enter the exercise view — never set `AppState::ExercisePrompt` directly.
    fn enter_exercise_state(&mut self) {
        if self
            .current_exercise()
            .is_some_and(|e| e.exercise_type == ExerciseType::Command)
        {
            // Clean up any previous shell state before re-entering
            if self.shell_state.is_some() {
                self.exit_shell_mode();
            }
            let _ = self.enter_shell_mode(self.sandbox_level);
        } else {
            self.state = AppState::ExercisePrompt;
        }
        self.scroll_offset = 0;
    }

    // --- Sandbox mode ---

    pub fn enter_sandbox(&mut self, lesson_idx: usize) {
        self.sandbox_lesson_idx = lesson_idx;

        let course_id = self.course_id();
        let lesson = match self.course.loaded_lessons.get(lesson_idx) {
            Some(l) => l,
            None => return,
        };

        // Try to load persisted sandbox files
        if let Ok(files) = crate::state::sandbox::load_sandbox_files(
            &crate::state::sandbox::sandbox_dir(&course_id, &self.course.version, &lesson.id)
                .unwrap_or_default(),
        ) {
            if !files.is_empty() {
                self.sandbox_code = files
                    .into_iter()
                    .map(|(name, content)| ExerciseFile {
                        name,
                        editable: true,
                        content,
                    })
                    .collect();
            } else {
                self.create_blank_sandbox();
            }
        } else {
            self.create_blank_sandbox();
        }

        self.sandbox_last_output = None;
        self.scroll_offset = 0;
        self.state = AppState::Sandbox;

        #[cfg(feature = "llm")]
        {
            if let Some(ref mut chat) = self.chat_state {
                chat.reset();
            }
            self.chat_visible = false;
        }
    }

    fn create_blank_sandbox(&mut self) {
        let ext = &self.course.language.extension;
        let filename = format!("playground{}", ext);
        self.sandbox_code = vec![ExerciseFile {
            name: filename,
            editable: true,
            content: String::new(),
        }];
    }

    pub fn save_draft_to_disk(&self) {
        let course_id = self.course_id();
        let lesson_id = self
            .current_lesson()
            .map(|l| l.id.clone())
            .unwrap_or_default();
        let exercise_id = self
            .current_exercise()
            .map(|e| e.id.clone())
            .unwrap_or_default();
        if lesson_id.is_empty() || exercise_id.is_empty() {
            return;
        }

        let dir = match crate::state::sandbox::draft_dir(
            &course_id,
            &self.course.version,
            &lesson_id,
            &exercise_id,
        ) {
            Ok(d) => d,
            Err(_) => return,
        };

        let files: Vec<(String, String)> = self
            .session
            .current_code
            .iter()
            .filter(|f| f.editable)
            .map(|f| (f.name.clone(), f.content.clone()))
            .collect();

        let _ = crate::state::sandbox::save_draft_files(&dir, &files);
    }

    fn clear_current_draft(&self) {
        let course_id = self.course_id();
        let lesson_id = self
            .current_lesson()
            .map(|l| l.id.clone())
            .unwrap_or_default();
        let exercise_id = self
            .current_exercise()
            .map(|e| e.id.clone())
            .unwrap_or_default();
        if lesson_id.is_empty() || exercise_id.is_empty() {
            return;
        }

        if let Ok(dir) = crate::state::sandbox::draft_dir(
            &course_id,
            &self.course.version,
            &lesson_id,
            &exercise_id,
        ) {
            let _ = crate::state::sandbox::clear_draft_files(&dir);
        }
    }

    pub fn save_sandbox_to_disk(&self) {
        let course_id = self.course_id();
        let lesson = match self.course.loaded_lessons.get(self.sandbox_lesson_idx) {
            Some(l) => l,
            None => return,
        };

        let dir = match crate::state::sandbox::sandbox_dir(
            &course_id,
            &self.course.version,
            &lesson.id,
        ) {
            Ok(d) => d,
            Err(_) => return,
        };

        let files: Vec<(String, String)> = self
            .sandbox_code
            .iter()
            .map(|f| (f.name.clone(), f.content.clone()))
            .collect();

        let _ = crate::state::sandbox::save_sandbox_files(&dir, &files);
    }

    fn sandbox_has_code(&self) -> bool {
        self.sandbox_code
            .iter()
            .any(|f| !f.content.trim().is_empty())
    }

    fn render_sandbox(&mut self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let lesson_title = self
            .course
            .loaded_lessons
            .get(self.sandbox_lesson_idx)
            .map(|l| l.title.clone())
            .unwrap_or_default();

        if !self.sandbox_has_code() && self.sandbox_last_output.is_none() {
            // Welcome banner
            lines.push(Line::from(""));
            let banner_lines = vec![
                format!("┌─ Sandbox: {} ─┐", lesson_title),
                "│".to_string(),
                "│  Experiment freely with what you learned.".to_string(),
                "│  No grading, no rules — just play.".to_string(),
                "│".to_string(),
            ];

            for bl in &banner_lines {
                lines.push(Line::from(Span::styled(
                    format!("  {}", bl),
                    Style::default().fg(Color::Yellow),
                )));
            }

            #[cfg(feature = "llm")]
            if self.ai_enabled && self.ai_status == "ready" {
                lines.push(Line::from(Span::styled(
                    "  │  Press [a] to ask the AI for ideas —".to_string(),
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(Span::styled(
                    "  │  it knows the concepts from this lesson.".to_string(),
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(Span::styled(
                    "  │".to_string(),
                    Style::default().fg(Color::Yellow),
                )));
            }

            // Compute closing border width to roughly match the title line
            let title_line = format!("┌─ Sandbox: {} ─┐", lesson_title);
            let border_width = title_line.len();
            let bottom = format!("└{}┘", "─".repeat(border_width.saturating_sub(2)));
            lines.push(Line::from(Span::styled(
                format!("  {}", bottom),
                Style::default().fg(Color::Yellow),
            )));
            lines.push(Line::from(""));
        }

        // Code box
        if !self.sandbox_code.is_empty() {
            let file = &self.sandbox_code[0];
            let filename = &file.name;
            let content = &file.content;

            let code_modified = self.sandbox_has_code();
            let border_color = if code_modified {
                Color::Yellow
            } else {
                theme.code_border
            };

            let code_width = area.width.saturating_sub(4) as usize;
            let title_text = format!("─ {} ", filename);
            let pad = code_width.saturating_sub(title_text.len() + 2);
            let top_border = format!("  ┌{}{}┐", title_text, "─".repeat(pad));

            lines.push(Line::from(Span::styled(
                top_border,
                Style::default().fg(border_color),
            )));

            if content.is_empty() {
                // Show empty placeholder
                lines.push(Line::from(vec![
                    Span::styled("  │ ", Style::default().fg(border_color)),
                    Span::styled("1 │ ", Style::default().fg(border_color)),
                    Span::styled(
                        format!("{: <width$}", "", width = code_width.saturating_sub(6)),
                        Style::default().fg(theme.muted),
                    ),
                    Span::styled("│", Style::default().fg(border_color)),
                ]));
            } else {
                for (i, line) in content.lines().enumerate() {
                    let line_num = format!("{:>3} │ ", i + 1);
                    let content_width = code_width.saturating_sub(line_num.len() + 1);
                    let padded = if line.len() < content_width {
                        format!("{}{}", line, " ".repeat(content_width - line.len()))
                    } else {
                        line[..content_width].to_string()
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  │", Style::default().fg(border_color)),
                        Span::styled(line_num, Style::default().fg(border_color)),
                        Span::styled(padded, Style::default().fg(theme.code)),
                        Span::styled("│", Style::default().fg(border_color)),
                    ]));
                }
            }

            let bottom_border = format!("  └{}┘", "─".repeat(code_width));
            lines.push(Line::from(Span::styled(
                bottom_border,
                Style::default().fg(border_color),
            )));
        }

        // Output section
        lines.push(Line::from(""));
        if let Some(ref output) = self.sandbox_last_output {
            if output.timed_out {
                lines.push(Line::from(Span::styled(
                    "  Execution timed out",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                )));
            } else if !output.success {
                lines.push(Line::from(Span::styled(
                    "  ── Error ──",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
            } else {
                lines.push(Line::from(Span::styled(
                    "  ── Output ──",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                if output.stdout.trim().is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  (no output)",
                        Style::default().fg(theme.muted),
                    )));
                } else {
                    for line in output.stdout.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(theme.body_text),
                        )));
                    }
                }
                if !output.stderr.trim().is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled(
                        "  Warnings:",
                        Style::default()
                            .fg(theme.keyword)
                            .add_modifier(Modifier::BOLD),
                    )));
                    let parsed = crate::ui::diagnostics::parse_compiler_output(&output.stderr);
                    lines.extend(crate::ui::diagnostics::render_diagnostics(&parsed, theme));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "  (no output yet — press [Enter] to run)",
                Style::default().fg(theme.muted),
            )));
        }

        self.render_scrollable(frame, area, lines);
    }

    fn handle_sandbox_input(
        &mut self,
        key: KeyCode,
        modifiers: KeyModifiers,
        config: &Config,
        sandbox_level: SandboxLevel,
    ) -> Result<CourseAction> {
        // When in edit mode, route keys to editor
        if self.editing {
            return match key {
                KeyCode::Char('e') | KeyCode::Esc => {
                    self.apply_inline_editor_to_session();
                    self.editing = false;
                    self.inline_editor = None;
                    self.sandbox_editing = false;
                    Ok(CourseAction::Continue)
                }
                KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.apply_inline_editor_to_session();
                    Ok(CourseAction::Continue)
                }
                _ => {
                    if let Some(ref mut editor) = self.inline_editor {
                        editor.handle_key(key, modifiers);
                    }
                    Ok(CourseAction::Continue)
                }
            };
        }

        match key {
            KeyCode::Esc => {
                self.save_sandbox_to_disk();
                self.state = AppState::LessonRecap;
                self.scroll_offset = 0;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('q') => {
                self.save_sandbox_to_disk();
                Ok(CourseAction::Quit)
            }
            KeyCode::Char('e') => {
                // Inline editor for sandbox
                if !self.sandbox_code.is_empty() {
                    let content = &self.sandbox_code[0].content;
                    self.inline_editor = Some(InlineEditorState::new(content, 0));
                    self.sandbox_editing = true;
                    self.editing = true;
                }
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('E') if modifiers.contains(KeyModifiers::SHIFT) => {
                self.launch_sandbox_editor(config)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Enter => {
                self.run_sandbox(sandbox_level)?;
                Ok(CourseAction::Continue)
            }
            KeyCode::Char('w') => {
                self.enter_sandbox_watch_mode(config)?;
                Ok(CourseAction::Continue)
            }
            #[cfg(feature = "llm")]
            KeyCode::Char('a') if self.ai_enabled && self.ai_status == "ready" => {
                self.open_chat();
                Ok(CourseAction::Continue)
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                Ok(CourseAction::Continue)
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                Ok(CourseAction::Continue)
            }
            _ => Ok(CourseAction::Continue),
        }
    }

    fn run_sandbox(&mut self, sandbox_level: SandboxLevel) -> Result<()> {
        // Save before running
        self.save_sandbox_to_disk();

        let sandbox =
            crate::exec::sandbox::Sandbox::new(&self.course.language.limits, sandbox_level)?;

        let file_names: Vec<String> = self.sandbox_code.iter().map(|f| f.name.clone()).collect();
        for file in &self.sandbox_code {
            sandbox.write_file(&file.name, &file.content)?;
        }

        let main_file = self
            .sandbox_code
            .first()
            .map(|f| f.name.clone())
            .unwrap_or_default();

        let mut last_stdout = String::new();
        let mut last_stderr = String::new();

        for step in &self.course.language.steps {
            // Skip validation steps (only run compile/run steps)
            if step.name.to_lowercase().contains("valid") {
                continue;
            }

            let command = crate::exec::placeholder::substitute(
                &step.command,
                sandbox.dir(),
                &main_file,
                &file_names,
            );
            let args: Vec<String> = step
                .args
                .iter()
                .map(|a| {
                    crate::exec::placeholder::substitute(a, sandbox.dir(), &main_file, &file_names)
                })
                .collect();

            let output = sandbox.run_command(&command, &args, None, None, None)?;

            if output.timed_out {
                self.sandbox_last_output = Some(runner::RunOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: false,
                    step_failed: Some(step.name.clone()),
                    timed_out: true,
                    teardown_warnings: Vec::new(),
                });
                self.scroll_offset = 0;
                return Ok(());
            }

            if output.exit_code != 0 {
                self.sandbox_last_output = Some(runner::RunOutput {
                    stdout: output.stdout.clone(),
                    stderr: output.stderr.clone(),
                    success: false,
                    step_failed: Some(step.name.clone()),
                    timed_out: false,
                    teardown_warnings: Vec::new(),
                });
                self.scroll_offset = 0;
                return Ok(());
            }

            last_stdout = output.stdout;
            last_stderr = output.stderr;
        }

        self.sandbox_last_output = Some(runner::RunOutput {
            stdout: last_stdout,
            stderr: last_stderr,
            success: true,
            step_failed: None,
            timed_out: false,
            teardown_warnings: Vec::new(),
        });
        self.scroll_offset = 0;

        Ok(())
    }

    fn launch_sandbox_editor(&mut self, config: &Config) -> Result<()> {
        if self.sandbox_code.is_empty() {
            return Ok(());
        }

        terminal::leave_alternate_screen()?;

        let sandbox_dir = tempfile::tempdir()?;
        let editable_info: Vec<(String, String)> = self
            .sandbox_code
            .iter()
            .map(|f| (f.name.clone(), f.content.clone()))
            .collect();

        for (name, content) in &editable_info {
            let path = sandbox_dir.path().join(name);
            std::fs::write(&path, content)?;
        }

        for (name, _) in &editable_info {
            let path = sandbox_dir.path().join(name);
            let new_content =
                crate::ui::editor::edit_file_with_config(&path, config.editor.as_deref())?;
            if let Some(f) = self.sandbox_code.iter_mut().find(|f| f.name == *name) {
                f.content = new_content;
            }
        }

        terminal::enter_alternate_screen()?;
        self.state = AppState::Sandbox;
        self.scroll_offset = 0;

        Ok(())
    }

    fn enter_sandbox_watch_mode(&mut self, config: &Config) -> Result<()> {
        if self.sandbox_code.is_empty() {
            return Ok(());
        }

        let sandbox_dir = tempfile::tempdir()?;

        let mut watched_files = Vec::new();
        for file in &self.sandbox_code {
            let path = sandbox_dir.path().join(&file.name);
            std::fs::write(&path, &file.content)?;
            watched_files.push((file.name.clone(), path));
        }

        let editor_cmd = crate::ui::editor::detect_editor(config.editor.as_deref());
        let editor_type = crate::ui::editor_detect::resolve_editor_type(
            editor_cmd.as_deref(),
            &config.editor_type,
        );

        let editor_process = if let Some(ref cmd) = editor_cmd {
            let first_file = watched_files.first().map(|(_, p)| p.clone());
            if let Some(file_path) = first_file {
                match editor_type {
                    crate::config::EditorType::Gui => {
                        std::process::Command::new(cmd).arg(&file_path).spawn().ok()
                    }
                    _ => {
                        terminal::leave_alternate_screen()?;
                        let child = std::process::Command::new(cmd).arg(&file_path).spawn().ok();
                        terminal::enter_alternate_screen()?;
                        child
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Ok(ws) = WatchState::new(sandbox_dir, watched_files, editor_process, false) {
            self.watch_state = Some(ws);
            self.sandbox_watching = true;
            self.state = AppState::Watching;
            self.scroll_offset = 0;
        }

        Ok(())
    }

    // --- Watch mode ---

    fn enter_watch_mode(&mut self, config: &Config, auto_test: bool) -> Result<()> {
        if self.current_exercise().is_none() {
            return Ok(());
        }

        let sandbox_dir = tempfile::tempdir()?;

        // Write exercise files to sandbox
        let mut watched_files = Vec::new();
        for file in &self.session.current_code {
            let path = sandbox_dir.path().join(&file.name);
            std::fs::write(&path, &file.content)?;
            if file.editable {
                watched_files.push((file.name.clone(), path));
            }
        }

        // Detect editor and spawn non-blocking
        let editor_cmd = crate::ui::editor::detect_editor(config.editor.as_deref());
        let editor_type = crate::ui::editor_detect::resolve_editor_type(
            editor_cmd.as_deref(),
            &config.editor_type,
        );

        let editor_process = if let Some(ref cmd) = editor_cmd {
            // For GUI editors or explicit watch mode, spawn non-blocking
            let first_editable = watched_files.first().map(|(_, p)| p.clone());
            if let Some(file_path) = first_editable {
                match editor_type {
                    crate::config::EditorType::Gui => {
                        // GUI: spawn and don't wait
                        std::process::Command::new(cmd).arg(&file_path).spawn().ok()
                    }
                    _ => {
                        // Terminal: leave alt screen, run blocking, come back
                        // For watch mode, we still spawn non-blocking since the user explicitly asked
                        terminal::leave_alternate_screen()?;
                        let child = std::process::Command::new(cmd).arg(&file_path).spawn().ok();
                        terminal::enter_alternate_screen()?;
                        child
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        match WatchState::new(sandbox_dir, watched_files, editor_process, auto_test) {
            Ok(ws) => {
                self.watch_state = Some(ws);
                self.state = AppState::Watching;
                self.scroll_offset = 0;
            }
            Err(_) => {
                // Failed to set up watcher, stay in current state
            }
        }

        Ok(())
    }

    fn handle_watching_input(
        &mut self,
        key: KeyCode,
        _progress_store: &mut ProgressStore,
        _sandbox_level: SandboxLevel,
    ) -> CourseAction {
        match key {
            KeyCode::Esc => {
                self.exit_watch_mode();
                CourseAction::Continue
            }
            KeyCode::Char('q') => {
                self.exit_watch_mode();
                CourseAction::Quit
            }
            KeyCode::Char('t') => {
                if let Some(ref mut ws) = self.watch_state {
                    ws.auto_test = !ws.auto_test;
                }
                CourseAction::Continue
            }
            KeyCode::Up => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                CourseAction::Continue
            }
            KeyCode::Down => {
                self.scroll_offset += 1;
                CourseAction::Continue
            }
            _ => CourseAction::Continue,
        }
    }

    pub fn tick_watch_mode(&mut self, sandbox_level: SandboxLevel) {
        let changed = if let Some(ref mut ws) = self.watch_state {
            ws.check_changes()
        } else {
            false
        };

        if !changed {
            return;
        }

        // Read files back from disk
        if let Some(ref ws) = self.watch_state {
            let file_contents = ws.read_files_back();
            if self.sandbox_watching {
                for (name, content) in file_contents {
                    if let Some(f) = self.sandbox_code.iter_mut().find(|f| f.name == name) {
                        f.content = content;
                    }
                }
            } else {
                for (name, content) in file_contents {
                    if let Some(f) = self
                        .session
                        .current_code
                        .iter_mut()
                        .find(|f| f.name == name)
                    {
                        f.content = content;
                    }
                }
            }
        }

        if self.sandbox_watching {
            // Sandbox watch: run without exercise validation
            if let Ok(output) = self.run_sandbox_for_watch(sandbox_level) {
                if let Some(ref mut ws) = self.watch_state {
                    ws.last_watch_output = Some(output);
                }
            }
        } else {
            // Exercise watch: run with exercise validation
            if let Ok(output) = runner::run_exercise_with_sandbox(
                &self.course,
                self.current_exercise().unwrap(),
                &self.session.current_code,
                sandbox_level,
            ) {
                let auto_test = self
                    .watch_state
                    .as_ref()
                    .map(|w| w.auto_test)
                    .unwrap_or(false);

                if auto_test && output.success {
                    // Auto-grade: transition out of watch mode on success
                }

                if let Some(ref mut ws) = self.watch_state {
                    ws.last_watch_output = Some(output);
                }
            }
        }
    }

    fn run_sandbox_for_watch(&self, sandbox_level: SandboxLevel) -> Result<runner::RunOutput> {
        let sandbox =
            crate::exec::sandbox::Sandbox::new(&self.course.language.limits, sandbox_level)?;

        let file_names: Vec<String> = self.sandbox_code.iter().map(|f| f.name.clone()).collect();
        for file in &self.sandbox_code {
            sandbox.write_file(&file.name, &file.content)?;
        }

        let main_file = self
            .sandbox_code
            .first()
            .map(|f| f.name.clone())
            .unwrap_or_default();

        let mut last_stdout = String::new();
        let mut last_stderr = String::new();

        for step in &self.course.language.steps {
            if step.name.to_lowercase().contains("valid") {
                continue;
            }

            let command = crate::exec::placeholder::substitute(
                &step.command,
                sandbox.dir(),
                &main_file,
                &file_names,
            );
            let args: Vec<String> = step
                .args
                .iter()
                .map(|a| {
                    crate::exec::placeholder::substitute(a, sandbox.dir(), &main_file, &file_names)
                })
                .collect();

            let output = sandbox.run_command(&command, &args, None, None, None)?;

            if output.timed_out {
                return Ok(runner::RunOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    success: false,
                    step_failed: Some(step.name.clone()),
                    timed_out: true,
                    teardown_warnings: Vec::new(),
                });
            }

            if output.exit_code != 0 {
                return Ok(runner::RunOutput {
                    stdout: output.stdout,
                    stderr: output.stderr,
                    success: false,
                    step_failed: Some(step.name.clone()),
                    timed_out: false,
                    teardown_warnings: Vec::new(),
                });
            }

            last_stdout = output.stdout;
            last_stderr = output.stderr;
        }

        Ok(runner::RunOutput {
            stdout: last_stdout,
            stderr: last_stderr,
            success: true,
            step_failed: None,
            timed_out: false,
            teardown_warnings: Vec::new(),
        })
    }

    fn exit_watch_mode(&mut self) {
        // Read final file state back
        if let Some(ref ws) = self.watch_state {
            let file_contents = ws.read_files_back();
            if self.sandbox_watching {
                for (name, content) in file_contents {
                    if let Some(f) = self.sandbox_code.iter_mut().find(|f| f.name == name) {
                        f.content = content;
                    }
                }
            } else {
                for (name, content) in file_contents {
                    if let Some(f) = self
                        .session
                        .current_code
                        .iter_mut()
                        .find(|f| f.name == name)
                    {
                        f.content = content;
                    }
                }
            }
        }

        let was_sandbox = self.sandbox_watching;
        self.watch_state = None;
        self.sandbox_watching = false;
        self.state = if was_sandbox {
            AppState::Sandbox
        } else {
            AppState::ExercisePrompt
        };
        self.scroll_offset = 0;
    }

    // --- Progress management ---

    fn course_id(&self) -> String {
        self.course.name.to_lowercase().replace(' ', "-")
    }

    fn ensure_course_progress(&self, progress_store: &mut ProgressStore) {
        let key = {
            let cid = self.course_id();
            crate::state::types::progress_key(&cid, &self.course.version)
        };
        progress_store
            .data
            .courses
            .entry(key)
            .or_insert_with(|| CourseProgress {
                course_version: self.course.version.clone(),
                started_at: chrono::Utc::now().to_rfc3339(),
                last_activity: chrono::Utc::now().to_rfc3339(),
                lessons: std::collections::HashMap::new(),
            });
    }

    fn get_current_ids(&self) -> (String, String, String) {
        let cid = self.course_id();
        let key = crate::state::types::progress_key(&cid, &self.course.version);
        let lesson_id = self
            .current_lesson()
            .map(|l| l.id.clone())
            .unwrap_or_default();
        let exercise_id = self
            .current_exercise()
            .map(|e| e.id.clone())
            .unwrap_or_default();
        (key, lesson_id, exercise_id)
    }

    fn record_attempt(&self, attempt: &AttemptRecord, progress_store: &mut ProgressStore) {
        self.ensure_course_progress(progress_store);
        let (key, lesson_id, exercise_id) = self.get_current_ids();

        if let Some(course_progress) = progress_store.data.courses.get_mut(&key) {
            course_progress.last_activity = chrono::Utc::now().to_rfc3339();

            let lesson_progress =
                course_progress
                    .lessons
                    .entry(lesson_id)
                    .or_insert_with(|| LessonProgress {
                        status: ProgressStatus::InProgress,
                        completed_at: None,
                        exercises: std::collections::HashMap::new(),
                    });

            let exercise_progress =
                lesson_progress
                    .exercises
                    .entry(exercise_id)
                    .or_insert_with(|| ExerciseProgress {
                        status: ProgressStatus::InProgress,
                        attempts: Vec::new(),
                    });

            exercise_progress.attempts.push(attempt.clone());
        }

        let _ = progress_store.save();
    }

    fn mark_exercise_completed(&self, progress_store: &mut ProgressStore) {
        self.ensure_course_progress(progress_store);
        let (key, lesson_id, exercise_id) = self.get_current_ids();

        if let Some(course_progress) = progress_store.data.courses.get_mut(&key) {
            if let Some(lesson_progress) = course_progress.lessons.get_mut(&lesson_id) {
                if let Some(ex) = lesson_progress.exercises.get_mut(&exercise_id) {
                    ex.status = ProgressStatus::Completed;
                }
            }
        }

        let _ = progress_store.save();
        self.clear_current_draft();
    }

    fn mark_exercise_skipped(&self, progress_store: &mut ProgressStore) {
        self.ensure_course_progress(progress_store);
        let (key, lesson_id, exercise_id) = self.get_current_ids();

        if let Some(course_progress) = progress_store.data.courses.get_mut(&key) {
            let lesson_progress =
                course_progress
                    .lessons
                    .entry(lesson_id)
                    .or_insert_with(|| LessonProgress {
                        status: ProgressStatus::InProgress,
                        completed_at: None,
                        exercises: std::collections::HashMap::new(),
                    });

            let exercise_progress =
                lesson_progress
                    .exercises
                    .entry(exercise_id)
                    .or_insert_with(|| ExerciseProgress {
                        status: ProgressStatus::InProgress,
                        attempts: Vec::new(),
                    });

            exercise_progress.status = ProgressStatus::Skipped;
        }

        let _ = progress_store.save();
    }

    fn mark_lesson_completed(&self, progress_store: &mut ProgressStore) {
        self.ensure_course_progress(progress_store);
        let (key, lesson_id, _) = self.get_current_ids();

        if let Some(course_progress) = progress_store.data.courses.get_mut(&key) {
            if let Some(lesson_progress) = course_progress.lessons.get_mut(&lesson_id) {
                lesson_progress.status = ProgressStatus::Completed;
                lesson_progress.completed_at = Some(chrono::Utc::now().to_rfc3339());
            }
        }

        let _ = progress_store.save();
    }

    // --- LLM integration methods ---

    #[cfg(feature = "llm")]
    pub fn drain_llm_events(&mut self) {
        let channel = match &self.llm_channel {
            Some(ch) => ch,
            None => return,
        };

        while let Ok(event) = channel.response_rx.try_recv() {
            match event {
                LlmEvent::BackendReady(_name) => {
                    self.ai_status = "ready".to_string();
                }
                LlmEvent::BackendUnavailable(_msg) => {
                    self.ai_status = "offline".to_string();
                }
                LlmEvent::Token(token) => {
                    if let Some(ref mut chat) = self.chat_state {
                        chat.append_token(&token);
                    }
                }
                LlmEvent::Done(full_text) => {
                    if let Some(ref mut chat) = self.chat_state {
                        chat.push_assistant_message(full_text);
                    }
                    self.ai_status = "ready".to_string();
                }
                LlmEvent::Error(msg) => {
                    if let Some(ref mut chat) = self.chat_state {
                        chat.push_assistant_message(format!("[Error: {}]", msg));
                    }
                    self.ai_status = "ready".to_string();
                }
            }
        }
    }

    #[cfg(feature = "llm")]
    pub fn shutdown_llm(&mut self) {
        if let Some(ref channel) = self.llm_channel {
            let _ = channel.request_tx.send(LlmRequest::Shutdown);
        }
    }

    #[cfg(feature = "llm")]
    fn open_chat(&mut self) {
        self.chat_visible = true;
    }

    #[cfg(feature = "llm")]
    fn send_quick_action(&mut self, prompt: &str) {
        self.chat_visible = true;
        self.send_chat_message(prompt.to_string());
    }

    #[cfg(feature = "llm")]
    fn send_chat_message(&mut self, content: String) {
        if let Some(ref mut chat) = self.chat_state {
            chat.push_user_message(content);
            chat.is_streaming = true;
        }
        self.ai_status = "streaming...".to_string();

        let context = self.build_llm_context();

        let messages: Vec<ChatMessage> = self
            .chat_state
            .as_ref()
            .map(|c| c.messages.clone())
            .unwrap_or_default();

        if let Some(ref channel) = self.llm_channel {
            let _ = channel
                .request_tx
                .send(LlmRequest::Chat { context, messages });
        }
    }

    #[cfg(feature = "llm")]
    fn build_llm_context(&self) -> LlmContext {
        // Sandbox mode: use sandbox-specific context
        if self.state == AppState::Sandbox {
            let lesson = self
                .course
                .loaded_lessons
                .get(self.sandbox_lesson_idx)
                .expect("no lesson for sandbox");
            let last_output = self.sandbox_last_output.as_ref().map(|o| {
                if o.success {
                    o.stdout.as_str()
                } else if o.timed_out {
                    "Execution timed out"
                } else {
                    o.stderr.as_str()
                }
            });
            return LlmContext::assemble_sandbox(
                &self.course,
                lesson,
                &self.sandbox_code,
                last_output,
                self.sandbox_lesson_idx,
            );
        }

        // Shell mode: use shell transcript as context
        if self.state == AppState::Shell {
            let lesson = self.current_lesson().expect("no lesson");
            let exercise = self.current_exercise().expect("no exercise");
            let include_content = self
                .llm_config
                .as_ref()
                .map(|c| c.settings.include_lesson_content)
                .unwrap_or(true);
            let max_history = self
                .llm_config
                .as_ref()
                .map(|c| c.settings.max_history_attempts)
                .unwrap_or(3);
            let empty_store = ProgressStore::empty();
            let mut ctx = LlmContext::assemble(
                &self.course,
                lesson,
                exercise,
                &self.session,
                &empty_store,
                self.current_lesson_idx,
                include_content,
                max_history,
            );
            ctx.current_code = self.format_shell_history_for_llm();
            ctx.last_execution_summary = "See shell transcript in current_code".to_string();
            return ctx;
        }

        let lesson = self.current_lesson().expect("no lesson");
        let exercise = self.current_exercise().expect("no exercise");
        let include_content = self
            .llm_config
            .as_ref()
            .map(|c| c.settings.include_lesson_content)
            .unwrap_or(true);
        let max_history = self
            .llm_config
            .as_ref()
            .map(|c| c.settings.max_history_attempts)
            .unwrap_or(3);

        let empty_store = ProgressStore::empty();

        LlmContext::assemble(
            &self.course,
            lesson,
            exercise,
            &self.session,
            &empty_store,
            self.current_lesson_idx,
            include_content,
            max_history,
        )
    }

    #[cfg(feature = "llm")]
    fn format_shell_history_for_llm(&self) -> String {
        let shell = match &self.shell_state {
            Some(s) => s,
            None => return String::new(),
        };
        let entries = &shell.history;
        // Cap at last 20 entries to bound context size
        let start = entries.len().saturating_sub(20);
        let mut lines = Vec::new();
        for entry in &entries[start..] {
            lines.push(format!("$ {}", entry.command));
            if !entry.stdout.is_empty() {
                lines.push(entry.stdout.clone());
            }
            if !entry.stderr.is_empty() {
                lines.push(format!("[stderr] {}", entry.stderr));
            }
            if entry.timed_out {
                lines.push("[timed out]".to_string());
            } else if entry.exit_code != 0 {
                lines.push(format!("[exit code: {}]", entry.exit_code));
            }
        }
        lines.join("\n")
    }

    #[cfg(feature = "llm")]
    fn handle_chat_input(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Esc => {
                self.chat_visible = false;
            }
            // Ctrl+Enter or Alt+Enter = send message
            KeyCode::Enter
                if modifiers.contains(KeyModifiers::CONTROL)
                    || modifiers.contains(KeyModifiers::ALT) =>
            {
                self.try_send_chat();
            }
            // Plain Enter = newline (up to 3 lines)
            KeyCode::Enter => {
                if let Some(ref mut chat) = self.chat_state {
                    let line_count = chat.input_buffer.chars().filter(|c| *c == '\n').count();
                    if line_count < 2 {
                        chat.input_buffer.push('\n');
                    }
                }
            }
            // Tab = send (alternative to Ctrl+Enter for terminals that eat it)
            KeyCode::Tab => {
                self.try_send_chat();
            }
            KeyCode::Char(c) => {
                if let Some(ref mut chat) = self.chat_state {
                    chat.input_buffer.push(c);
                }
            }
            KeyCode::Backspace => {
                if let Some(ref mut chat) = self.chat_state {
                    chat.input_buffer.pop();
                }
            }
            KeyCode::Up => {
                if let Some(ref mut chat) = self.chat_state {
                    chat.scroll_offset = chat.scroll_offset.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if let Some(ref mut chat) = self.chat_state {
                    chat.scroll_offset += 1;
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "llm")]
    fn try_send_chat(&mut self) {
        let is_streaming = self
            .chat_state
            .as_ref()
            .map(|c| c.is_streaming)
            .unwrap_or(false);
        if is_streaming {
            return;
        }
        let input = self
            .chat_state
            .as_ref()
            .map(|c| c.input_buffer.clone())
            .unwrap_or_default();
        if input.trim().is_empty() {
            return;
        }
        if let Some(ref mut chat) = self.chat_state {
            chat.input_buffer.clear();
        }
        self.send_chat_message(input);
    }

    #[cfg(feature = "llm")]
    fn render_chat_panel(&self, frame: &mut ratatui::Frame, area: Rect, theme: &Theme) {
        let chat = match &self.chat_state {
            Some(c) => c,
            None => return,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(5)])
            .split(area);

        let mut lines: Vec<Line<'static>> = Vec::new();
        for msg in &chat.messages {
            match msg.role {
                ChatRole::User => {
                    lines.push(Line::from(Span::styled(
                        "You: ".to_string(),
                        Style::default()
                            .fg(ratatui::style::Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )));
                    for text_line in msg.content.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", text_line),
                            Style::default().fg(theme.body_text),
                        )));
                    }
                }
                ChatRole::Assistant => {
                    lines.push(Line::from(Span::styled(
                        "AI: ".to_string(),
                        Style::default()
                            .fg(ratatui::style::Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )));
                    let md_lines = markdown::render_markdown(&msg.content, theme);
                    lines.extend(md_lines);
                }
                ChatRole::System => continue,
            }
            lines.push(Line::from(""));
        }

        if chat.is_streaming {
            let epoch_ms = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as usize;

            if chat.streaming_buffer.is_empty() {
                // Spinner while waiting for first token
                let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let spinner = spinner_frames[(epoch_ms / 100) % spinner_frames.len()];
                lines.push(Line::from(vec![
                    Span::styled(
                        "AI: ".to_string(),
                        Style::default()
                            .fg(ratatui::style::Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{} Thinking...", spinner),
                        Style::default().fg(ratatui::style::Color::Yellow),
                    ),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    "AI: ".to_string(),
                    Style::default()
                        .fg(ratatui::style::Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                // Render streaming buffer with markdown too
                let md_lines = markdown::render_markdown(&chat.streaming_buffer, theme);
                lines.extend(md_lines);
                let dots_frames = [".", "..", "..."];
                let dots = dots_frames[(epoch_ms / 400) % dots_frames.len()];
                lines.push(Line::from(Span::styled(
                    format!("  {}", dots),
                    Style::default().fg(ratatui::style::Color::DarkGray),
                )));
            }
        }

        if lines.is_empty() {
            lines.push(Line::from(Span::styled(
                "  Type a message or use quick actions".to_string(),
                Style::default().fg(ratatui::style::Color::DarkGray),
            )));
        }

        let messages_widget = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" AI Chat ({}) ", self.ai_status))
                    .border_style(Style::default().fg(if self.chat_visible {
                        ratatui::style::Color::Cyan
                    } else {
                        ratatui::style::Color::DarkGray
                    })),
            )
            .wrap(Wrap { trim: false })
            .scroll((chat.scroll_offset, 0));
        frame.render_widget(messages_widget, chunks[0]);

        // Multi-line input area (3 lines of text + border)
        let cursor_char = if chat.is_streaming { "" } else { "\u{2588}" };
        let mut input_lines: Vec<Line<'static>> = Vec::new();
        let buf_lines: Vec<&str> = chat.input_buffer.split('\n').collect();
        for (i, line) in buf_lines.iter().enumerate() {
            let prefix = if i == 0 { "> " } else { "  " };
            if i == buf_lines.len() - 1 {
                input_lines.push(Line::from(format!("{}{}{}", prefix, line, cursor_char)));
            } else {
                input_lines.push(Line::from(format!("{}{}", prefix, line)));
            }
        }

        let send_hint = if chat.is_streaming {
            ""
        } else {
            "  Ctrl+Enter/Tab: send"
        };
        let input_widget = Paragraph::new(input_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Input{} ", send_hint))
                    .border_style(Style::default().fg(ratatui::style::Color::DarkGray)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(input_widget, chunks[1]);
    }
}

fn find_resume_lesson(course: &Course, store: &ProgressStore) -> usize {
    let course_id = course.name.to_lowercase().replace(' ', "-");
    let key = crate::state::types::progress_key(&course_id, &course.version);

    if let Some(cp) = store.data.courses.get(&key) {
        for (i, lesson) in course.loaded_lessons.iter().enumerate() {
            if let Some(lp) = cp.lessons.get(&lesson.id) {
                if lp.status != ProgressStatus::Completed {
                    return i;
                }
            } else {
                return i;
            }
        }
    }
    0
}

fn find_resume_exercise(course: &Course, store: &ProgressStore, lesson_idx: usize) -> usize {
    let course_id = course.name.to_lowercase().replace(' ', "-");
    let key = crate::state::types::progress_key(&course_id, &course.version);

    if let Some(cp) = store.data.courses.get(&key) {
        if let Some(lesson) = course.loaded_lessons.get(lesson_idx) {
            if let Some(lp) = cp.lessons.get(&lesson.id) {
                for (i, exercise) in lesson.loaded_exercises.iter().enumerate() {
                    if let Some(ep) = lp.exercises.get(&exercise.id) {
                        if ep.status != ProgressStatus::Completed {
                            return i;
                        }
                    } else {
                        return i;
                    }
                }
            }
        }
    }
    0
}

fn get_tips_for_state(state: &AppState) -> &'static [&'static str] {
    match state {
        AppState::ExercisePrompt => &[
            "[w] Watch mode auto-runs code on save",
            "[E] opens your external editor (vim, VS Code)",
            "[h] reveals hints one at a time",
            "[Enter] runs without grading, [t] submits for validation",
            "[r] resets code to the original starter",
            "[s] skip if you're stuck",
        ],
        AppState::LessonContent => &[
            "[Space] reveals content progressively",
            "[\u{2191}/\u{2193}] focus sections, [PgUp/PgDn] scroll within",
            "[\u{2190}/\u{2192}] jump between lessons",
        ],
        AppState::ResultFail => &[
            "[h] reveals hints progressively",
            "[Enter] go back to try again",
            "[e] edit your code and retry",
            "[r] reset code to starter and try fresh",
        ],
        AppState::RunResult => &[
            "[t] submit for grading when ready",
            "[e] edit and try again",
        ],
        AppState::Watching => &[
            "[t] toggles auto-test on save",
            "Edit files externally, output updates here",
        ],
        AppState::Sandbox => &[
            "[E] opens your external editor",
            "[w] watch mode — edit externally, see output live",
            "No grading here — just experiment freely",
            "Your code is saved automatically on exit",
        ],
        _ => &[],
    }
}

fn dim_lines(lines: &mut Vec<Line<'static>>, dim_color: Color) {
    for line in lines.iter_mut() {
        let spans: Vec<Span<'static>> = line
            .spans
            .drain(..)
            .map(|span| Span::styled(span.content, span.style.fg(dim_color)))
            .collect();
        *line = Line::from(spans);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_transitions() {
        assert_eq!(AppState::LessonContent, AppState::LessonContent);
        assert_ne!(AppState::LessonContent, AppState::ExercisePrompt);
    }
}
