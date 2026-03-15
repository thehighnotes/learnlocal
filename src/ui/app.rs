use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::{Config, EditorType, SandboxLevelPref};
use crate::course::types::{Course, CourseInfo};
use crate::error::Result;
use crate::exec::sandbox::SandboxLevel;
use crate::exec::toolcheck;
use crate::state::progress::ProgressStore;
use crate::state::types::*;
use crate::ui::celebration::{format_duration, AggregateStats};
use crate::ui::course_app::CourseApp;
use crate::ui::howto;
use crate::ui::screens::*;
use crate::ui::terminal::Tui;
use crate::ui::theme::Theme;
use crate::ui::tour;

#[cfg(feature = "llm")]
use crate::llm::channel::LlmChannel;

pub struct App {
    pub screen: Screen,
    pub theme: Theme,
    pub config: Config,
    pub progress_store: ProgressStore,
    pub courses: Vec<CourseInfo>,
    pub sandbox_level: SandboxLevel,
    pub should_quit: bool,
    pub home: HomeState,
    pub howto: HowToState,
    pub tour: TourState,
    pub stats: StatsState,
    pub settings: SettingsState,
    pub progress_view: ProgressViewState,
    pub layout_cache: LayoutCache,
    pub course_app: Option<CourseApp>,
    #[allow(dead_code)]
    pub courses_dir: PathBuf,
    // LLM fields at outer level — persist across screens
    #[cfg(feature = "llm")]
    pub llm_channel: Option<LlmChannel>,
}

impl App {
    pub fn new(
        courses: Vec<CourseInfo>,
        progress_store: ProgressStore,
        config: Config,
        sandbox_level: SandboxLevel,
        courses_dir: PathBuf,
    ) -> Self {
        let theme = Theme::new(&config.theme);
        let mut home = HomeState::new();
        home.summaries = build_course_summaries(&courses, &progress_store);
        home.display_order = build_display_order(&home.summaries);

        let mut settings = SettingsState::new();
        settings.editor_value = config.editor.clone().unwrap_or_default();
        settings.editor_type_value = config.editor_type.to_string();
        settings.theme_value = config.theme.to_string();
        settings.sandbox_value = match config.sandbox_level {
            SandboxLevelPref::Auto => "auto".to_string(),
            SandboxLevelPref::Basic => "basic".to_string(),
            SandboxLevelPref::Contained => "contained".to_string(),
        };
        #[cfg(feature = "llm")]
        {
            settings.ai_enabled = config.llm.enabled;
            settings.ollama_url = config.llm.ollama.url.clone();
            settings.ollama_model = config.llm.ollama.model.clone();
        }

        Self {
            screen: Screen::Home,
            theme,
            config,
            progress_store,
            courses,
            sandbox_level,
            should_quit: false,
            home,
            howto: HowToState::new(),
            tour: TourState::new(),
            stats: StatsState::new(),
            settings,
            progress_view: ProgressViewState::new(),
            layout_cache: LayoutCache::default(),
            course_app: None,
            courses_dir,
            #[cfg(feature = "llm")]
            llm_channel: None,
        }
    }

    /// Create an App that jumps directly into a course (preserves `learnlocal start` behavior).
    pub fn new_with_course(
        course: Course,
        progress_store: ProgressStore,
        config: Config,
        sandbox_level: SandboxLevel,
        start_lesson: Option<&str>,
        courses_dir: PathBuf,
    ) -> Self {
        let theme = Theme::new(&config.theme);
        let mut course_app = CourseApp::new(course, &progress_store, start_lesson, None);
        course_app.sandbox_level = sandbox_level;

        Self {
            screen: Screen::Course,
            theme,
            config,
            progress_store,
            courses: Vec::new(),
            sandbox_level,
            should_quit: false,
            home: HomeState::new(),
            howto: HowToState::new(),
            tour: TourState::new(),
            stats: StatsState::new(),
            settings: SettingsState::new(),
            progress_view: ProgressViewState::new(),
            layout_cache: LayoutCache::default(),
            course_app: Some(course_app),
            courses_dir,
            #[cfg(feature = "llm")]
            llm_channel: None,
        }
    }

    #[cfg(feature = "llm")]
    pub fn enable_ai(&mut self, channel: LlmChannel) {
        self.config.llm.enabled = true;
        self.llm_channel = Some(channel);
        // If a course_app already exists, forward AI to it
        // If not, AI will be forwarded when a course is started
    }

    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        // If we started directly in Course screen with AI, forward it now
        #[cfg(feature = "llm")]
        if self.config.llm.enabled && self.screen == Screen::Course {
            self.forward_ai_to_course_app();
        }

        while !self.should_quit {
            // Check for termination signals (SIGTERM/SIGINT/SIGHUP)
            if crate::ui::terminal::signal_received() {
                if let Some(ref ca) = self.course_app {
                    ca.save_draft_to_disk();
                }
                break;
            }

            // Drain LLM events from course_app if active
            #[cfg(feature = "llm")]
            if let Some(ref mut ca) = self.course_app {
                ca.drain_llm_events();
            }

            // Tick watch mode if course_app is in Watching state
            if let Some(ref mut ca) = self.course_app {
                if ca.state == crate::ui::course_app::AppState::Watching {
                    ca.tick_watch_mode(self.sandbox_level);
                }
            }

            terminal.draw(|f| self.render(f))?;
            self.handle_input()?;
        }

        #[cfg(feature = "llm")]
        if let Some(ref mut ca) = self.course_app {
            ca.shutdown_llm();
        }

