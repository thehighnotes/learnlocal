use crate::course::types::{Course, CourseInfo};
use crate::exec::toolcheck::{ToolStatus, PlatformStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    HowTo,
    Settings,
    Progress,
    Stats,
    Tour,
    Course,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HomePanelFocus {
    CourseList,
    LessonList,
}

/// What the CourseApp wants the outer shell to do after handling input.
pub enum CourseAction {
    Continue,
    Quit,
    GoHome,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CourseStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// Summary info for displaying a course on the Home screen.
#[allow(dead_code)]
pub struct CourseProgressSummary {
    pub info: CourseInfo,
    pub status: CourseStatus,
    pub completed_lessons: usize,
    pub total_lessons: usize,
    pub completed_exercises: usize,
    pub total_exercises: usize,
}

pub struct HowToState {
    pub slide_index: usize,
}

impl HowToState {
    pub fn new() -> Self {
        Self { slide_index: 0 }
    }
}

pub struct StatsState {
    pub scroll_offset: u16,
    pub content_height: u16,
}

impl StatsState {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            content_height: 0,
        }
    }
}

pub struct TourState {
    pub slide_index: usize,
}

impl TourState {
    pub fn new() -> Self {
        Self { slide_index: 0 }
    }
}

pub struct HomeState {
    /// Visual cursor position (indexes into display_order)
    pub selected_idx: usize,
    pub summaries: Vec<CourseProgressSummary>,
    /// Maps visual position → flat summaries index (matches BTreeMap language grouping)
    pub display_order: Vec<usize>,
    /// Cached tool check results, indexed by course position. None = not yet checked.
    pub tool_check_cache: Vec<Option<Vec<ToolStatus>>>,
    /// Cached platform check results, indexed by course position. None = not yet checked.
    pub platform_check_cache: Vec<Option<PlatformStatus>>,
    /// Which panel has input focus
    pub focus: HomePanelFocus,
    /// Lesson cursor in right panel
    pub right_selected_idx: usize,
}

impl HomeState {
    pub fn new() -> Self {
        Self {
            selected_idx: 0,
            summaries: Vec::new(),
            display_order: Vec::new(),
            tool_check_cache: Vec::new(),
            platform_check_cache: Vec::new(),
            focus: HomePanelFocus::CourseList,
            right_selected_idx: 0,
        }
    }

    /// Resolve visual cursor position to the flat summaries index.
    pub fn flat_idx(&self) -> usize {
        self.display_order.get(self.selected_idx).copied().unwrap_or(self.selected_idx)
    }

    /// Check if a course (by flat summaries index) can be started.
    /// Blocked if required tools are missing or platform doesn't match.
    pub fn is_course_startable(&self, flat_idx: usize) -> bool {
        let tools_ok = self.tool_check_cache.get(flat_idx)
            .and_then(|c| c.as_ref())
            .map(|statuses| statuses.iter().all(|s| s.found))
            .unwrap_or(true);
        let platform_ok = self.platform_check_cache.get(flat_idx)
            .and_then(|c| c.as_ref())
            .map(|ps| ps.supported)
            .unwrap_or(true);
        tools_ok && platform_ok
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsField {
    Editor,
    EditorType,
    SandboxLevel,
    #[cfg(feature = "llm")]
    AiEnabled,
    #[cfg(feature = "llm")]
    OllamaUrl,
    #[cfg(feature = "llm")]
    OllamaModel,
}

impl SettingsField {
    pub fn all() -> Vec<SettingsField> {
        #[allow(unused_mut)]
        let mut fields = vec![
            SettingsField::Editor,
            SettingsField::EditorType,
            SettingsField::SandboxLevel,
        ];
        #[cfg(feature = "llm")]
        {
            fields.push(SettingsField::AiEnabled);
            fields.push(SettingsField::OllamaUrl);
            fields.push(SettingsField::OllamaModel);
        }
        fields
    }
}

pub struct SettingsState {
    pub focused_idx: usize,
    pub fields: Vec<SettingsField>,
    pub editing: bool,
    pub edit_buffer: String,
    // Cached values for display
    pub editor_value: String,
    pub editor_type_value: String,
    pub sandbox_value: String,
    #[cfg(feature = "llm")]
    pub ai_enabled: bool,
    #[cfg(feature = "llm")]
    pub ollama_url: String,
    #[cfg(feature = "llm")]
    pub ollama_model: String,
    #[cfg(feature = "llm")]
    pub available_models: Vec<String>,
    #[cfg(feature = "llm")]
    pub model_picker_open: bool,
    #[cfg(feature = "llm")]
    pub model_picker_idx: usize,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            focused_idx: 0,
            fields: SettingsField::all(),
            editing: false,
            edit_buffer: String::new(),
            editor_value: String::new(),
            editor_type_value: "auto".to_string(),
            sandbox_value: "auto".to_string(),
            #[cfg(feature = "llm")]
            ai_enabled: false,
            #[cfg(feature = "llm")]
            ollama_url: "http://localhost:11434".to_string(),
            #[cfg(feature = "llm")]
            ollama_model: "qwen3:4b".to_string(),
            #[cfg(feature = "llm")]
            available_models: Vec::new(),
            #[cfg(feature = "llm")]
            model_picker_open: false,
            #[cfg(feature = "llm")]
            model_picker_idx: 0,
        }
    }

    pub fn focused_field(&self) -> &SettingsField {
        &self.fields[self.focused_idx]
    }
}

pub struct ProgressViewState {
    pub course_idx: usize,
    pub course: Option<Course>,
    pub selected_lesson_idx: usize,
    pub confirm_reset: bool,
}

impl ProgressViewState {
    pub fn new() -> Self {
        Self {
            course_idx: 0,
            course: None,
            selected_lesson_idx: 0,
            confirm_reset: false,
        }
    }
}
