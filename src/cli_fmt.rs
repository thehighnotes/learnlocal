//! ANSI color helpers for CLI (non-TUI) output. Respects NO_COLOR.

fn no_color() -> bool {
    std::env::var("NO_COLOR").is_ok()
}

pub fn green(s: &str) -> String {
    if no_color() {
        s.to_string()
    } else {
        format!("\x1b[32m{}\x1b[0m", s)
    }
}

pub fn red(s: &str) -> String {
    if no_color() {
        s.to_string()
    } else {
        format!("\x1b[31m{}\x1b[0m", s)
    }
}

pub fn yellow(s: &str) -> String {
    if no_color() {
        s.to_string()
    } else {
        format!("\x1b[33m{}\x1b[0m", s)
    }
}

pub fn dim(s: &str) -> String {
    if no_color() {
        s.to_string()
    } else {
        format!("\x1b[2m{}\x1b[0m", s)
    }
}

pub fn bold(s: &str) -> String {
    if no_color() {
        s.to_string()
    } else {
        format!("\x1b[1m{}\x1b[0m", s)
    }
}
