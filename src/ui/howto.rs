use ratatui::style::Color;
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;
use crate::ui::tour::{
    blank, body, bold, bot, dbot, drow, dtop, dual, dual_bot, dual_top, heading, muted, row,
    separator, styled, top, FULL,
};

pub const SLIDE_COUNT: usize = 7;

pub struct HowToCtx {
    pub config_path: String,
    pub progress_path: String,
    pub sandbox_path: String,
    pub courses_path: String,
}

pub fn build_slide(index: usize, theme: &Theme, ctx: &HowToCtx) -> Vec<Line<'static>> {
    match index {
        0 => slide_overview(theme),
        1 => slide_editing(theme),
        2 => slide_running(theme),
        3 => slide_reading(theme),
        4 => slide_help(theme),
        5 => slide_files(theme, ctx),
        6 => slide_keys(theme),
        _ => vec![],
    }
}

// ─────────────────────────── Slide 0: Overview ───────────────────────────

fn slide_overview(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;

    let mut l = Vec::new();
    l.push(blank());
    l.push(blank());

    // Title banner
    l.push(dtop(FULL, cyan));
    l.push(drow(cyan, vec![], FULL));
    l.push(drow(
        cyan,
        vec![
            Span::raw("            ".to_string()),
            bold("H O W   T O   U S E", Color::White),
            bold("   ", cyan),
            bold("L E A R N L O C A L", green),
        ],
        FULL,
    ));
    l.push(drow(
        cyan,
        vec![
            Span::raw("            ".to_string()),
            body("Quick Reference Guide", theme),
        ],
        FULL,
    ));
    l.push(drow(cyan, vec![], FULL));
    l.push(dbot(FULL, cyan));
    l.push(blank());

    // Compact cycle box
    l.push(top("The Exercise Cycle", FULL, yellow));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![
            body("  ", theme),
            styled("[Space]", cyan),
            body(" READ  ", theme),
            muted("→  ", theme),
            styled("[e]", cyan),
            body(" EDIT      ", theme),
            muted("→  ", theme),
            styled("[Enter]", cyan),
            body(" RUN  ", theme),
            muted("→  ", theme),
            styled("[t]", cyan),
            body(" SUBMIT", theme),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![
            body("  Run freely with ", theme),
            styled("[Enter]", cyan),
            body(" — only ", theme),
            styled("[t]", green),
            body(" submit counts toward progress.", theme),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            body("  Stuck?  ", theme),
            styled("[h]", yellow),
            body(" hints  →  ", theme),
            styled("[s]", yellow),
            body(" skip and return later", theme),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(bot(FULL, yellow));
    l.push(blank());

    // Table of contents
    l.push(heading("In this guide:", theme));
    l.push(blank());
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[2]", cyan),
        body(" Editing Code      ", theme),
        muted("— the built-in inline editor", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[3]", cyan),
        body(" Running & Testing ", theme),
        muted("— run vs submit, feedback types", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[4]", cyan),
        body(" Reading Lessons   ", theme),
        muted("— progressive reveal, section focus", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[5]", cyan),
        body(" Getting Help      ", theme),
        muted("— hints, skipping, optional AI", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[6]", cyan),
        body(" Files & Settings  ", theme),
        muted("— paths, configuration, sandboxes", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[7]", cyan),
        body(" Key Reference     ", theme),
        muted("— complete keyboard cheat sheet", theme),
    ]));
    l.push(blank());
    l.push(Line::from(vec![
        muted("   Press ", theme),
        styled("→", cyan),
        muted(" or ", theme),
        styled("Enter", cyan),
        muted(" to navigate. Jump to any page with ", theme),
        styled("1-7", cyan),
        muted(".", theme),
    ]));

    l
}

// ─────────────────────────── Slide 1: Editing ────────────────────────────

fn slide_editing(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;
    let blue = Color::Rgb(180, 220, 255);

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("EDITING YOUR CODE", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Press [e] to open the built-in inline editor — edit right inside the TUI:",
        theme,
    )));
    l.push(blank());

    // Inline editor — full width box with mock code
    l.push(top("[e] Inline Editor", FULL, green));
    l.push(row(green, vec![], FULL));
    l.push(row(
        green,
        vec![
            styled("  1 │ ", muted_c),
            styled("#include <iostream>", blue),
        ],
        FULL,
    ));
    l.push(row(
        green,
        vec![
            styled("  2 │ ", muted_c),
            styled("using namespace std;", blue),
        ],
        FULL,
    ));
    l.push(row(green, vec![styled("  3 │ ", muted_c)], FULL));
    l.push(row(
        green,
        vec![styled("  4 │ ", muted_c), styled("int main() {", blue)],
        FULL,
    ));
    l.push(row(
        green,
        vec![
            styled("  5 │ ", muted_c),
            styled("    cout << \"Hello!\" << endl;", blue),
        ],
        FULL,
    ));
    l.push(row(
        green,
        vec![styled("  6 │ ", muted_c), styled("    return 0;", blue)],
        FULL,
    ));
    l.push(row(
        green,
        vec![styled("  7 │ ", muted_c), styled("}", blue)],
        FULL,
    ));
    l.push(row(green, vec![], FULL));
    l.push(bot(FULL, green));
    l.push(blank());

    // Editor controls — side by side
    l.push(dual_top("Controls", cyan, "Tips", yellow));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual(
        cyan,
        vec![styled("  Arrows", green), body("  Move cursor", theme)],
        yellow,
        vec![styled("  →", green), body(" Line numbers shown", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Type  ", green), body("  Insert text", theme)],
        yellow,
        vec![styled("  →", green), body(" Auto-scrolls to cursor", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Bksp  ", green), body("  Delete backward", theme)],
        yellow,
        vec![styled("  →", green), body(" Code box replaces the", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Del   ", green), body("  Delete forward", theme)],
        yellow,
        vec![body("    lesson content while open", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Enter ", green), body("  New line", theme)],
        yellow,
        vec![],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Tab   ", green), body("  Insert spaces", theme)],
        yellow,
        vec![styled("  →", green), body(" Esc saves and runs —", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Home  ", green), body("  Line start", theme)],
        yellow,
        vec![body("    see output immediately", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  End   ", green), body("  Line end", theme)],
        yellow,
        vec![],
    ));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual(
        cyan,
        vec![styled("  Esc   ", green), body("  Save & close", theme)],
        yellow,
        vec![
            styled("  →", green),
            body(" Draft saved if you quit", theme),
        ],
    ));
    l.push(dual(
        cyan,
        vec![styled("  Ctrl+S", green), body(" Save (stay open)", theme)],
        yellow,
        vec![body("    before finishing", theme)],
    ));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual_bot(cyan, yellow));

    l
}

// ─────────────────────────── Slide 2: Running ────────────────────────────

fn slide_running(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let red = theme.error;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("RUNNING & TESTING", theme));
    l.push(separator(theme));
    l.push(blank());

    // Side by side: Run vs Submit
    l.push(dual_top("[Enter] Run", cyan, "[t] Submit", green));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual(
        cyan,
        vec![body("  Executes your code and", theme)],
        green,
        vec![body("  Validates your solution", theme)],
    ));
    l.push(dual(
        cyan,
        vec![body("  shows the output.", theme)],
        green,
        vec![body("  against expected output.", theme)],
    ));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual(
        cyan,
        vec![styled("  No grading", yellow), body(" — experiment", theme)],
        green,
        vec![styled("  ✓ Pass", green), body(" → next exercise", theme)],
    ));
    l.push(dual(
        cyan,
        vec![body("  freely, as often as you", theme)],
        green,
        vec![styled("  ✘ Fail", red), body(" → detailed feedback", theme)],
    ));
    l.push(dual(cyan, vec![body("  want.", theme)], green, vec![]));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual(
        cyan,
        vec![body("  Use this to debug and", theme)],
        green,
        vec![body("  Only submit when you're", theme)],
    ));
    l.push(dual(
        cyan,
        vec![body("  develop your solution.", theme)],
        green,
        vec![body("  confident it's correct.", theme)],
    ));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual_bot(cyan, green));
    l.push(blank());

    // Feedback types — full width
    l.push(top("What You See When Code Fails", FULL, yellow));
    l.push(row(yellow, vec![], FULL));
    l.push(row(yellow, vec![bold("  Output Diff", cyan)], FULL));
    l.push(row(
        yellow,
        vec![
            body("  Color-coded comparison:  ", theme),
            styled("Expected: ", muted_c),
            styled("Hello, World!", green),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            body("                           ", theme),
            styled("Got:      ", muted_c),
            styled("hello world", red),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![bold("  Compiler Diagnostics", cyan)],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            body("  Parsed errors with file/line info:  ", theme),
            styled("error", red),
            body(" → main.rs:3:12", theme),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![
            bold("  State Assertions", cyan),
            body("  (environment exercises)", theme),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            body("  ", theme),
            styled("✓", green),
            body(" File exists  ", theme),
            styled("✓", green),
            body(" Content matches  ", theme),
            styled("✓", green),
            body(" Permissions correct  ", theme),
            styled("✘", red),
            body(" Missing", theme),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(bot(FULL, yellow));
    l.push(blank());

    // Shell mode — full width
    l.push(top("[COMMAND] Shell Mode", FULL, Color::Blue));
    l.push(row(Color::Blue, vec![], FULL));
    l.push(row(
        Color::Blue,
        vec![body(
            "  Command exercises open a terminal instead of the code editor.",
            theme,
        )],
        FULL,
    ));
    l.push(row(
        Color::Blue,
        vec![
            body("  Type shell commands at the ", theme),
            styled("$ ", green),
            body("prompt — ", theme),
            styled("[Enter]", cyan),
            body(" runs and validates.", theme),
        ],
        FULL,
    ));
    l.push(row(Color::Blue, vec![], FULL));
    l.push(row(
        Color::Blue,
        vec![
            styled("  [Enter]", cyan),
            body("   Execute command       ", theme),
            styled("[↑/↓]", cyan),
            body("     Command history", theme),
        ],
        FULL,
    ));
    l.push(row(
        Color::Blue,
        vec![
            styled("  [Ctrl+H]", cyan),
            body("  Reveal hint            ", theme),
            styled("[Ctrl+C]", cyan),
            body("   Clear input", theme),
        ],
        FULL,
    ));
    l.push(row(
        Color::Blue,
        vec![
            styled("  [F1]", cyan),
            body("      Help overlay           ", theme),
            styled("[Esc]", cyan),
            body("      Exit shell mode", theme),
        ],
        FULL,
    ));
    l.push(row(Color::Blue, vec![], FULL));
    l.push(bot(FULL, Color::Blue));
    l.push(blank());
    l.push(Line::from(vec![
        body("   Every error shows ", theme),
        styled("why", green),
        body(
            " — not just \"wrong\". Use feedback to learn and iterate.",
            theme,
        ),
    ]));

    l
}

// ─────────────────────────── Slide 3: Reading ────────────────────────────

fn slide_reading(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("READING LESSONS", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Lesson content is revealed one section at a time — absorb, then advance:",
        theme,
    )));
    l.push(blank());

    // ASCII art showing progressive reveal
    l.push(top("Progressive Reveal", FULL, cyan));
    l.push(row(cyan, vec![], FULL));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            bold("▐", green),
            styled(" Section 1: Introduction to Variables", green),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            bold("▐", green),
            styled(
                " Variables store data. In C++, you declare them with a type...",
                green,
            ),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![body("    ", theme), styled("▐", muted_c)],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            styled("▐", muted_c),
            styled(" Section 2: Data Types", muted_c),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            styled("▐", muted_c),
            styled(
                " C++ has several built-in types: int, double, char, bool...",
                muted_c,
            ),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![body("    ", theme), styled("▐", muted_c)],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            styled("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░", muted_c),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            styled(
                "░  Section 3: hidden — press [Space] to reveal   ░",
                muted_c,
            ),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body("    ", theme),
            styled("░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░", muted_c),
        ],
        FULL,
    ));
    l.push(row(cyan, vec![], FULL));
    l.push(bot(FULL, cyan));
    l.push(blank());

    // Controls
    l.push(dual_top("Reading Controls", green, "How It Works", yellow));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual(
        green,
        vec![
            styled("  [Space]", cyan),
            body("  Reveal next section", theme),
        ],
        yellow,
        vec![
            styled("  →", green),
            body(" Focused section is bright", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![
            styled("  [↑/↓]  ", cyan),
            body("  Focus between sections", theme),
        ],
        yellow,
        vec![
            styled("  →", green),
            body(" Earlier sections are dimmed", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![styled("  [PgUp] ", cyan), body("  Scroll up a page", theme)],
        yellow,
        vec![
            styled("  →", green),
            body(" Hidden sections show as", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![
            styled("  [PgDn] ", cyan),
            body("  Scroll down a page", theme),
        ],
        yellow,
        vec![body("    blocked until revealed", theme)],
    ));
    l.push(dual(
        green,
        vec![styled("  [Home] ", cyan), body("  Jump to top", theme)],
        yellow,
        vec![],
    ));
    l.push(dual(
        green,
        vec![styled("  [End]  ", cyan), body("  Jump to bottom", theme)],
        yellow,
        vec![styled("  →", green), body(" Read at your own pace!", theme)],
    ));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual_bot(green, yellow));

    l
}

// ─────────────────────────── Slide 4: Help ───────────────────────────────

fn slide_help(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("GETTING HELP", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   When you're stuck, LearnLocal offers a graduated support path:",
        theme,
    )));
    l.push(blank());

    // Escalation path as a flow — two boxes
    let pw = 27; // box inner width

    l.push(Line::from(vec![
        Span::raw("            ".to_string()),
        bold(&format!("┌{}┐", "─".repeat(pw)), yellow),
        Span::raw("             ".to_string()),
        bold(&format!("┌{}┐", "─".repeat(pw)), green),
    ]));
    l.push(Line::from(vec![
        Span::raw("            ".to_string()),
        bold("│", yellow),
        styled(&format!("{:<w$}", " [h] HINTS", w = pw), yellow),
        bold("│", yellow),
        body("  ────────→  ", theme),
        bold("│", green),
        styled(&format!("{:<w$}", " [s] SKIP", w = pw), green),
        bold("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("            ".to_string()),
        bold("│", yellow),
        body(
            &format!("{:<w$}", " Reveals one hint at a time", w = pw),
            theme,
        ),
        bold("│", yellow),
        Span::raw("             ".to_string()),
        bold("│", green),
        body(&format!("{:<w$}", " Come back to it later", w = pw), theme),
        bold("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("            ".to_string()),
        bold(&format!("└{}┘", "─".repeat(pw)), yellow),
        Span::raw("             ".to_string()),
        bold(&format!("└{}┘", "─".repeat(pw)), green),
    ]));
    l.push(blank());

    // Details — full width boxes
    l.push(top("[h] Progressive Hints", FULL, yellow));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![
            body("  Each press of ", theme),
            styled("[h]", cyan),
            body(
                " reveals one more hint. Try to solve with fewer hints.",
                theme,
            ),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![body(
            "  Hints range from gentle nudges to near-solutions.",
            theme,
        )],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(bot(FULL, yellow));

    l.push(blank());
    l.push(top("[a] AI Tutor (Optional)", FULL, cyan));
    l.push(row(cyan, vec![], FULL));
    l.push(row(
        cyan,
        vec![body(
            "  Chat with an AI that sees your lesson, code, errors, and hints.",
            theme,
        )],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![body(
            "  Guides you toward the answer without spoiling it.",
            theme,
        )],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            body(
                "  Runs on your machine — no data sent anywhere. Enable in ",
                theme,
            ),
            styled("Settings [s]", green),
            body(".", theme),
        ],
        FULL,
    ));
    l.push(row(cyan, vec![], FULL));
    l.push(bot(FULL, cyan));

    l.push(blank());
    l.push(top("Other Help", FULL, green));
    l.push(row(green, vec![], FULL));
    l.push(row(
        green,
        vec![
            styled("  [?]", cyan),
            body("  Quick keyboard shortcut overlay (inside a course)", theme),
        ],
        FULL,
    ));
    l.push(row(
        green,
        vec![
            styled("  [r]", cyan),
            body(
                "  Reset exercise to starter code if you want a fresh start",
                theme,
            ),
        ],
        FULL,
    ));
    l.push(row(green, vec![], FULL));
    l.push(bot(FULL, green));

    l
}

// ─────────────────────────── Slide 5: Files ──────────────────────────────

fn slide_files(theme: &Theme, ctx: &HowToCtx) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("FILES & SETTINGS", theme));
    l.push(separator(theme));
    l.push(blank());

    // File paths — full width
    l.push(top("Where Everything Lives", FULL, cyan));
    l.push(row(cyan, vec![], FULL));
    l.push(row(
        cyan,
        vec![
            styled("  Config      ", yellow),
            body(&ctx.config_path, theme),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            styled("  Progress    ", yellow),
            body(&ctx.progress_path, theme),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            styled("  Sandboxes   ", yellow),
            body(&ctx.sandbox_path, theme),
        ],
        FULL,
    ));
    l.push(row(
        cyan,
        vec![
            styled("  Courses     ", yellow),
            body(&ctx.courses_path, theme),
        ],
        FULL,
    ));
    l.push(row(cyan, vec![], FULL));
    l.push(bot(FULL, cyan));
    l.push(blank());

    // Side by side: Progress / Drafts & Sandboxes
    l.push(dual_top(
        "Progress Tracking",
        green,
        "Drafts & Sandboxes",
        yellow,
    ));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual(
        green,
        vec![styled("  →", green), body(" Saved automatically", theme)],
        yellow,
        vec![
            styled("  →", green),
            body(" Draft code saved on exit", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![styled("  →", green), body(" Pick up where you left", theme)],
        yellow,
        vec![
            styled("  →", green),
            body(" Restored when you return", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![body("    off, any time", theme)],
        yellow,
        vec![styled("  →", green), body(" Cleared on completion", theme)],
    ));
    l.push(dual(
        green,
        vec![
            styled("  →", green),
            body(" Keyed to course + major", theme),
        ],
        yellow,
        vec![],
    ));
    l.push(dual(
        green,
        vec![body("    version (survives patches)", theme)],
        yellow,
        vec![bold("  Lesson Sandboxes:", yellow)],
    ));
    l.push(dual(
        green,
        vec![],
        yellow,
        vec![
            styled("  →", green),
            body(" Free playground per lesson", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![body("  View progress:", theme)],
        yellow,
        vec![
            styled("  →", green),
            body(" No grading, just explore", theme),
        ],
    ));
    l.push(dual(
        green,
        vec![styled("  [p]", cyan), body(" Progress details", theme)],
        yellow,
        vec![styled("  →", green), body(" Code persists between", theme)],
    ));
    l.push(dual(
        green,
        vec![styled("  [t]", cyan), body(" Stats overview", theme)],
        yellow,
        vec![body("    sessions", theme)],
    ));
    l.push(dual(
        green,
        vec![styled("  [r]", cyan), body(" Reset (on Progress)", theme)],
        yellow,
        vec![],
    ));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual_bot(green, yellow));
    l.push(blank());

    // Settings
    l.push(top("Settings", FULL, muted_c));
    l.push(row(muted_c, vec![], FULL));
    l.push(row(
        muted_c,
        vec![
            body("  Press ", theme),
            styled("[s]", cyan),
            body(" from the home screen to configure:", theme),
        ],
        FULL,
    ));
    l.push(row(
        muted_c,
        vec![
            body("  editor preference, sandbox level", theme),
            #[cfg(feature = "llm")]
            body(", AI model and Ollama URL", theme),
            #[cfg(not(feature = "llm"))]
            body("", theme),
        ],
        FULL,
    ));
    l.push(row(muted_c, vec![], FULL));
    l.push(bot(FULL, muted_c));

    l
}

// ─────────────────────────── Slide 6: Keys ───────────────────────────────

fn slide_keys(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("KEY REFERENCE", theme));
    l.push(separator(theme));
    l.push(blank());

    // Dual boxes: Home / Course
    l.push(dual_top("Home Screen", cyan, "Inside a Course", green));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual(
        cyan,
        vec![styled("  [Enter]", cyan), body("  Start course", theme)],
        green,
        vec![styled("  [e]    ", cyan), body("  Inline editor", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [→]    ", cyan), body("  Browse lessons", theme)],
        green,
        vec![styled("  [Enter]", cyan), body("  Run (no grading)", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [w]    ", cyan), body("  Welcome tour", theme)],
        green,
        vec![
            styled("  [t]    ", cyan),
            body("  Submit for grading", theme),
        ],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [h]    ", cyan), body("  How To (this!)", theme)],
        green,
        vec![
            styled("  [a]    ", cyan),
            body("  AI chat (optional)", theme),
        ],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [t]    ", cyan), body("  Stats overview", theme)],
        green,
        vec![styled("  [h]    ", cyan), body("  Reveal hint", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [p]    ", cyan), body("  Progress details", theme)],
        green,
        vec![styled("  [s]    ", cyan), body("  Skip exercise", theme)],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [s]    ", cyan), body("  Settings", theme)],
        green,
        vec![
            styled("  [Space]", cyan),
            body("  Reveal next section", theme),
        ],
    ));
    l.push(dual(
        cyan,
        vec![styled("  [q]    ", cyan), body("  Quit", theme)],
        green,
        vec![
            styled("  [r]    ", cyan),
            body("  Reset starter code", theme),
        ],
    ));
    l.push(dual(
        cyan,
        vec![],
        green,
        vec![styled("  [?]    ", cyan), body("  Help overlay", theme)],
    ));
    l.push(dual(
        cyan,
        vec![],
        green,
        vec![styled("  [←/→]  ", cyan), body("  Navigate lessons", theme)],
    ));
    l.push(dual(
        cyan,
        vec![],
        green,
        vec![styled("  [Esc]  ", cyan), body("  Return home", theme)],
    ));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual_bot(cyan, green));
    l.push(blank());

    // Inline editor keys
    l.push(top("Inside the Inline Editor [e]", FULL, yellow));
    l.push(row(yellow, vec![], FULL));
    l.push(row(
        yellow,
        vec![
            styled("  Arrows", cyan),
            body("  Move cursor          ", theme),
            styled("Esc", cyan),
            body("       Save & close", theme),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            styled("  Type  ", cyan),
            body("  Insert text          ", theme),
            styled("Ctrl+S", cyan),
            body("    Save (stay open)", theme),
        ],
        FULL,
    ));
    l.push(row(
        yellow,
        vec![
            styled("  Bksp  ", cyan),
            body("  Delete backward      ", theme),
            styled("Home/End", cyan),
            body("  Line start/end", theme),
        ],
        FULL,
    ));
    l.push(row(yellow, vec![], FULL));
    l.push(bot(FULL, yellow));
    l.push(blank());

    // Shell mode keys
    l.push(top("Shell Mode [COMMAND exercises]", FULL, Color::Blue));
    l.push(row(Color::Blue, vec![], FULL));
    l.push(row(
        Color::Blue,
        vec![
            styled("  Enter ", cyan),
            body("  Run command          ", theme),
            styled("↑/↓", cyan),
            body("       Command history", theme),
        ],
        FULL,
    ));
    l.push(row(
        Color::Blue,
        vec![
            styled("  Ctrl+H", cyan),
            body(" Reveal hint           ", theme),
            styled("Ctrl+C", cyan),
            body("    Clear input", theme),
        ],
        FULL,
    ));
    l.push(row(
        Color::Blue,
        vec![
            styled("  F1    ", cyan),
            body("  Help overlay         ", theme),
            styled("Esc", cyan),
            body("       Exit shell mode", theme),
        ],
        FULL,
    ));
    l.push(row(Color::Blue, vec![], FULL));
    l.push(bot(FULL, Color::Blue));

    l
}