        Ok(())
    }

    #[cfg(feature = "llm")]
    fn forward_ai_to_course_app(&mut self) {
        if let Some(ref mut ca) = self.course_app {
            if !ca.ai_enabled {
                if let Some(channel) = self.llm_channel.take() {
                    ca.enable_ai(channel, self.config.llm.clone());
                    // We can't put it back since enable_ai consumed it,
                    // but the channel is now owned by course_app
                }
            }
        }
    }

    // --- Rendering ---

    fn render(&mut self, frame: &mut ratatui::Frame) {
        match self.screen {
            Screen::Home => self.render_home(frame),
            Screen::HowTo => self.render_howto(frame),
            Screen::Tour => self.render_tour(frame),
            Screen::Stats => self.render_stats(frame),
            Screen::Settings => self.render_settings(frame),
            Screen::Progress => self.render_progress(frame),
            Screen::Course => {
                let theme = self.theme.clone();
                if let Some(ref mut ca) = self.course_app {
                    ca.render(frame, &theme);
                }
            }
        };
    }

    fn render_home(&mut self, frame: &mut ratatui::Frame) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // title header
                Constraint::Min(1),    // content area
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        // Title header with box-drawing (33 chars wide: ╔ + 31×═ + ╗)
        let title_lines = vec![
            Line::from(Span::styled(
                "  \u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}",
                Style::default().fg(self.theme.cursor),
            )),
            Line::from(Span::styled(
                "  \u{2551}      L E A R N L O C A L      \u{2551}",
                Style::default()
                    .fg(self.theme.cursor)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  \u{255A}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255D}",
                Style::default().fg(self.theme.cursor),
            )),
        ];
        let title_widget = Paragraph::new(title_lines);
        frame.render_widget(title_widget, outer[0]);

        if self.home.summaries.is_empty() {
            // Empty state: styled welcome
            let lines: Vec<Line<'static>> = vec![
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled(
                    "  Welcome to LearnLocal!",
                    Style::default()
                        .fg(self.theme.heading)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  No courses found in courses/",
                    Style::default().fg(self.theme.body_text),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "  Get started:",
                    Style::default().fg(self.theme.body_text),
                )),
                Line::from(Span::styled(
                    "    1. Place a course directory in courses/",
                    Style::default().fg(self.theme.muted),
                )),
                Line::from(Span::styled(
                    "    2. Or run: learnlocal validate <path>",
                    Style::default().fg(self.theme.muted),
                )),
            ];

            let content = Paragraph::new(lines);
            frame.render_widget(content, outer[1]);
        } else {
            let total_width = outer[1].width;
            if total_width < 100 {
                // Narrow: single-panel, full-width course list
                self.layout_cache.home_left = outer[1];
                self.layout_cache.home_right = ratatui::layout::Rect::default();
                self.render_home_left_panel(frame, outer[1]);
            } else {
                let constraints = if total_width >= 160 {
                    // Wide: cap left panel at 60 cols
                    [Constraint::Length(60), Constraint::Min(1)]
                } else {
                    // Normal: 40/60 split
                    [Constraint::Percentage(40), Constraint::Percentage(60)]
                };
                let panels = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(constraints)
                    .split(outer[1]);

                self.layout_cache.home_left = panels[0];
                self.layout_cache.home_right = panels[1];
                self.render_home_left_panel(frame, panels[0]);
                self.render_home_right_panel(frame, panels[1]);
            }
        }

        // Key bar — changes based on panel focus, startability, and width
        let startable = self.home.is_course_startable(self.home.flat_idx());
        let narrow = outer[1].width < 100;
        let key_text = match self.home.focus {
            HomePanelFocus::CourseList => {
                if startable && !narrow {
                    " [Enter] Start  [\u{2192}] Lessons  [\u{2191}/\u{2193}] Navigate  [w] Tour  [h] How To  [t] Stats  [p] Progress  [s] Settings  [q] Quit"
                } else if startable {
                    " [Enter] Start  [\u{2191}/\u{2193}] Navigate  [w] Tour  [h] How To  [s] Settings  [q] Quit"
                } else if !narrow {
                    " [\u{2192}] Lessons  [\u{2191}/\u{2193}] Navigate  [w] Tour  [h] How To  [t] Stats  [p] Progress  [s] Settings  [q] Quit"
                } else {
                    " [\u{2191}/\u{2193}] Navigate  [w] Tour  [h] How To  [s] Settings  [q] Quit"
                }
            }
            HomePanelFocus::LessonList => {
                if startable {
                    " [Enter] Start Lesson  [s] Sandbox  [\u{2190}] Back  [\u{2191}/\u{2193}] Navigate"
                } else {
                    " [\u{2190}] Back  [\u{2191}/\u{2193}] Navigate"
                }
            }
        };
        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(key_text, outer[2].width as usize),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, outer[2]);
    }

    fn render_home_left_panel(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let mut left_lines: Vec<Line<'static>> = Vec::new();
        self.layout_cache.home_course_ys.clear();

        // COURSES header
        left_lines.push(Line::from(Span::styled(
            "  COURSES",
            Style::default().fg(self.theme.muted),
        )));
        left_lines.push(Line::from(""));

        // Ensure caches are sized correctly
        while self.home.tool_check_cache.len() < self.home.summaries.len() {
            self.home.tool_check_cache.push(None);
        }
        while self.home.platform_check_cache.len() < self.home.summaries.len() {
            self.home.platform_check_cache.push(None);
        }

        // Lazily check platform for selected course
        let sel = self.home.flat_idx();
        if sel < self.home.summaries.len() && self.home.platform_check_cache[sel].is_none() {
            let platform = &self.home.summaries[sel].info.platform;
            self.home.platform_check_cache[sel] = Some(toolcheck::check_platform(platform));
        }

        // Lazily check tools for selected course (language steps + env commands)
        // Embedded provision courses always show as ready (no external tools needed)
        if sel < self.home.summaries.len() && self.home.tool_check_cache[sel].is_none() {
            use crate::course::types::Provision;
            if self.home.summaries[sel].info.provision == Provision::Embedded {
                self.home.tool_check_cache[sel] = Some(Vec::new());
            } else {
                let step_cmds = &self.home.summaries[sel].info.step_commands;
                let env_cmds = &self.home.summaries[sel].info.env_commands;
                if !step_cmds.is_empty() || !env_cmds.is_empty() {
                    let mut seen = std::collections::HashSet::new();
                    let mut statuses = Vec::new();
                    for cmd in step_cmds.iter().chain(env_cmds.iter()) {
                        if seen.insert(cmd.clone()) {
                            let found = toolcheck::command_exists(cmd);
                            statuses.push(toolcheck::ToolStatus {
                                command: cmd.clone(),
                                found,
                                install_hint: if !found {
                                    toolcheck::suggest_install(cmd)
                                } else {
                                    None
                                },
                            });
                        }
                    }
                    self.home.tool_check_cache[sel] = Some(statuses);
                } else {
                    self.home.tool_check_cache[sel] = Some(Vec::new());
                }
            }
        }

        // Group by language
        // Tuple: (flat_idx, completed, total, total_lessons, issue_label, tools_ready, startable)
        #[allow(clippy::type_complexity)]
        let mut groups: BTreeMap<
            String,
            Vec<(usize, usize, usize, usize, String, bool, bool)>,
        > = BTreeMap::new();
        for (i, summary) in self.home.summaries.iter().enumerate() {
            // Check tool readiness from cache
            let all_ready = self
                .home
                .tool_check_cache
                .get(i)
                .and_then(|c| c.as_ref())
                .map(|statuses| statuses.iter().all(|s| s.found))
                .unwrap_or(true); // assume ready if not checked

            let platform_ok = self
                .home
                .platform_check_cache
                .get(i)
                .and_then(|c| c.as_ref())
                .map(|ps| ps.supported)
                .unwrap_or(true);

            let missing_tool = self
                .home
                .tool_check_cache
                .get(i)
                .and_then(|c| c.as_ref())
                .and_then(|statuses| statuses.iter().find(|s| !s.found))
                .map(|s| s.command.clone())
                .unwrap_or_default();

            let issue_label = if !platform_ok {
                let req = summary.info.platform.as_deref().unwrap_or("?");
                format!(" {} only", req)
            } else if !all_ready {
                format!(" Needs {}", missing_tool)
            } else {
                String::new()
            };

            let startable = all_ready && platform_ok;

            groups
                .entry(summary.info.language_name.clone())
                .or_default()
                .push((
                    i,
                    summary.completed_exercises,
                    summary.total_exercises,
                    summary.total_lessons,
                    issue_label,
                    all_ready,
                    startable,
                ));
        }

        // Find max course name width for alignment
        let max_name_len = self
            .home
            .summaries
            .iter()
            .map(|s| s.info.name.len())
            .max()
            .unwrap_or(20);
        let name_col = max_name_len + 3; // extra padding before bar

        let bar_width = 12usize;
        for (lang, entries) in &groups {
            // Language header
            left_lines.push(Line::from(Span::styled(
                format!("  {}", lang),
                Style::default()
                    .fg(self.theme.warning)
                    .add_modifier(Modifier::BOLD),
            )));

            for &(
                flat_idx,
                completed,
                total,
                _total_lessons,
                ref issue_label,
                _tools_ready,
                startable,
            ) in entries
            {
                let selected = flat_idx == self.home.flat_idx();
                let cursor = if selected && self.home.focus == HomePanelFocus::CourseList {
                    "\u{25b6}"
                } else if selected {
                    "\u{2022}" // bullet when focused on right panel
                } else {
                    " "
                };

                let pct = if total > 0 {
                    completed * 100 / total
                } else {
                    0
                };
                let filled = if total > 0 {
                    (pct * bar_width) / 100
                } else {
                    0
                };
                let empty = bar_width - filled;

                let name_style = if !startable {
                    Style::default().fg(self.theme.muted)
                } else if selected {
                    Style::default()
                        .fg(self.theme.heading)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.theme.body_text)
                };

                let name = &self.home.summaries[flat_idx].info.name;

                let mut spans = vec![
                    Span::styled(format!("  {} ", cursor), name_style),
                    Span::styled(format!("{:<width$}", name, width = name_col), name_style),
                    Span::styled(
                        "\u{2588}".repeat(filled),
                        Style::default().fg(self.theme.progress_filled),
                    ),
                    Span::styled(
                        "\u{2591}".repeat(empty),
                        Style::default().fg(self.theme.progress_empty),
                    ),
                    Span::styled(
                        format!(" {:>3}%", pct),
                        Style::default().fg(self.theme.muted),
                    ),
                ];

                // Issue indicator (platform or tool)
                if !issue_label.is_empty() {
                    let color = if !startable
                        && self.home.summaries[flat_idx].info.platform.is_some()
                        && !self
                            .home
                            .platform_check_cache
                            .get(flat_idx)
                            .and_then(|c| c.as_ref())
                            .map(|ps| ps.supported)
                            .unwrap_or(true)
                    {
                        self.theme.error
                    } else {
                        self.theme.warning
                    };
                    spans.push(Span::styled(
                        issue_label.clone(),
                        Style::default().fg(color),
                    ));
                }

                self.layout_cache
                    .home_course_ys
                    .push((flat_idx, area.y + left_lines.len() as u16));
                left_lines.push(Line::from(spans));
            }

            left_lines.push(Line::from(""));
        }

        let border_color = match self.home.focus {
            HomePanelFocus::CourseList => self.theme.border_active,
            HomePanelFocus::LessonList => self.theme.border_inactive,
        };
        let left_block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(border_color));
        let left = Paragraph::new(left_lines).block(left_block);
        frame.render_widget(left, area);
    }

    fn render_home_right_panel(&mut self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        let mut right_lines: Vec<Line<'static>> = Vec::new();
        self.layout_cache.home_lesson_ys.clear();

        if let Some(summary) = self.home.summaries.get(self.home.flat_idx()) {
            let info = &summary.info;

            // Progress lookup for per-lesson status
            let course_id = info.name.to_lowercase().replace(' ', "-");
            let key = crate::state::types::progress_key(&course_id, &info.version);
            let cp = self.progress_store.data.courses.get(&key);

            right_lines.push(Line::from(""));

            // Course name
            right_lines.push(Line::from(Span::styled(
                format!("  {}", info.name),
                Style::default()
                    .fg(self.theme.heading)
                    .add_modifier(Modifier::BOLD),
            )));
            // Separator under course name
            right_lines.push(Line::from(Span::styled(
                "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(self.theme.muted),
            )));
            right_lines.push(Line::from(""));

            // Description
            right_lines.push(Line::from(Span::styled(
                format!("  {}", info.description),
                Style::default().fg(self.theme.body_text),
            )));
            right_lines.push(Line::from(""));

            // Metadata
            right_lines.push(Line::from(vec![
                Span::styled("  Author: ", Style::default().fg(self.theme.muted)),
                Span::styled(
                    info.author.clone(),
                    Style::default().fg(self.theme.body_text),
                ),
            ]));
            right_lines.push(Line::from(vec![
                Span::styled("  Version: ", Style::default().fg(self.theme.muted)),
                Span::styled(
                    info.version.clone(),
                    Style::default().fg(self.theme.body_text),
                ),
            ]));
            if let Some(ref license) = info.license {
                right_lines.push(Line::from(vec![
                    Span::styled("  License: ", Style::default().fg(self.theme.muted)),
                    Span::styled(license.clone(), Style::default().fg(self.theme.body_text)),
                ]));
            }
            right_lines.push(Line::from(""));

            // Lesson count + exercise count + time estimate
            let exercise_str = info
                .total_exercise_count
                .map(|n| format!(" \u{00b7} {} exercises", n))
                .unwrap_or_default();
            let time_str = if let Some(mins) = info.estimated_minutes_per_lesson {
                format!(" \u{00b7} ~{} min each", mins)
            } else {
                String::new()
            };
            right_lines.push(Line::from(Span::styled(
                format!(
                    "  {} lessons{}{}",
                    info.lesson_count, exercise_str, time_str
                ),
                Style::default().fg(self.theme.muted),
            )));

            // Course status
            let status_text = match summary.status {
                CourseStatus::NotStarted => ("Not Started", self.theme.muted),
                CourseStatus::InProgress => {
                    let completed_lessons = summary.completed_lessons;
                    let total = summary.total_lessons;
                    let text = format!("In Progress ({}/{} lessons)", completed_lessons, total);
                    // We need to leak the string to get a 'static lifetime — ok for rendering
                    (Box::leak(text.into_boxed_str()) as &str, self.theme.warning)
                }
                CourseStatus::Completed => ("Completed", self.theme.success),
            };
            right_lines.push(Line::from(Span::styled(
                format!("  Status: {}", status_text.0),
                Style::default().fg(status_text.1),
            )));

            // Tool requirements
            if let Some(Some(ref statuses)) = self.home.tool_check_cache.get(self.home.flat_idx()) {
                if !statuses.is_empty() {
                    right_lines.push(Line::from(""));
                    for status in statuses {
                        if status.found {
                            right_lines.push(Line::from(Span::styled(
                                format!("  \u{2713} {} found", status.command),
                                Style::default().fg(self.theme.success),
                            )));
                        } else {
                            right_lines.push(Line::from(Span::styled(
                                format!("  \u{26a0} {} not found", status.command),
                                Style::default().fg(self.theme.warning),
                            )));
                            if let Some(ref hint) = status.install_hint {
                                right_lines.push(Line::from(Span::styled(
                                    format!("    {}", hint),
                                    Style::default().fg(self.theme.muted),
                                )));
                            }
                        }
                    }
                }
            }

            // Platform requirement
            if let Some(Some(ref ps)) = self.home.platform_check_cache.get(self.home.flat_idx()) {
                if ps.required.is_some() {
                    let req = ps.required.as_deref().unwrap_or("?");
                    if ps.supported {
                        right_lines.push(Line::from(Span::styled(
                            format!("  \u{2713} Platform: {} (current)", req),
                            Style::default().fg(self.theme.success),
                        )));
                    } else {
                        right_lines.push(Line::from(Span::styled(
                            format!("  \u{2717} Requires {} (you are on {})", req, ps.current),
                            Style::default().fg(self.theme.error),
                        )));
                    }
                }
            }

            right_lines.push(Line::from(""));

            // Lesson list with per-lesson progress
            let lessons_focused = self.home.focus == HomePanelFocus::LessonList;
            let header_text = if lessons_focused {
                "  Lessons: (select one)"
            } else {
                "  Lessons:"
            };
            right_lines.push(Line::from(Span::styled(
                header_text,
                Style::default()
                    .fg(self.theme.body_text)
                    .add_modifier(Modifier::BOLD),
            )));

            // Determine "current" lesson: first non-completed lesson
            let mut found_current = false;
            for (i, (lesson_id, lesson_title)) in info
                .lesson_ids
                .iter()
                .zip(info.lesson_titles.iter())
                .enumerate()
            {
                let is_complete = cp
                    .and_then(|cp| cp.lessons.get(lesson_id))
                    .map(|lp| lp.status == ProgressStatus::Completed)
                    .unwrap_or(false);

                let lesson_selected = lessons_focused && i == self.home.right_selected_idx;

                let (icon, icon_color) = if lesson_selected {
                    ("\u{25b6}", self.theme.cursor)
                } else if is_complete {
                    ("\u{2713}", self.theme.success)
                } else if !found_current {
                    found_current = true;
                    ("\u{2022}", self.theme.cursor) // bullet for current/next
                } else {
                    (" ", self.theme.muted)
                };

                let text_style = if lesson_selected {
                    Style::default()
                        .fg(self.theme.heading)
                        .add_modifier(Modifier::BOLD)
                } else if is_complete {
                    Style::default().fg(self.theme.muted)
                } else {
                    Style::default().fg(self.theme.body_text)
                };

                self.layout_cache
                    .home_lesson_ys
                    .push((i, area.y + right_lines.len() as u16));
                right_lines.push(Line::from(vec![
                    Span::styled(format!("  {} ", icon), Style::default().fg(icon_color)),
                    Span::styled(format!("{}. {}", i + 1, lesson_title), text_style),
                ]));
            }
        } else {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(
                "  No course selected",
                Style::default().fg(self.theme.muted),
            )));
        }

        let right = Paragraph::new(right_lines).wrap(Wrap { trim: false });
        frame.render_widget(right, area);
    }

    fn render_howto(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title bar
                Constraint::Min(1),    // content
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        // Title bar
        let title_bar = Paragraph::new(Line::from(Span::styled(
            " LearnLocal | How To Use",
            Style::default()
                .fg(self.theme.title_bar_fg)
                .bg(self.theme.title_bar_bg)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(title_bar, chunks[0]);

        // Build context for slides that need file paths
        let ctx = howto::HowToCtx {
            config_path: dirs::config_dir()
                .map(|d| {
                    d.join("learnlocal")
                        .join("config.yaml")
                        .display()
                        .to_string()
                })
                .unwrap_or_else(|| "~/.config/learnlocal/config.yaml".to_string()),
            progress_path: dirs::data_dir()
                .map(|d| {
                    d.join("learnlocal")
                        .join("progress.json")
                        .display()
                        .to_string()
                })
                .unwrap_or_else(|| "~/.local/share/learnlocal/progress.json".to_string()),
            sandbox_path: dirs::data_dir()
                .map(|d| d.join("learnlocal").join("sandboxes").display().to_string())
                .unwrap_or_else(|| "~/.local/share/learnlocal/sandboxes/".to_string()),
            courses_path: self.courses_dir.display().to_string(),
        };

        // Content — centered in available area
        let lines = howto::build_slide(self.howto.slide_index, &self.theme, &ctx);
        let content_height = lines.len() as u16;
        let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(0);

        let area = chunks[1];
        let v_pad = area.height.saturating_sub(content_height) / 2;
        let h_pad = area.width.saturating_sub(content_width) / 2;

        let centered = ratatui::layout::Rect::new(
            area.x + h_pad,
            area.y + v_pad,
            area.width.saturating_sub(h_pad),
            content_height.min(area.height.saturating_sub(v_pad)),
        );

        let content = Paragraph::new(lines);
        frame.render_widget(content, centered);

        // Key bar with slide counter
        let slide_num = self.howto.slide_index + 1;
        let slide_total = howto::SLIDE_COUNT;
        let key_text = format!(
            " [\u{2190}/\u{2192}] Navigate  [1-7] Jump  Page {}/{}  [Esc] Back",
            slide_num, slide_total,
        );
        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(&key_text, chunks[2].width as usize),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, chunks[2]);
    }

    fn handle_howto_input(&mut self, key: KeyCode) {
        let max = howto::SLIDE_COUNT.saturating_sub(1);
        match key {
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                if self.howto.slide_index < max {
                    self.howto.slide_index += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.howto.slide_index > 0 {
                    self.howto.slide_index -= 1;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Home;
            }
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let idx = (c as usize) - ('1' as usize);
                if idx <= max {
                    self.howto.slide_index = idx;
                }
            }
            _ => {}
        }
    }

    fn render_tour(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title bar
                Constraint::Min(1),    // content
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        // Title bar
        let title_bar = Paragraph::new(Line::from(Span::styled(
            " LearnLocal | Welcome Tour",
            Style::default()
                .fg(self.theme.title_bar_fg)
                .bg(self.theme.title_bar_bg)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(title_bar, chunks[0]);

        // Build course names for the last slide
        let course_names: Vec<String> = self.courses.iter().map(|c| c.name.clone()).collect();

        // Content — centered in available area
        let lines = tour::build_slide(self.tour.slide_index, &self.theme, &course_names);
        let content_height = lines.len() as u16;
        let content_width = lines.iter().map(|l| l.width() as u16).max().unwrap_or(0);

        let area = chunks[1];
        let v_pad = area.height.saturating_sub(content_height) / 2;
        let h_pad = area.width.saturating_sub(content_width) / 2;

        let centered = ratatui::layout::Rect::new(
            area.x + h_pad,
            area.y + v_pad,
            area.width.saturating_sub(h_pad),
            content_height.min(area.height.saturating_sub(v_pad)),
        );

        let content = Paragraph::new(lines);
        frame.render_widget(content, centered);

        // Key bar with slide counter
        let slide_num = self.tour.slide_index + 1;
        let slide_total = tour::SLIDE_COUNT;
        let key_text = format!(
            " [\u{2190}/\u{2192}] Navigate  [1-9] Jump  Slide {}/{}  [Esc] Back",
            slide_num, slide_total,
        );
        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(&key_text, chunks[2].width as usize),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, chunks[2]);
    }

    fn handle_tour_input(&mut self, key: KeyCode) {
        let max = tour::SLIDE_COUNT.saturating_sub(1);
        match key {
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => {
                if self.tour.slide_index < max {
                    self.tour.slide_index += 1;
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.tour.slide_index > 0 {
                    self.tour.slide_index -= 1;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Home;
            }
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let idx = (c as usize) - ('1' as usize);
                if idx <= max {
                    self.tour.slide_index = idx;
                }
            }
            _ => {}
        }
    }

    fn render_stats(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title bar
                Constraint::Min(1),    // content
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        // Title bar
        let title_bar = Paragraph::new(Line::from(Span::styled(
            " LearnLocal | Your Stats",
            Style::default()
                .fg(self.theme.title_bar_fg)
                .bg(self.theme.title_bar_bg)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(title_bar, chunks[0]);

        // Compute stats fresh each render
        let stats = AggregateStats::compute(&self.home.summaries, &self.progress_store);

        let heading = self.theme.heading;
        let body = self.theme.body_text;
        let muted = self.theme.muted;

        let mut lines: Vec<Line<'static>> = Vec::new();

        // OVERALL section
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  OVERALL",
            Style::default().fg(heading).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(muted),
        )));

        lines.push(Line::from(vec![
            Span::styled("  Courses:    ", Style::default().fg(muted)),
            Span::styled(
                format!(
                    "{} started, {} completed",
                    stats.courses_started, stats.courses_completed
                ),
                Style::default().fg(body),
            ),
        ]));

        let pct = if stats.exercises_total > 0 {
            stats.exercises_completed * 100 / stats.exercises_total
        } else {
            0
        };
        lines.push(Line::from(vec![
            Span::styled("  Exercises:  ", Style::default().fg(muted)),
            Span::styled(
                format!(
                    "{}/{} completed ({}%)",
                    stats.exercises_completed, stats.exercises_total, pct
                ),
                Style::default().fg(body),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled("  Time:       ", Style::default().fg(muted)),
            Span::styled(
                format!("{} invested", format_duration(stats.total_time_seconds)),
                Style::default().fg(body),
            ),
        ]));

        if stats.exercises_completed > 0 {
            let ft_pct = stats.first_try_count * 100 / stats.exercises_completed;
            lines.push(Line::from(vec![
                Span::styled("  First-try:  ", Style::default().fg(muted)),
                Span::styled(
                    format!(
                        "{}% ({}/{})",
                        ft_pct, stats.first_try_count, stats.exercises_completed
                    ),
                    Style::default().fg(body),
                ),
            ]));

            let hf_pct = stats.hint_free_count * 100 / stats.exercises_completed;
            lines.push(Line::from(vec![
                Span::styled("  Hint-free:  ", Style::default().fg(muted)),
                Span::styled(
                    format!(
                        "{}% ({}/{})",
                        hf_pct, stats.hint_free_count, stats.exercises_completed
                    ),
                    Style::default().fg(body),
                ),
            ]));
        }

        lines.push(Line::from(""));

        // PER COURSE section
        lines.push(Line::from(Span::styled(
            "  PER COURSE",
            Style::default().fg(heading).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(muted),
        )));

        // Find max course name width for alignment
        let max_name = stats
            .per_course
            .iter()
            .map(|c| c.name.len())
            .max()
            .unwrap_or(20);

        for pc in &stats.per_course {
            let progress = format!("{:>3}/{:<3}", pc.exercises_done, pc.exercises_total);
            let time = if pc.time_seconds > 0 {
                format_duration(pc.time_seconds)
            } else {
                "\u{2014}".to_string() // em dash
            };
            let started = if pc.started_date.is_empty() {
                "Not started".to_string()
            } else {
                format!("Started {}", pc.started_date)
            };

            let status_color = if pc.completed {
                self.theme.success
            } else if pc.exercises_done > 0 {
                self.theme.warning
            } else {
                muted
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {:<width$}  ", pc.name, width = max_name),
                    Style::default().fg(body),
                ),
                Span::styled(progress, Style::default().fg(status_color)),
                Span::styled(format!("  {:>7}", time), Style::default().fg(muted)),
                Span::styled(format!("   {}", started), Style::default().fg(muted)),
            ]));
        }

        lines.push(Line::from(""));

        // PATHS section
        lines.push(Line::from(Span::styled(
            "  PATHS",
            Style::default().fg(heading).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            "  \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
            Style::default().fg(muted),
        )));

        let config_path = dirs::config_dir()
            .map(|d| {
                d.join("learnlocal")
                    .join("config.yaml")
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| "~/.config/learnlocal/config.yaml".to_string());
        let data_path = dirs::data_dir()
            .map(|d| {
                d.join("learnlocal")
                    .join("progress.json")
                    .display()
                    .to_string()
            })
            .unwrap_or_else(|| "~/.local/share/learnlocal/progress.json".to_string());
        let sandbox_path = dirs::data_dir()
            .map(|d| d.join("learnlocal").join("sandboxes").display().to_string())
            .unwrap_or_else(|| "~/.local/share/learnlocal/sandboxes/".to_string());
        let courses_path = self.courses_dir.display().to_string();

        lines.push(Line::from(vec![
            Span::styled("  Config:     ", Style::default().fg(muted)),
            Span::styled(config_path, Style::default().fg(body)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Progress:   ", Style::default().fg(muted)),
            Span::styled(data_path, Style::default().fg(body)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Sandboxes:  ", Style::default().fg(muted)),
            Span::styled(sandbox_path, Style::default().fg(body)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Courses:    ", Style::default().fg(muted)),
            Span::styled(courses_path, Style::default().fg(body)),
        ]));
        lines.push(Line::from(""));

        self.stats.content_height = lines.len() as u16;

        // Clamp scroll
        let viewport = chunks[1].height;
        let max_scroll = self.stats.content_height.saturating_sub(viewport);
        if self.stats.scroll_offset > max_scroll {
            self.stats.scroll_offset = max_scroll;
        }

        let content = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.stats.scroll_offset, 0));
        frame.render_widget(content, chunks[1]);

        // Scroll indicators
        if self.stats.content_height > viewport {
            let indicator_style = Style::default()
                .fg(self.theme.warning)
                .add_modifier(Modifier::BOLD);
            if self.stats.scroll_offset > 0 {
                let r = ratatui::layout::Rect::new(
                    chunks[1].x + chunks[1].width.saturating_sub(9),
                    chunks[1].y,
                    9.min(chunks[1].width),
                    1,
                );
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(" \u{25b2} more ", indicator_style))),
                    r,
                );
            }
            if self.stats.scroll_offset < max_scroll {
                let r = ratatui::layout::Rect::new(
                    chunks[1].x + chunks[1].width.saturating_sub(9),
                    chunks[1].y + chunks[1].height.saturating_sub(1),
                    9.min(chunks[1].width),
                    1,
                );
                frame.render_widget(
                    Paragraph::new(Line::from(Span::styled(" \u{25bc} more ", indicator_style))),
                    r,
                );
            }
        }

        // Key bar
        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(
                " [\u{2191}/\u{2193}] Scroll  [Esc] Back",
                chunks[2].width as usize,
            ),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, chunks[2]);
    }

    fn handle_stats_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.stats.scroll_offset = 0;
                self.screen = Screen::Home;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.stats.scroll_offset = self.stats.scroll_offset.saturating_sub(3);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.stats.scroll_offset = self.stats.scroll_offset.saturating_add(3);
            }
            KeyCode::PageUp => {
                self.stats.scroll_offset = self.stats.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.stats.scroll_offset = self.stats.scroll_offset.saturating_add(20);
            }
            KeyCode::Home => {
                self.stats.scroll_offset = 0;
            }
            KeyCode::End => {
                self.stats.scroll_offset = self.stats.content_height;
            }
            _ => {}
        }
    }

    fn render_settings(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title bar
                Constraint::Min(1),    // settings content
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        // Title
        let title_bar = Paragraph::new(Line::from(Span::styled(
            " LearnLocal | Settings",
            Style::default()
                .fg(self.theme.title_bar_fg)
                .bg(self.theme.title_bar_bg)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(title_bar, chunks[0]);

        // Settings fields
        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  GENERAL",
            Style::default()
                .fg(self.theme.heading)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        self.layout_cache.settings_field_ys.clear();

        for (i, field) in self.settings.fields.iter().enumerate() {
            let focused = i == self.settings.focused_idx;
            let cursor = if focused { ">" } else { " " };
            let style = if focused {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Section headers for AI fields
            #[cfg(feature = "llm")]
            if *field == SettingsField::AiEnabled && i > 0 {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  AI",
                    Style::default()
                        .fg(self.theme.heading)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
            }

            let (label, value) = match field {
                SettingsField::Editor => {
                    let val = if self.settings.editing && focused {
                        format!("[{}]", self.settings.edit_buffer)
                    } else if self.settings.editor_value.is_empty() {
                        "(default: $EDITOR)".to_string()
                    } else {
                        self.settings.editor_value.clone()
                    };
                    ("Editor", val)
                }
                SettingsField::EditorType => {
                    let val = format!("< {} >", self.settings.editor_type_value);
                    ("Editor Type", val)
                }
                SettingsField::SandboxLevel => {
                    let val = format!("< {} >", self.settings.sandbox_value);
                    ("Sandbox Level", val)
                }
                SettingsField::Theme => {
                    let val = format!("< {} >", self.settings.theme_value);
                    ("Theme", val)
                }
                #[cfg(feature = "llm")]
                SettingsField::AiEnabled => {
                    let val = if self.settings.ai_enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    };
                    ("AI Hints", val.to_string())
                }
                #[cfg(feature = "llm")]
                SettingsField::OllamaUrl => {
                    let val = if self.settings.editing && focused {
                        format!("[{}]", self.settings.edit_buffer)
                    } else {
                        self.settings.ollama_url.clone()
                    };
                    ("Ollama URL", val)
                }
                #[cfg(feature = "llm")]
                SettingsField::OllamaModel => {
                    let val = if self.settings.model_picker_open {
                        "(selecting...)".to_string()
                    } else if self.settings.editing && focused {
                        format!("[{}]", self.settings.edit_buffer)
                    } else {
                        self.settings.ollama_model.clone()
                    };
                    ("Model", val)
                }
            };

            // chunks[1].y is the content area origin; lines.len() is the current line index
            self.layout_cache
                .settings_field_ys
                .push(chunks[1].y + lines.len() as u16);
            lines.push(Line::from(Span::styled(
                format!("  {} {:<18} {}", cursor, label, value),
                style,
            )));
        }

        // Model picker overlay
        #[cfg(feature = "llm")]
        if self.settings.model_picker_open && !self.settings.available_models.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Available models:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            for (i, model) in self.settings.available_models.iter().enumerate() {
                let sel = if i == self.settings.model_picker_idx {
                    ">"
                } else {
                    " "
                };
                let s = if i == self.settings.model_picker_idx {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                lines.push(Line::from(Span::styled(
                    format!("    {} {}", sel, model),
                    s,
                )));
            }
        }

        let content = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(content, chunks[1]);

        // Key bar
        let keys = if self.settings.editing {
            "[Enter] Confirm  [Esc] Cancel"
        } else {
            #[cfg(feature = "llm")]
            {
                if self.settings.model_picker_open {
                    "[Enter] Select  [Esc] Cancel"
                } else {
                    "[Enter] Edit  [Left/Right] Toggle  [Esc] Save & Back"
                }
            }
            #[cfg(not(feature = "llm"))]
            "[Enter] Edit  [Left/Right] Toggle  [Esc] Save & Back"
        };

        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(&format!(" {}", keys), chunks[2].width as usize),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, chunks[2]);
    }

    fn render_progress(&mut self, frame: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title bar
                Constraint::Min(1),    // content
                Constraint::Length(1), // key bar
            ])
            .split(frame.size());

        let course = match &self.progress_view.course {
            Some(c) => c,
            None => {
                let msg = Paragraph::new("  Loading course...");
                frame.render_widget(msg, chunks[1]);
                return;
            }
        };

        // Title
        let title_bar = Paragraph::new(Line::from(Span::styled(
            format!(" LearnLocal | {} v{}", course.name, course.version),
            Style::default()
                .fg(self.theme.title_bar_fg)
                .bg(self.theme.title_bar_bg)
                .add_modifier(Modifier::BOLD),
        )));
        frame.render_widget(title_bar, chunks[0]);

        // Progress content
        let course_id = course.name.to_lowercase().replace(' ', "-");
        let key = crate::state::types::progress_key(&course_id, &course.version);
        let cp = self.progress_store.data.courses.get(&key);

        let mut lines: Vec<Line<'static>> = Vec::new();
        lines.push(Line::from(""));

        if let Some(cp) = cp {
            lines.push(Line::from(format!("  Started: {}", &cp.started_at[..10])));
            lines.push(Line::from(format!(
                "  Last active: {}",
                &cp.last_activity[..10]
            )));
        }

        let total_lessons = course.loaded_lessons.len();
        let completed_lessons = course
            .loaded_lessons
            .iter()
            .filter(|l| {
                cp.and_then(|cp| cp.lessons.get(&l.id))
                    .map(|lp| lp.status == ProgressStatus::Completed)
                    .unwrap_or(false)
            })
            .count();

        let total_exercises: usize = course
            .loaded_lessons
            .iter()
            .map(|l| l.loaded_exercises.len())
            .sum();
        let completed_exercises: usize = course
            .loaded_lessons
            .iter()
            .map(|l| {
                cp.and_then(|cp| cp.lessons.get(&l.id))
                    .map(|lp| {
                        lp.exercises
                            .values()
                            .filter(|e| e.status == ProgressStatus::Completed)
                            .count()
                    })
                    .unwrap_or(0)
            })
            .sum();

        let pct = if total_exercises > 0 {
            completed_exercises * 100 / total_exercises
        } else {
            0
        };

        lines.push(Line::from(format!(
            "  Overall: {}%  ({}/{} lessons, {}/{} exercises)",
            pct, completed_lessons, total_lessons, completed_exercises, total_exercises
        )));
        lines.push(Line::from(""));
        self.layout_cache.progress_lesson_ys.clear();

        for (i, lesson) in course.loaded_lessons.iter().enumerate() {
            let total_ex = lesson.loaded_exercises.len();
            let completed_ex = cp
                .and_then(|cp| cp.lessons.get(&lesson.id))
                .map(|lp| {
                    lp.exercises
                        .values()
                        .filter(|e| e.status == ProgressStatus::Completed)
                        .count()
                })
                .unwrap_or(0);

            let is_complete = cp
                .and_then(|cp| cp.lessons.get(&lesson.id))
                .map(|lp| lp.status == ProgressStatus::Completed)
                .unwrap_or(false);

            let status_icon = if is_complete {
                "[x]"
            } else if completed_ex > 0 {
                "[~]"
            } else {
                "[ ]"
            };

            let selected = i == self.progress_view.selected_lesson_idx;
            let cursor = if selected { ">" } else { " " };
            let style = if selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let sandbox_marker = if crate::state::sandbox::has_sandbox_files(
                &course_id,
                &course.version,
                &lesson.id,
            ) {
                " S"
            } else {
                "  "
            };

            self.layout_cache
                .progress_lesson_ys
                .push(chunks[1].y + lines.len() as u16);
            lines.push(Line::from(Span::styled(
                format!(
                    "  {} {} {:02}. {:<24} {}/{} exercises{}",
                    cursor,
                    status_icon,
                    i + 1,
                    lesson.title,
                    completed_ex,
                    total_ex,
                    sandbox_marker,
                ),
                style,
            )));
        }

        // Reset confirmation
        if self.progress_view.confirm_reset {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Reset all progress for this course? [y] Yes  [any] Cancel",
                Style::default()
                    .fg(self.theme.warning)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let content = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(content, chunks[1]);

        // Key bar
        let key_text = if self.progress_view.confirm_reset {
            " [y] Confirm Reset  [any] Cancel"
        } else {
            " [Enter] Resume from here  [s] Sandbox  [r] Reset  [Esc] Back"
        };
        let key_bar = Paragraph::new(Line::from(Span::styled(
            truncate_key_bar(key_text, chunks[2].width as usize),
            Style::default()
                .fg(self.theme.key_bar_fg)
                .bg(self.theme.key_bar_bg),
        )));
        frame.render_widget(key_bar, chunks[2]);
    }

    // --- Input ---

    fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Global Ctrl+C
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        if let Some(ref ca) = self.course_app {
                            ca.save_draft_to_disk();
                        }
                        self.should_quit = true;
                        return Ok(());
                    }

                    match self.screen {
                        Screen::Home => self.handle_home_input(key.code),
                        Screen::HowTo => self.handle_howto_input(key.code),
                        Screen::Tour => self.handle_tour_input(key.code),
                        Screen::Stats => self.handle_stats_input(key.code),
                        Screen::Settings => self.handle_settings_input(key.code),
                        Screen::Progress => self.handle_progress_input(key.code)?,
                        Screen::Course => self.handle_course_input(key)?,
                    }
                }
                Event::Mouse(mouse) => {
                    self.handle_mouse(mouse);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }

        let col = mouse.column;
        let row = mouse.row;

        match self.screen {
            Screen::Home => {
                // Click in left panel — select course
                let left = self.layout_cache.home_left;
                if col >= left.x
                    && col < left.x + left.width
                    && row >= left.y
                    && row < left.y + left.height
                {
                    for &(flat_idx, y) in &self.layout_cache.home_course_ys {
                        if row == y {
                            if let Some(pos) = self
                                .home
                                .display_order
                                .iter()
                                .position(|&di| di == flat_idx)
                            {
                                self.home.selected_idx = pos;
                                self.home.right_selected_idx = 0;
                                self.home.focus = HomePanelFocus::CourseList;
                            }
                            break;
                        }
                    }
                }
                // Click in right panel — select lesson
                let right = self.layout_cache.home_right;
                if col >= right.x
                    && col < right.x + right.width
                    && row >= right.y
                    && row < right.y + right.height
                {
                    for &(lesson_idx, y) in &self.layout_cache.home_lesson_ys {
                        if row == y {
                            self.home.right_selected_idx = lesson_idx;
                            self.home.focus = HomePanelFocus::LessonList;
                            break;
                        }
                    }
                }
            }
            Screen::Settings => {
                if self.settings.editing {
                    return;
                }
                #[cfg(feature = "llm")]
                if self.settings.model_picker_open {
                    return;
                }
                for (i, &y) in self.layout_cache.settings_field_ys.iter().enumerate() {
                    if row == y {
                        self.settings.focused_idx = i;
                        break;
                    }
                }
            }
            Screen::Progress => {
                if self.progress_view.confirm_reset {
                    return;
                }
                for (i, &y) in self.layout_cache.progress_lesson_ys.iter().enumerate() {
                    if row == y {
                        self.progress_view.selected_lesson_idx = i;
                        break;
                    }
                }
            }
            _ => {} // Tour, HowTo, Stats, Course — keyboard-only
        }
    }

    fn handle_home_input(&mut self, key: KeyCode) {
        match self.home.focus {
            HomePanelFocus::CourseList => match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.home.selected_idx > 0 {
                        self.home.selected_idx -= 1;
                        self.home.right_selected_idx = 0;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.home.selected_idx + 1 < self.home.display_order.len() {
                        self.home.selected_idx += 1;
                        self.home.right_selected_idx = 0;
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Only switch to lesson panel if it's visible (not narrow layout)
                    if self.layout_cache.home_right.width > 0 && !self.home.summaries.is_empty() {
                        let idx = self.home.flat_idx();
                        if idx < self.home.summaries.len()
                            && !self.home.summaries[idx].info.lesson_ids.is_empty()
                        {
                            self.home.focus = HomePanelFocus::LessonList;
                        }
                    }
                }
                KeyCode::Enter => {
                    if self.home.is_course_startable(self.home.flat_idx()) {
                        self.start_selected_course(None);
                    }
                }
                KeyCode::Char('p') => {
                    self.open_progress_for_selected();
                }
                KeyCode::Char('s') => {
                    self.screen = Screen::Settings;
                }
                KeyCode::Char('h') => {
                    self.howto.slide_index = 0;
                    self.screen = Screen::HowTo;
                }
                KeyCode::Char('t') => {
                    self.stats.scroll_offset = 0;
                    self.screen = Screen::Stats;
                }
                KeyCode::Char('w') => {
                    self.tour.slide_index = 0;
                    self.screen = Screen::Tour;
                }
                _ => {}
            },
            HomePanelFocus::LessonList => match key {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                    self.home.focus = HomePanelFocus::CourseList;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.home.right_selected_idx > 0 {
                        self.home.right_selected_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let idx = self.home.flat_idx();
                    if idx < self.home.summaries.len() {
                        let max = self.home.summaries[idx]
                            .info
                            .lesson_ids
                            .len()
                            .saturating_sub(1);
                        if self.home.right_selected_idx < max {
                            self.home.right_selected_idx += 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if self.home.is_course_startable(self.home.flat_idx()) {
                        let lesson_idx = self.home.right_selected_idx;
                        self.start_selected_course(Some(lesson_idx));
                    }
                }
                KeyCode::Char('s') => {
                    if self.home.is_course_startable(self.home.flat_idx()) {
                        let lesson_idx = self.home.right_selected_idx;
                        self.start_selected_course(Some(lesson_idx));
                        if let Some(ref mut ca) = self.course_app {
                            ca.enter_sandbox(lesson_idx);
                        }
                    }
                }
                KeyCode::Char('w') => {
                    self.tour.slide_index = 0;
                    self.screen = Screen::Tour;
                }
                _ => {}
            },
        }
    }

    fn handle_settings_input(&mut self, key: KeyCode) {
        // Model picker sub-state
        #[cfg(feature = "llm")]
        if self.settings.model_picker_open {
            match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.settings.model_picker_idx > 0 {
                        self.settings.model_picker_idx -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.settings.model_picker_idx + 1 < self.settings.available_models.len() {
                        self.settings.model_picker_idx += 1;
                    }
                }
                KeyCode::Enter => {
                    if let Some(model) = self
                        .settings
                        .available_models
                        .get(self.settings.model_picker_idx)
                    {
                        self.settings.ollama_model = model.clone();
                    }
                    self.settings.model_picker_open = false;
                }
                KeyCode::Esc => {
                    self.settings.model_picker_open = false;
                }
                _ => {}
            }
            return;
        }

        // Editing sub-state
        if self.settings.editing {
            match key {
                KeyCode::Enter => {
                    self.apply_settings_edit();
                    self.settings.editing = false;
                }
                KeyCode::Esc => {
                    self.settings.editing = false;
                }
                KeyCode::Char(c) => {
                    self.settings.edit_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.settings.edit_buffer.pop();
                }
                _ => {}
            }
            return;
        }

        // Normal navigation
        match key {
            KeyCode::Esc => {
                self.save_settings();
                self.screen = Screen::Home;
                // Refresh home summaries since config may have changed
                self.home.summaries = build_course_summaries(&self.courses, &self.progress_store);
                self.home.display_order = build_display_order(&self.home.summaries);
                self.home.tool_check_cache.clear();
                self.home.platform_check_cache.clear();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings.focused_idx > 0 {
                    self.settings.focused_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings.focused_idx + 1 < self.settings.fields.len() {
                    self.settings.focused_idx += 1;
                }
            }
            KeyCode::Enter => {
                self.start_settings_edit();
            }
            KeyCode::Left | KeyCode::Right => {
                self.toggle_settings_field(key);
            }
            _ => {}
        }
    }

    fn handle_progress_input(&mut self, key: KeyCode) -> Result<()> {
        // Confirmation sub-state for reset
        if self.progress_view.confirm_reset {
            match key {
                KeyCode::Char('y') => {
                    // Reset progress for this course
                    if let Some(ref course) = self.progress_view.course {
                        // Best-effort backup before destructive reset
                        let _ = self.progress_store.backup();

                        let course_id = course.name.to_lowercase().replace(' ', "-");
                        let keys_to_remove: Vec<String> = self
                            .progress_store
                            .data
                            .courses
                            .keys()
                            .filter(|k| k.starts_with(&format!("{}@", course_id)))
                            .cloned()
                            .collect();
                        for k in &keys_to_remove {
                            self.progress_store.data.courses.remove(k);
                        }
                        let _ = self.progress_store.save();
                        // Refresh home summaries
                        self.home.summaries =
                            build_course_summaries(&self.courses, &self.progress_store);
                        self.home.display_order = build_display_order(&self.home.summaries);
                    }
                    self.progress_view.confirm_reset = false;
                }
                _ => {
                    self.progress_view.confirm_reset = false;
                }
            }
            return Ok(());
        }

        let startable = self.home.is_course_startable(self.home.flat_idx());
        match key {
            KeyCode::Esc => {
                self.screen = Screen::Home;
                self.progress_view.course = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.progress_view.selected_lesson_idx > 0 {
                    self.progress_view.selected_lesson_idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let max = self
                    .progress_view
                    .course
                    .as_ref()
                    .map(|c| c.loaded_lessons.len().saturating_sub(1))
                    .unwrap_or(0);
                if self.progress_view.selected_lesson_idx < max {
                    self.progress_view.selected_lesson_idx += 1;
                }
            }
            KeyCode::Enter => {
                if startable {
                    let lesson_idx = self.progress_view.selected_lesson_idx;
                    self.start_selected_course(Some(lesson_idx));
                }
            }
            KeyCode::Char('s') => {
                if startable {
                    let lesson_idx = self.progress_view.selected_lesson_idx;
                    self.start_selected_course(Some(lesson_idx));
                    if let Some(ref mut ca) = self.course_app {
                        ca.enter_sandbox(lesson_idx);
                    }
                }
            }
            KeyCode::Char('r') => {
                self.progress_view.confirm_reset = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_course_input(&mut self, key: crossterm::event::KeyEvent) -> Result<()> {
        let action = if let Some(ref mut ca) = self.course_app {
            ca.handle_input(
                key,
                &mut self.progress_store,
                &self.config,
                self.sandbox_level,
            )?
        } else {
            CourseAction::GoHome
        };

        match action {
            CourseAction::Continue => {}
            CourseAction::Quit => {
                if let Some(ref ca) = self.course_app {
                    ca.save_draft_to_disk();
                }
                self.should_quit = true;
            }
            CourseAction::GoHome => {
                if let Some(ref ca) = self.course_app {
                    ca.save_draft_to_disk();
                }
                self.course_app = None;
                self.screen = Screen::Home;
                // Refresh summaries with updated progress
                self.home.summaries = build_course_summaries(&self.courses, &self.progress_store);
                self.home.display_order = build_display_order(&self.home.summaries);
                self.home.tool_check_cache.clear();
                self.home.platform_check_cache.clear();
            }
        }

        Ok(())
    }

    // --- Actions ---

    fn start_selected_course(&mut self, lesson_idx: Option<usize>) {
        if self.home.summaries.is_empty() {
            return;
        }

        let idx = self.home.flat_idx();
        let source_dir = self.home.summaries[idx].info.source_dir.clone();

        match crate::course::load_course(&source_dir) {
            Ok(course) => {
                // Shut down any existing LLM thread before starting a new course
                #[cfg(feature = "llm")]
                if let Some(ref mut old_ca) = self.course_app {
                    old_ca.shutdown_llm();
                }

                let mut ca = CourseApp::new(course, &self.progress_store, None, lesson_idx);
                ca.sandbox_level = self.sandbox_level;

                // Forward AI if enabled in config
                #[cfg(feature = "llm")]
                if self.config.llm.enabled {
                    let channel = crate::llm::ollama::spawn_llm_thread(self.config.llm.clone());
                    ca.enable_ai(channel, self.config.llm.clone());
                }

                self.course_app = Some(ca);
                self.screen = Screen::Course;
            }
            Err(e) => {
                // Stay on home, could show error but for now just log
                eprintln!("Failed to load course: {}", e);
            }
        }
    }

    fn open_progress_for_selected(&mut self) {
        if self.home.summaries.is_empty() {
            return;
        }

        let idx = self.home.flat_idx();
        let source_dir = self.home.summaries[idx].info.source_dir.clone();

        if let Ok(course) = crate::course::load_course(&source_dir) {
            self.progress_view.course_idx = idx;
            self.progress_view.course = Some(course);
            self.progress_view.selected_lesson_idx = 0;
            self.screen = Screen::Progress;
        }
    }

    fn start_settings_edit(&mut self) {
        let field = self.settings.focused_field().clone();
        match field {
            SettingsField::Editor => {
                self.settings.editing = true;
                self.settings.edit_buffer = self.settings.editor_value.clone();
            }
            SettingsField::EditorType => {
                self.toggle_settings_field(KeyCode::Right);
            }
            SettingsField::SandboxLevel => {
                self.toggle_settings_field(KeyCode::Right);
            }
            SettingsField::Theme => {
                self.toggle_settings_field(KeyCode::Right);
            }
            #[cfg(feature = "llm")]
            SettingsField::AiEnabled => {
                self.settings.ai_enabled = !self.settings.ai_enabled;
            }
            #[cfg(feature = "llm")]
            SettingsField::OllamaUrl => {
                self.settings.editing = true;
                self.settings.edit_buffer = self.settings.ollama_url.clone();
            }
            #[cfg(feature = "llm")]
            SettingsField::OllamaModel => {
                // Try to fetch models, fall back to text edit
                self.fetch_and_open_model_picker();
            }
        }
    }

    fn apply_settings_edit(&mut self) {
        let field = self.settings.focused_field().clone();
        let value = self.settings.edit_buffer.clone();
        match field {
            SettingsField::Editor => {
                self.settings.editor_value = value;
            }
            #[cfg(feature = "llm")]
            SettingsField::OllamaUrl => {
                self.settings.ollama_url = value;
            }
            #[cfg(feature = "llm")]
            SettingsField::OllamaModel => {
                self.settings.ollama_model = value;
            }
            _ => {}
        }
    }

    fn toggle_settings_field(&mut self, key: KeyCode) {
        let field = self.settings.focused_field().clone();
        match field {
            SettingsField::EditorType => {
                let types = ["auto", "terminal", "gui"];
                let current = types
                    .iter()
                    .position(|&t| t == self.settings.editor_type_value)
                    .unwrap_or(0);
                let next = match key {
                    KeyCode::Right => (current + 1) % types.len(),
                    KeyCode::Left => (current + types.len() - 1) % types.len(),
                    _ => current,
                };
                self.settings.editor_type_value = types[next].to_string();
            }
            SettingsField::SandboxLevel => {
                let levels = ["auto", "basic", "contained"];
                let current = levels
                    .iter()
                    .position(|&l| l == self.settings.sandbox_value)
                    .unwrap_or(0);
                let next = match key {
                    KeyCode::Right => (current + 1) % levels.len(),
                    KeyCode::Left => (current + levels.len() - 1) % levels.len(),
                    _ => current,
                };
                self.settings.sandbox_value = levels[next].to_string();
            }
            SettingsField::Theme => {
                let themes = ["default", "high-contrast"];
                let current = themes
                    .iter()
                    .position(|&t| t == self.settings.theme_value)
                    .unwrap_or(0);
                let next = match key {
                    KeyCode::Right => (current + 1) % themes.len(),
                    KeyCode::Left => (current + themes.len() - 1) % themes.len(),
                    _ => current,
                };
                self.settings.theme_value = themes[next].to_string();
            }
            #[cfg(feature = "llm")]
            SettingsField::AiEnabled => {
                self.settings.ai_enabled = !self.settings.ai_enabled;
            }
            _ => {}
        }
    }

    #[cfg(feature = "llm")]
    fn fetch_and_open_model_picker(&mut self) {
        let base_url = self.settings.ollama_url.trim_end_matches('/').to_string();

        // Quick sync fetch with short timeout
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => {
                // Fall back to text editing
                self.settings.editing = true;
                self.settings.edit_buffer = self.settings.ollama_model.clone();
                return;
            }
        };

        match rt.block_on(crate::llm::ollama::list_available_models(&base_url)) {
            Ok(models) if !models.is_empty() => {
                self.settings.available_models = models;
                self.settings.model_picker_idx = 0;
                self.settings.model_picker_open = true;
            }
            _ => {
                // Can't reach Ollama, fall back to text edit
                self.settings.editing = true;
                self.settings.edit_buffer = self.settings.ollama_model.clone();
            }
        }
    }

    fn save_settings(&mut self) {
        // Apply settings back to config
        self.config.editor = if self.settings.editor_value.is_empty() {
            None
        } else {
            Some(self.settings.editor_value.clone())
        };
        self.config.editor_type = match self.settings.editor_type_value.as_str() {
            "terminal" => EditorType::Terminal,
            "gui" => EditorType::Gui,
            _ => EditorType::Auto,
        };
        self.config.sandbox_level = match self.settings.sandbox_value.as_str() {
            "basic" => SandboxLevelPref::Basic,
            "contained" => SandboxLevelPref::Contained,
            _ => SandboxLevelPref::Auto,
        };
        self.config.theme = match self.settings.theme_value.as_str() {
            "high-contrast" => crate::config::ThemePreset::HighContrast,
            _ => crate::config::ThemePreset::Default,
        };
        // Live-update theme without restart
        self.theme = crate::ui::theme::Theme::new(&self.config.theme);

        #[cfg(feature = "llm")]
        {
            self.config.llm.enabled = self.settings.ai_enabled;
            self.config.llm.ollama.url = self.settings.ollama_url.clone();
            self.config.llm.ollama.model = self.settings.ollama_model.clone();
        }

        // Re-detect sandbox level
        self.sandbox_level = SandboxLevel::detect(&self.config.sandbox_level);

        let _ = self.config.save();
    }
}

/// Build display order: flat summaries indices grouped by language (BTreeMap order).
/// This ensures arrow-key navigation matches the visual grouping on screen.
/// Truncate key bar text to fit terminal width, adding "..." if truncated.
fn truncate_key_bar(text: &str, max_width: usize) -> String {
    if text.chars().count() <= max_width {
        text.to_string()
    } else if max_width > 3 {
        let truncated: String = text.chars().take(max_width - 3).collect();
        format!("{}...", truncated)
    } else {
        text.chars().take(max_width).collect()
    }
}

fn build_display_order(summaries: &[CourseProgressSummary]) -> Vec<usize> {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, s) in summaries.iter().enumerate() {
        groups
            .entry(s.info.language_name.clone())
            .or_default()
            .push(i);
    }
    groups.into_values().flatten().collect()
}

fn build_course_summaries(
    courses: &[CourseInfo],
    store: &ProgressStore,
) -> Vec<CourseProgressSummary> {
    courses
        .iter()
        .map(|info| {
            let course_id = info.name.to_lowercase().replace(' ', "-");
            let key = crate::state::types::progress_key(&course_id, &info.version);

            let (status, completed_lessons, completed_exercises) =
                if let Some(cp) = store.data.courses.get(&key) {
                    let cl = cp
                        .lessons
                        .values()
                        .filter(|lp| lp.status == ProgressStatus::Completed)
                        .count();
                    let ce: usize = cp
                        .lessons
                        .values()
                        .map(|lp| {
                            lp.exercises
                                .values()
                                .filter(|e| e.status == ProgressStatus::Completed)
                                .count()
                        })
                        .sum();

                    let status = if cl >= info.lesson_count && info.lesson_count > 0 {
                        CourseStatus::Completed
                    } else if ce > 0 || cl > 0 {
                        CourseStatus::InProgress
                    } else {
                        CourseStatus::NotStarted
                    };

                    (status, cl, ce)
                } else {
                    (CourseStatus::NotStarted, 0, 0)
                };

            let total_exercises = info.total_exercise_count.unwrap_or(info.lesson_count * 4);

            CourseProgressSummary {
                info: info.clone(),
                status,
                completed_lessons,
                total_lessons: info.lesson_count,
                completed_exercises,
                total_exercises,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::ui::screens::Screen;

    #[test]
    fn test_screen_enum() {
        assert_eq!(Screen::Home, Screen::Home);
        assert_ne!(Screen::Home, Screen::Settings);
        assert_ne!(Screen::Stats, Screen::Home);
        assert_eq!(Screen::Stats, Screen::Stats);
    }
}
