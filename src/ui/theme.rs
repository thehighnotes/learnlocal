use ratatui::style::Color;

use crate::config::ThemePreset;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Theme {
    // Markdown / content colors
    pub heading: Color,
    pub heading_h2: Color,
    pub heading_h3: Color,
    pub code: Color,
    pub code_border: Color,
    pub keyword: Color,
    pub string_lit: Color,
    pub comment: Color,
    pub error: Color,
    pub success: Color,
    pub prompt: Color,
    pub muted: Color,
    pub diff_expected: Color,
    pub diff_actual: Color,
    pub body_text: Color,
    pub table_border: Color,
    // UI chrome colors
    pub title_bar_fg: Color,
    pub title_bar_bg: Color,
    pub key_bar_fg: Color,
    pub key_bar_bg: Color,
    pub progress_filled: Color,
    pub progress_empty: Color,
    pub cursor: Color,
    pub warning: Color,
    pub border_active: Color,
    pub border_inactive: Color,
    pub no_color: bool,
}

impl Theme {
    pub fn new(preset: &ThemePreset) -> Self {
        if std::env::var("NO_COLOR").is_ok() {
            Self::no_color()
        } else {
            match preset {
                ThemePreset::Default => Self::default(),
                ThemePreset::HighContrast => Self::high_contrast(),
            }
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            heading: Color::LightCyan,
            heading_h2: Color::LightCyan,
            heading_h3: Color::LightYellow,
            code: Color::White,
            code_border: Color::White,
            keyword: Color::LightYellow,
            string_lit: Color::LightGreen,
            comment: Color::White,
            error: Color::LightRed,
            success: Color::LightGreen,
            prompt: Color::LightBlue,
            muted: Color::White,
            diff_expected: Color::LightRed,
            diff_actual: Color::LightYellow,
            body_text: Color::White,
            table_border: Color::White,
            title_bar_fg: Color::Black,
            title_bar_bg: Color::LightCyan,
            key_bar_fg: Color::Black,
            key_bar_bg: Color::White,
            progress_filled: Color::LightGreen,
            progress_empty: Color::White,
            cursor: Color::LightCyan,
            warning: Color::LightYellow,
            border_active: Color::White,
            border_inactive: Color::White,
            no_color: false,
        }
    }

    pub fn no_color() -> Self {
        Self {
            heading: Color::Reset,
            heading_h2: Color::Reset,
            heading_h3: Color::Reset,
            code: Color::Reset,
            code_border: Color::Reset,
            keyword: Color::Reset,
            string_lit: Color::Reset,
            comment: Color::Reset,
            error: Color::Reset,
            success: Color::Reset,
            prompt: Color::Reset,
            muted: Color::Reset,
            diff_expected: Color::Reset,
            diff_actual: Color::Reset,
            body_text: Color::Reset,
            table_border: Color::Reset,
            title_bar_fg: Color::Reset,
            title_bar_bg: Color::Reset,
            key_bar_fg: Color::Reset,
            key_bar_bg: Color::Reset,
            progress_filled: Color::Reset,
            progress_empty: Color::Reset,
            cursor: Color::Reset,
            warning: Color::Reset,
            border_active: Color::Reset,
            border_inactive: Color::Reset,
            no_color: true,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            heading: Color::Cyan,
            heading_h2: Color::Cyan,
            heading_h3: Color::Yellow,
            code: Color::Rgb(180, 220, 255),
            code_border: Color::DarkGray,
            keyword: Color::Yellow,
            string_lit: Color::Green,
            comment: Color::Gray,
            error: Color::Red,
            success: Color::Green,
            prompt: Color::Blue,
            muted: Color::DarkGray,
            diff_expected: Color::Red,
            diff_actual: Color::Yellow,
            body_text: Color::Rgb(200, 200, 200),
            table_border: Color::DarkGray,
            title_bar_fg: Color::Black,
            title_bar_bg: Color::Cyan,
            key_bar_fg: Color::Black,
            key_bar_bg: Color::White,
            progress_filled: Color::Green,
            progress_empty: Color::DarkGray,
            cursor: Color::Cyan,
            warning: Color::Yellow,
            border_active: Color::Cyan,
            border_inactive: Color::DarkGray,
            no_color: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_contrast_body_text_is_white() {
        let theme = Theme::high_contrast();
        assert_eq!(theme.body_text, Color::White);
        assert_eq!(theme.border_active, Color::White);
        assert!(!theme.no_color);
    }

    #[test]
    fn test_no_color_overrides_preset() {
        let theme = Theme::no_color();
        assert_eq!(theme.body_text, Color::Reset);
        assert_eq!(theme.cursor, Color::Reset);
        assert!(theme.no_color);
    }

    #[test]
    fn test_default_differs_from_no_color() {
        let def = Theme::default();
        let nc = Theme::no_color();
        assert_ne!(def.heading, nc.heading);
        assert_ne!(def.body_text, nc.body_text);
    }

    #[test]
    fn test_new_respects_preset() {
        // Can't easily test NO_COLOR env var, but we can test preset dispatch
        std::env::remove_var("NO_COLOR");
        let default_theme = Theme::new(&ThemePreset::Default);
        assert_eq!(default_theme.heading, Color::Cyan);
        let hc_theme = Theme::new(&ThemePreset::HighContrast);
        assert_eq!(hc_theme.heading, Color::LightCyan);
    }
}
