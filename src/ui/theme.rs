use ratatui::style::Color;

#[derive(Clone)]
#[allow(dead_code)]
pub struct Theme {
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
    pub no_color: bool,
}

impl Theme {
    pub fn new() -> Self {
        if std::env::var("NO_COLOR").is_ok() {
            Self::no_color()
        } else {
            Self::default()
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
            no_color: false,
        }
    }
}
