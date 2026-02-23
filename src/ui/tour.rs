use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;

pub const SLIDE_COUNT: usize = 9;

// Box inner widths (chars between left and right border characters)
pub(crate) const FULL: usize = 73; // full-width box: "   │" + 73 + "│" = 78 total
pub(crate) const SIDE: usize = 35; // side-by-side:   "   │" + 35 + "│  │" + 35 + "│" = 79 total

pub fn build_slide(index: usize, theme: &Theme, courses: &[String]) -> Vec<Line<'static>> {
    match index {
        0 => slide_welcome(theme),
        1 => slide_core_loop(theme),
        2 => slide_exercise_types(theme),
        3 => slide_editor(theme),
        4 => slide_feedback(theme),
        5 => slide_environment(theme),
        6 => slide_ai_tutor(theme),
        7 => slide_progress_combined(theme),
        8 => slide_get_started(theme, courses),
        _ => vec![],
    }
}

// ─── Text helpers ───────────────────────────────────────────────────────

pub(crate) fn blank() -> Line<'static> {
    Line::from("")
}

pub(crate) fn styled(text: &str, color: Color) -> Span<'static> {
    Span::styled(text.to_string(), Style::default().fg(color))
}

pub(crate) fn bold(text: &str, color: Color) -> Span<'static> {
    Span::styled(
        text.to_string(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )
}

pub(crate) fn muted(text: &str, theme: &Theme) -> Span<'static> {
    styled(text, theme.muted)
}

pub(crate) fn body(text: &str, theme: &Theme) -> Span<'static> {
    styled(text, theme.body_text)
}

pub(crate) fn heading(title: &str, theme: &Theme) -> Line<'static> {
    Line::from(bold(&format!("   {}", title), theme.heading))
}

pub(crate) fn separator(theme: &Theme) -> Line<'static> {
    Line::from(muted(&format!("   {}", "═".repeat(75)), theme))
}

// ─── Box helpers (auto-pad to exact inner width) ────────────────────────

/// Full-width box row: "   │{content padded to w}│"
pub(crate) fn row(bc: Color, content: Vec<Span<'static>>, w: usize) -> Line<'static> {
    let used: usize = content.iter().map(|s| s.content.chars().count()).sum();
    let pad = w.saturating_sub(used);
    let mut v = Vec::with_capacity(content.len() + 4);
    v.push(Span::raw("   ".to_string()));
    v.push(styled("│", bc));
    v.extend(content);
    if pad > 0 {
        v.push(Span::raw(" ".repeat(pad)));
    }
    v.push(styled("│", bc));
    Line::from(v)
}

/// Full-width box top: "   ┌─ title ──...──┐" or "   ┌──...──┐"
pub(crate) fn top(title: &str, w: usize, c: Color) -> Line<'static> {
    let s = if title.is_empty() {
        format!("   ┌{}┐", "─".repeat(w))
    } else {
        let rest = w.saturating_sub(title.chars().count() + 3);
        format!("   ┌─ {} {}┐", title, "─".repeat(rest))
    };
    Line::from(bold(&s, c))
}

/// Full-width box bottom: "   └──...──┘"
pub(crate) fn bot(w: usize, c: Color) -> Line<'static> {
    Line::from(bold(&format!("   └{}┘", "─".repeat(w)), c))
}

/// Double-line box row: "   ║{content padded to w}║"
pub(crate) fn drow(bc: Color, content: Vec<Span<'static>>, w: usize) -> Line<'static> {
    let used: usize = content.iter().map(|s| s.content.chars().count()).sum();
    let pad = w.saturating_sub(used);
    let mut v = Vec::with_capacity(content.len() + 4);
    v.push(Span::raw("   ".to_string()));
    v.push(bold("║", bc));
    v.extend(content);
    if pad > 0 {
        v.push(Span::raw(" ".repeat(pad)));
    }
    v.push(bold("║", bc));
    Line::from(v)
}

/// Double-line box top/bottom
pub(crate) fn dtop(w: usize, c: Color) -> Line<'static> {
    Line::from(bold(&format!("   ╔{}╗", "═".repeat(w)), c))
}
pub(crate) fn dbot(w: usize, c: Color) -> Line<'static> {
    Line::from(bold(&format!("   ╚{}╝", "═".repeat(w)), c))
}

/// Dual box row: "   │left..pad│  │right..pad│" (both SIDE width)
pub(crate) fn dual(lc: Color, left: Vec<Span<'static>>, rc: Color, right: Vec<Span<'static>>) -> Line<'static> {
    let lu: usize = left.iter().map(|s| s.content.chars().count()).sum();
    let lp = SIDE.saturating_sub(lu);
    let ru: usize = right.iter().map(|s| s.content.chars().count()).sum();
    let rp = SIDE.saturating_sub(ru);
    let mut v = Vec::new();
    v.push(Span::raw("   ".to_string()));
    v.push(styled("│", lc));
    v.extend(left);
    if lp > 0 { v.push(Span::raw(" ".repeat(lp))); }
    v.push(styled("│", lc));
    v.push(Span::raw("  ".to_string()));
    v.push(styled("│", rc));
    v.extend(right);
    if rp > 0 { v.push(Span::raw(" ".repeat(rp))); }
    v.push(styled("│", rc));
    Line::from(v)
}

/// Dual box top: "   ┌─ lt ──...──┐  ┌─ rt ──...──┐"
pub(crate) fn dual_top(lt: &str, lc: Color, rt: &str, rc: Color) -> Line<'static> {
    let lr = SIDE.saturating_sub(lt.chars().count() + 3);
    let rr = SIDE.saturating_sub(rt.chars().count() + 3);
    Line::from(vec![
        Span::raw("   ".to_string()),
        bold(&format!("┌─ {} {}┐", lt, "─".repeat(lr)), lc),
        Span::raw("  ".to_string()),
        bold(&format!("┌─ {} {}┐", rt, "─".repeat(rr)), rc),
    ])
}

/// Dual box bottom: "   └──...──┘  └──...──┘"
pub(crate) fn dual_bot(lc: Color, rc: Color) -> Line<'static> {
    Line::from(vec![
        Span::raw("   ".to_string()),
        bold(&format!("└{}┘", "─".repeat(SIDE)), lc),
        Span::raw("  ".to_string()),
        bold(&format!("└{}┘", "─".repeat(SIDE)), rc),
    ])
}

// ─────────────────────────── Slide 0: Welcome ───────────────────────────

fn slide_welcome(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let body_c = theme.body_text;
    let green = theme.success;
    let yellow = theme.keyword;

    let mut l = Vec::new();
    l.push(blank());
    l.push(blank());

    l.push(dtop(FULL, cyan));
    l.push(drow(cyan, vec![], FULL));
    l.push(drow(cyan, vec![
        Span::raw("              ".to_string()),
        bold("L E A R N", Color::White),
        bold("  ", cyan),
        bold("L O C A L", green),
    ], FULL));
    l.push(drow(cyan, vec![
        Span::raw("              ".to_string()),
        styled("Interactive Programming Tutorials", body_c),
    ], FULL));
    l.push(drow(cyan, vec![], FULL));
    l.push(dbot(FULL, cyan));
    l.push(blank());

    l.push(Line::from(styled(
        "   Learn programming offline, in your terminal. No browser, no account, no internet.", body_c,
    )));
    l.push(Line::from(styled(
        "   Real exercises, real compilers, real feedback — everything runs on your machine.", body_c,
    )));
    l.push(blank());

    l.push(heading("What makes LearnLocal different:", theme));
    l.push(blank());

    // Feature grid using dual_top/dual/dual_bot for alignment
    l.push(dual_top("100% Offline", yellow, "Built-in Editor", yellow));
    l.push(dual(
        yellow, vec![muted("  Your code never leaves your", theme)],
        yellow, vec![muted("  Edit code right in the TUI.", theme)],
    ));
    l.push(dual(
        yellow, vec![muted("  machine. No telemetry.", theme)],
        yellow, vec![muted("  Line numbers, auto-scroll.", theme)],
    ));
    l.push(dual_bot(yellow, yellow));
    l.push(dual_top("Instant Feedback", green, "AI Tutor (Optional)", green));
    l.push(dual(
        green, vec![muted("  Colored diffs show where", theme)],
        green, vec![muted("  Ask questions while you", theme)],
    ));
    l.push(dual(
        green, vec![muted("  output differs. Errors shown.", theme)],
        green, vec![muted("  code. Enable in Settings.", theme)],
    ));
    l.push(dual_bot(green, green));
    l.push(dual_top("Real Sandboxing", cyan, "Progress Tracking", cyan));
    l.push(dual(
        cyan, vec![muted("  Exercises run in isolated", theme)],
        cyan, vec![muted("  Pick up where you left off.", theme)],
    ));
    l.push(dual(
        cyan, vec![muted("  sandboxes. Never affects system.", theme)],
        cyan, vec![muted("  Stats and celebrations.", theme)],
    ));
    l.push(dual_bot(cyan, cyan));
    l.push(dual_top("Environment Engine", yellow, "Lesson Sandboxes", yellow));
    l.push(dual(
        yellow, vec![muted("  Courses set up files, services,", theme)],
        yellow, vec![muted("  Free playground per lesson", theme)],
    ));
    l.push(dual(
        yellow, vec![muted("  ports — real dev environments.", theme)],
        yellow, vec![muted("  to experiment beyond exercises.", theme)],
    ));
    l.push(dual_bot(yellow, yellow));

    l.push(blank());
    l.push(Line::from(vec![
        muted("   Press ", theme),
        styled("→", cyan),
        muted(" or ", theme),
        styled("Enter", cyan),
        muted(" to continue the tour...", theme),
    ]));
    l
}

// ─────────────────────────── Slide 1: Core Loop ─────────────────────────

fn slide_core_loop(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let body_c = theme.body_text;
    let green = theme.success;
    let yellow = theme.keyword;
    let blue = Color::Rgb(180, 220, 255);
    let magenta = Color::Magenta;

    // Each flowchart box: inner width 27, total 29 (│ + 27 + │)
    // Line layout: 7 indent + 29 left + 13 gap + 29 right = 78
    let bw = 27;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("THE CORE LOOP", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Every exercise follows the same cycle. No guessing what to do next:", theme,
    )));
    l.push(blank());

    // Row 1: READ → EDIT (top borders)
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("┌{}┐", "─".repeat(bw)), cyan),
        Span::raw("             ".to_string()),
        bold(&format!("┌{}┐", "─".repeat(bw)), green),
    ]));
    // READ content
    for (label, color, _desc, rdesc, rcolor) in [
        ("  1. READ", blue, "                  ", "  2. EDIT", yellow),
    ] {
        l.push(Line::from(vec![
            Span::raw("       ".to_string()),
            bold("│", cyan), styled(&format!("{:<w$}", label, w = bw), color), bold("│", cyan),
            body("   ────────→ ", theme),
            bold("│", green), styled(&format!("{:<w$}", rdesc, w = bw), rcolor), bold("│", green),
        ]));
    }
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", cyan), styled(&format!("{:<w$}", "  Read the lesson content.", w = bw), body_c), bold("│", cyan),
        Span::raw("             ".to_string()),
        bold("│", green), styled(&format!("{:<w$}", "  Write your solution in", w = bw), body_c), bold("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", cyan), styled(&format!("{:<w$}", "  [Space] reveals sections", w = bw), body_c), bold("│", cyan),
        Span::raw("             ".to_string()),
        bold("│", green), styled(&format!("{:<w$}", "  [e] opens the built-in", w = bw), body_c), bold("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", cyan), styled(&format!("{:<w$}", "  progressively as you go.", w = bw), body_c), bold("│", cyan),
        Span::raw("             ".to_string()),
        bold("│", green), styled(&format!("{:<w$}", "  inline editor.", w = bw), body_c), bold("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("└{}┘", "─".repeat(bw)), cyan),
        Span::raw("             ".to_string()),
        bold(&format!("└{}┬{}┘", "─".repeat(bw / 2), "─".repeat(bw - bw / 2 - 1)), green),
    ]));

    // Connector
    l.push(Line::from(body(&format!("       {}│", " ".repeat(29 + 13 + bw / 2)), theme)));

    // Row 2: SUBMIT ← RUN (top borders)
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("┌{}┐", "─".repeat(bw)), yellow),
        Span::raw("             ".to_string()),
        bold(&format!("┌{}┴{}┐", "─".repeat(bw / 2), "─".repeat(bw - bw / 2 - 1)), magenta),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", yellow), styled(&format!("{:<w$}", "  4. SUBMIT", w = bw), green), bold("│", yellow),
        body("   ←────────  ", theme),
        bold("│", magenta), styled(&format!("{:<w$}", "  3. RUN", w = bw), blue), bold("│", magenta),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", yellow), styled(&format!("{:<w$}", "  Grade when ready with [t].", w = bw), body_c), bold("│", yellow),
        Span::raw("             ".to_string()),
        bold("│", magenta), styled(&format!("{:<w$}", "  Test with [Enter] — see", w = bw), body_c), bold("│", magenta),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", yellow), styled(&format!("{:<w$}", "  ✓ Pass → next exercise.", w = bw), body_c), bold("│", yellow),
        Span::raw("             ".to_string()),
        bold("│", magenta), styled(&format!("{:<w$}", "  output without grading.", w = bw), body_c), bold("│", magenta),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold("│", yellow), styled(&format!("{:<w$}", "  ✘ Fail → see feedback.", w = bw), body_c), bold("│", yellow),
        Span::raw("             ".to_string()),
        bold("│", magenta), styled(&format!("{:<w$}", "  Run as often as you want.", w = bw), body_c), bold("│", magenta),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("└{}┘", "─".repeat(bw)), yellow),
        Span::raw("             ".to_string()),
        bold(&format!("└{}┘", "─".repeat(bw)), magenta),
    ]));

    l.push(blank());
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[Enter]", cyan), body("  Run code (see output, no grading)        ", theme),
        styled("[r]", cyan), body("  Reset to starter code", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[t]    ", cyan), body("  Submit for grading (checks answer)       ", theme),
        styled("[?]", cyan), body("  Help overlay", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[Space]", cyan), body("  Reveal next lesson section               ", theme),
        styled("[Esc]  ", cyan), body("Return home", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[h]    ", cyan), body("  Progressive hints (reveals one at a time)", theme),
    ]));
    l.push(Line::from(vec![
        body("   ", theme),
        styled("[s]    ", cyan), body("  Skip exercise (come back later)", theme),
    ]));
    l.push(blank());
    l.push(Line::from(vec![
        body("   Only ", theme), styled("[t] submit", green),
        body(" counts toward progress — run freely, submit when confident.", theme),
    ]));
    l.push(Line::from(vec![
        body("   Stuck?  ", theme),
        styled("[h]", yellow), body(" hints first  →  ", theme),
        styled("[s]", yellow), body(" skip and return later", theme),
    ]));

    l
}

// ─────────────────────────── Slide 2: Exercise Types ─────────────────────

fn slide_exercise_types(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let blue = Color::Rgb(180, 220, 255);
    let magenta = Color::Magenta;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("EXERCISE TYPES", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Courses use six exercise types to build different skills:", theme,
    )));
    l.push(blank());

    // Exercise types — full width box
    l.push(top("", FULL, cyan));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [WRITE]      ", green),
        body("Write code from scratch — the most common type.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 You get starter code with a TODO and build the solution.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [FIX]        ", yellow),
        body("Find and fix the bug in broken code.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 The code compiles/runs but produces wrong output.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [FILL BLANK] ", cyan),
        body("Complete partially-written code by filling gaps.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 Key parts are replaced with blanks you fill in.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [PREDICT]    ", blue),
        body("Read code, predict the output — no editing.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 Type what you think the code will print.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [CHOICE]     ", magenta),
        body("Select the correct answer from multiple options.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 Tests conceptual understanding without writing code.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        bold("  [COMMAND]    ", Color::Blue),
        body("Run shell commands — git, sysadmin, file ops.", theme),
    ], FULL));
    l.push(row(cyan, vec![
        body("                 Write shell scripts regardless of the course language.", theme),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(bot(FULL, cyan));
    l.push(blank());

    // Side-by-side: Code Exercises / Shell Mode
    l.push(dual_top("Code Exercises", green, "Shell Mode [COMMAND]", Color::Blue));
    l.push(dual(green, vec![], Color::Blue, vec![]));
    l.push(dual(
        green, vec![styled("  → ", green), body("Opens the inline editor", theme)],
        Color::Blue, vec![styled("  → ", Color::Blue), body("Opens a terminal prompt", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("Write/fix/fill-blank/predict", theme)],
        Color::Blue, vec![styled("  → ", Color::Blue), body("Type shell commands live", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("[Enter] run, [t] submit", theme)],
        Color::Blue, vec![styled("  → ", Color::Blue), body("[Enter] runs + validates", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("Choice: number keys", theme)],
        Color::Blue, vec![styled("  → ", Color::Blue), body("[↑/↓] command history", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("Predict: type answer", theme)],
        Color::Blue, vec![styled("  → ", Color::Blue), body("[Ctrl+H] hints, [Esc] exit", theme)],
    ));
    l.push(dual(green, vec![], Color::Blue, vec![]));
    l.push(dual_bot(green, Color::Blue));

    l
}

// ─────────────────────────── Slide 3: Editor ────────────────────────────

fn slide_editor(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;
    let blue = Color::Rgb(180, 220, 255);

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("THE INLINE EDITOR", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Press [e] to open the built-in editor — edit code right inside the TUI:", theme,
    )));
    l.push(blank());

    // Inline editor — full width code mockup
    l.push(top("[e] Inline Editor", FULL, green));
    l.push(row(green, vec![
        styled("  1 │ ", muted_c), styled("#include <iostream>", blue),
    ], FULL));
    l.push(row(green, vec![
        styled("  2 │ ", muted_c), styled("using namespace std;", blue),
    ], FULL));
    l.push(row(green, vec![
        styled("  3 │ ", muted_c),
    ], FULL));
    l.push(row(green, vec![
        styled("  4 │ ", muted_c), styled("int main() {", blue),
    ], FULL));
    l.push(row(green, vec![
        styled("  5 │ ", muted_c), styled("    cout << \"Hello, World!\" << endl;", blue),
    ], FULL));
    l.push(row(green, vec![
        styled("  6 │ ", muted_c), styled("    return 0;", blue),
    ], FULL));
    l.push(row(green, vec![
        styled("  7 │ ", muted_c), styled("}", blue),
    ], FULL));
    l.push(bot(FULL, green));
    l.push(blank());

    // Controls + Tips — side by side
    l.push(dual_top("Controls", cyan, "Tips", yellow));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual(
        cyan, vec![styled("  Arrows", green), body("  Move cursor", theme)],
        yellow, vec![styled("  →", green), body(" Line numbers shown", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  Type  ", green), body("  Insert text", theme)],
        yellow, vec![styled("  →", green), body(" Auto-scrolls to cursor", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  Bksp  ", green), body("  Delete backward", theme)],
        yellow, vec![styled("  →", green), body(" Code box replaces the", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  Enter ", green), body("  New line", theme)],
        yellow, vec![body("    lesson content while open", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  Tab   ", green), body("  Insert spaces", theme)],
        yellow, vec![],
    ));
    l.push(dual(
        cyan, vec![styled("  Home  ", green), body("  Line start", theme)],
        yellow, vec![styled("  →", green), body(" Esc saves and closes —", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  End   ", green), body("  Line end", theme)],
        yellow, vec![body("    see output immediately", theme)],
    ));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual(
        cyan, vec![styled("  Esc   ", green), body("  Save & close", theme)],
        yellow, vec![styled("  →", green), body(" Draft saved if you quit", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  Ctrl+S", green), body(" Save (stay open)", theme)],
        yellow, vec![body("    before finishing", theme)],
    ));
    l.push(dual(cyan, vec![], yellow, vec![]));
    l.push(dual_bot(cyan, yellow));

    l
}

// ─────────────────────────── Slide 4: Feedback ──────────────────────────

fn slide_feedback(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let body_c = theme.body_text;
    let green = theme.success;
    let red = theme.error;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("SMART FEEDBACK", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   When your code doesn't pass, LearnLocal shows you exactly why — not just \"wrong\":", theme,
    )));
    l.push(blank());

    // Two-column: diff + diagnostics
    l.push(dual_top("Output Diff", cyan, "Compiler Diagnostics", red));
    l.push(dual(cyan, vec![], red, vec![]));
    l.push(dual(
        cyan, vec![styled("  Expected: ", muted_c), styled("Hello, World!", green)],
        red, vec![styled("  error[E0308]:", red), styled(" mismatched", body_c)],
    ));
    l.push(dual(
        cyan, vec![styled("  Got:      ", muted_c), styled("hello world", red)],
        red, vec![styled("  types", body_c)],
    ));
    l.push(dual(
        cyan, vec![styled("             ^     ^", yellow)],
        red, vec![styled("    --> ", body_c), styled("main.rs:3:12", muted_c)],
    ));
    l.push(dual(cyan, vec![], red, vec![]));
    l.push(dual(
        cyan, vec![body("  Color-coded line-by-line", theme)],
        red, vec![styled("    expected ", body_c), styled("`i32`", yellow)],
    ));
    l.push(dual(
        cyan, vec![body("  diff shows exactly where", theme)],
        red, vec![styled("    found    ", body_c), styled("`&str`", yellow)],
    ));
    l.push(dual(
        cyan, vec![body("  output differs.", theme)],
        red, vec![body("  Parsed with file/line info.", theme)],
    ));
    l.push(dual_bot(cyan, red));
    l.push(blank());

    // Assertions — full width
    l.push(top("State Assertions (for environment exercises)", FULL, green));
    l.push(row(green, vec![], FULL));
    l.push(row(green, vec![
        styled("  ✓ ", green), body("File output.txt exists", theme),
    ], FULL));
    l.push(row(green, vec![
        styled("  ✓ ", green), body("File contains \"success\"", theme),
    ], FULL));
    l.push(row(green, vec![
        styled("  ✓ ", green), body("Directory backup/ exists", theme),
    ], FULL));
    l.push(row(green, vec![
        styled("  ✓ ", green), body("Symlink latest → backup/2024-01-15", theme),
    ], FULL));
    l.push(row(green, vec![
        styled("  ✘ ", red), body("File permissions are 0644 (got 0755)", theme),
    ], FULL));
    l.push(row(green, vec![
        styled("  ✘ ", red), body("Line count matches expected (got 3, want 5)", theme),
    ], FULL));
    l.push(row(green, vec![], FULL));
    l.push(row(green, vec![
        body("  Courses can validate: output text, file existence, contents, permissions,", theme),
    ], FULL));
    l.push(row(green, vec![
        body("  symlinks, directory state, file counts, regex patterns, and more.", theme),
    ], FULL));
    l.push(bot(FULL, green));
    l.push(blank());
    l.push(Line::from(vec![
        body("   Every error is a learning opportunity — feedback ", theme),
        styled("shows you why, not just that it's wrong.", green),
    ]));

    l
}

// ─────────────────────────── Slide 5: Environment ───────────────────────

fn slide_environment(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let body_c = theme.body_text;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    // Pipeline boxes: inner 35, total 37 (│ + 35 + │)
    // Layout: 7 indent + 37 box + 5 gap + text = fills to ~78
    let pw = 35;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("ENVIRONMENT ENGINE", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Courses can simulate real development environments — not just compile-and-run:", theme,
    )));
    l.push(blank());

    // Setup box
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("┌─ 1. Setup {}┐", "─".repeat(pw - 13)), yellow),
        body("     ", theme), bold("What courses can do:", cyan),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", yellow), styled(&format!("{:<w$}", " Create files and directories", w = pw), body_c), styled("│", yellow),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", yellow), styled(&format!("{:<w$}", " Set environment variables", w = pw), body_c), styled("│", yellow),
        body("     ", theme), styled("→ ", green), body("Create project scaffolding", theme),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", yellow), styled(&format!("{:<w$}", " Run setup commands", w = pw), body_c), styled("│", yellow),
        body("     ", theme), styled("→ ", green), body("Pre-populate config files", theme),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", yellow), styled(&format!("{:<w$}", " Create symlinks", w = pw), body_c), styled("│", yellow),
        body("     ", theme), styled("→ ", green), body("Initialize databases", theme),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("└{}┬{}┘", "─".repeat(pw / 2), "─".repeat(pw - pw / 2 - 1)), yellow),
        body("     ", theme), styled("→ ", green), body("Start background servers", theme),
    ]));
    // Connector
    l.push(Line::from(vec![
        body(&format!("       {}│", " ".repeat(pw / 2 + 1)), theme),
        body("                       ", theme),
        styled("→ ", green), body("Allocate dynamic ports", theme),
    ]));

    // Runtime box
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("┌{}┴{}┐", "─".repeat(pw / 2), "─".repeat(pw - pw / 2 - 1)), green),
        body("     ", theme), styled("→ ", green), body("Validate filesystem state", theme),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", green), styled(&format!("{:<w$}", " Start background services", w = pw), body_c), styled("│", green),
        body("     ", theme), styled("→ ", green), body("Check file permissions", theme),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", green), styled(&format!("{:<w$}", " Allocate dynamic ports", w = pw), body_c), styled("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", green), bold(&format!("{:<w$}", " ══► YOUR CODE RUNS HERE ══►", w = pw), Color::White), styled("│", green),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", green), styled(&format!("{:<w$}", " Sandboxed with loopback network", w = pw), body_c), styled("│", green),
        body("     ", theme), bold("Sandboxing tiers:", cyan),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("└{}┬{}┘", "─".repeat(pw / 2), "─".repeat(pw - pw / 2 - 1)), green),
        body("     ", theme), styled("  Basic:     ", yellow), body("tmpdir + timeout", theme),
    ]));
    l.push(Line::from(vec![
        body(&format!("       {}│", " ".repeat(pw / 2 + 1)), theme),
        body("                       ", theme),
        styled("  Contained: ", yellow), body("firejail / bubblewrap", theme),
    ]));

    // Teardown box
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("┌{}┴{}┐", "─".repeat(pw / 2), "─".repeat(pw - pw / 2 - 1)), muted_c),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", muted_c), styled(&format!("{:<w$}", " Teardown + cleanup", w = pw), body_c), styled("│", muted_c),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        styled("│", muted_c), styled(&format!("{:<w$}", " Validate final state assertions", w = pw), body_c), styled("│", muted_c),
    ]));
    l.push(Line::from(vec![
        Span::raw("       ".to_string()),
        bold(&format!("└{}┘", "─".repeat(pw)), muted_c),
    ]));
    l.push(blank());
    l.push(Line::from(vec![
        body("   All execution is ", theme),
        styled("sandboxed", yellow),
        body(" — courses run in isolated temp directories, can never affect your system.", theme),
    ]));

    l
}

// ─────────────────────────── Slide 6: AI Tutor ──────────────────────────

fn slide_ai_tutor(theme: &Theme) -> Vec<Line<'static>> {
    let mut l = Vec::new();
    l.push(blank());

    #[cfg(feature = "llm")]
    {
        let cyan = theme.heading;
        let body_c = theme.body_text;
        let yellow = theme.keyword;
        let green = theme.success;
        l.push(heading("AI TUTOR", theme));
        l.push(separator(theme));
        l.push(blank());
        l.push(Line::from(body(
            "   Stuck? Press [a] to chat with an AI that understands your full context:", theme,
        )));
        l.push(blank());

        l.push(top("AI Chat", FULL, cyan));
        l.push(row(cyan, vec![], FULL));
        l.push(row(cyan, vec![
            styled("  You: ", yellow),
            styled("My function returns 0 instead of the sum. What am I doing wrong?", body_c),
        ], FULL));
        l.push(row(cyan, vec![], FULL));
        l.push(row(cyan, vec![
            styled("  AI: ", green),
            styled("Looking at your code, I see the issue — you declared `sum` inside", body_c),
        ], FULL));
        l.push(row(cyan, vec![
            styled("      the loop, so it resets to 0 each iteration. Try moving the", body_c),
        ], FULL));
        l.push(row(cyan, vec![
            styled("      declaration before the for-loop. Think about variable scope —", body_c),
        ], FULL));
        l.push(row(cyan, vec![
            styled("      where does `sum` need to live to accumulate across iterations?", body_c),
        ], FULL));
        l.push(row(cyan, vec![], FULL));
        l.push(row(cyan, vec![
            styled("  You: ", yellow),
            styled("Oh! Moving it outside the loop fixed it. Thanks!", body_c),
        ], FULL));
        l.push(row(cyan, vec![], FULL));
        l.push(bot(FULL, cyan));
        l.push(blank());

        // Two columns
        l.push(dual_top("The AI sees your context", cyan, "How it works", green));
        l.push(dual(cyan, vec![], green, vec![]));
        l.push(dual(
            cyan, vec![styled("  → ", green), body("Lesson content", theme)],
            green, vec![styled("  → ", green), body("Runs 100% on your machine", theme)],
        ));
        l.push(dual(
            cyan, vec![styled("  → ", green), body("Your current code", theme)],
            green, vec![styled("  → ", green), body("No data sent anywhere", theme)],
        ));
        l.push(dual(
            cyan, vec![styled("  → ", green), body("Compiler errors & output", theme)],
            green, vec![styled("  → ", green), body("Guides without spoiling", theme)],
        ));
        l.push(dual(
            cyan, vec![styled("  → ", green), body("Exercise requirements", theme)],
            green, vec![styled("  → ", green), body("Works in sandboxes too", theme)],
        ));
        l.push(dual(
            cyan, vec![styled("  → ", green), body("Your attempt history", theme)],
            green, vec![styled("  → ", green), body("Enable in Settings [s]", theme)],
        ));
        l.push(dual(cyan, vec![], green, vec![]));
        l.push(dual_bot(cyan, green));
        l.push(blank());
        l.push(Line::from(vec![
            body("   Multi-line input: ", theme),
            styled("Enter", cyan), body(" = newline, ", theme),
            styled("Ctrl+Enter", cyan), body(" or ", theme),
            styled("Tab", cyan), body(" = send. ", theme),
            styled("[Esc]", cyan), body(" closes chat.", theme),
        ]));
    }

    #[cfg(not(feature = "llm"))]
    {
        let muted_c = theme.muted;
        let green = theme.success;
        l.push(heading("AI TUTOR (OPTIONAL)", theme));
        l.push(separator(theme));
        l.push(blank());
        l.push(Line::from(body(
            "   LearnLocal supports an optional AI tutor you can enable in Settings.", theme,
        )));
        l.push(Line::from(body(
            "   It guides you without spoiling the answer.", theme,
        )));
        l.push(blank());

        l.push(top("What it can do", FULL, muted_c));
        l.push(row(muted_c, vec![], FULL));
        l.push(row(muted_c, vec![
            styled("  → ", green), body("Answer questions about the lesson you're reading", theme),
        ], FULL));
        l.push(row(muted_c, vec![
            styled("  → ", green), body("Help debug your code using your actual errors and output", theme),
        ], FULL));
        l.push(row(muted_c, vec![
            styled("  → ", green), body("Guide you toward the solution without giving it away", theme),
        ], FULL));
        l.push(row(muted_c, vec![
            styled("  → ", green), body("Sees your lesson, code, compiler output, and hints", theme),
        ], FULL));
        l.push(row(muted_c, vec![], FULL));
        l.push(row(muted_c, vec![
            body("  Runs 100% on your machine — no data sent anywhere.", theme),
        ], FULL));
        l.push(row(muted_c, vec![
            body("  Enable via ", theme), styled("Settings [s]", green), body(" on the home screen.", theme),
        ], FULL));
        l.push(row(muted_c, vec![], FULL));
        l.push(bot(FULL, muted_c));
    }

    l
}

// ─────────────────────────── Slide 7: Progress + Sandboxes ──────────────

fn slide_progress_combined(theme: &Theme) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;
    let muted_c = theme.muted;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("PROGRESS & SANDBOXES", theme));
    l.push(separator(theme));
    l.push(blank());
    l.push(Line::from(body(
        "   Your progress is saved automatically — pick up exactly where you left off:", theme,
    )));
    l.push(blank());

    // Progress bars — full width
    l.push(top("Course Progress", FULL, cyan));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        body("  C++ Fundamentals    ", theme),
        styled("████████████████████", green), styled("░░░░░", muted_c),
        body("  L6/8  Ex 42/55  ", theme), styled("76%", green),
    ], FULL));
    l.push(row(cyan, vec![
        body("  Python Fundamentals ", theme),
        styled("████████████████", green), styled("░░░░░░░░░", muted_c),
        body("  L5/8  Ex 33/54  ", theme), styled("61%", green),
    ], FULL));
    l.push(row(cyan, vec![
        body("  JS Fundamentals     ", theme),
        styled("████████████", green), styled("░░░░░░░░░░░░░", muted_c),
        body("  L4/8  Ex 28/56  ", theme), styled("50%", yellow),
    ], FULL));
    l.push(row(cyan, vec![], FULL));
    l.push(row(cyan, vec![
        body("  Saved at ", theme), styled("~/.local/share/learnlocal/progress.json", yellow),
        body(" — keyed by course + major version.", theme),
    ], FULL));
    l.push(bot(FULL, cyan));
    l.push(blank());

    // Side-by-side: Drafts & Sandboxes / Celebrations
    l.push(dual_top("Drafts & Sandboxes", green, "Celebrations", yellow));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual(
        green, vec![styled("  → ", green), body("Draft code saved on exit", theme)],
        yellow, vec![styled("  ★ ", yellow), body("Exercise → success flash", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("Restored when you return", theme)],
        yellow, vec![styled("  ★ ", yellow), body("Lesson → stats summary", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("Cleared on completion", theme)],
        yellow, vec![styled("  ★ ", yellow), body("Course → full celebration", theme)],
    ));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual(
        green, vec![bold("  Lesson Sandboxes:", green)],
        yellow, vec![],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("One playground per lesson", theme)],
        yellow, vec![body("  View aggregate stats:", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("No grading — just explore", theme)],
        yellow, vec![styled("  [t]", cyan), body(" Stats from home", theme)],
    ));
    l.push(dual(
        green, vec![styled("  → ", green), body("[s] from lesson recap", theme)],
        yellow, vec![styled("  [p]", cyan), body(" Progress details", theme)],
    ));
    l.push(dual(
        green, vec![],
        yellow, vec![styled("  [r]", cyan), body(" Reset course progress", theme)],
    ));
    l.push(dual(green, vec![], yellow, vec![]));
    l.push(dual_bot(green, yellow));
    l.push(blank());
    l.push(Line::from(vec![
        body("   Sandboxes at ", theme),
        styled("~/.local/share/learnlocal/sandboxes/", yellow),
        body(" — persist across sessions.", theme),
    ]));

    l
}

// ─────────────────────────── Slide 8: Get Started ───────────────────────

fn slide_get_started(theme: &Theme, courses: &[String]) -> Vec<Line<'static>> {
    let cyan = theme.heading;
    let green = theme.success;
    let yellow = theme.keyword;

    let mut l = Vec::new();
    l.push(blank());
    l.push(heading("GET STARTED", theme));
    l.push(separator(theme));
    l.push(blank());

    if courses.is_empty() {
        l.push(Line::from(body("   No courses currently installed.", theme)));
        l.push(blank());
    } else {
        l.push(heading("Available Courses:", theme));
        l.push(blank());
        for name in courses {
            l.push(Line::from(vec![
                body("     ", theme), styled("→ ", green), body(name, theme),
            ]));
        }
        l.push(blank());
    }

    l.push(Line::from(body(
        "   Some courses require specific tools or OS — the home screen shows what's ready.", theme,
    )));
    l.push(blank());

    // Key reference — side by side
    l.push(dual_top("Home Screen", cyan, "Inside a Course", green));
    l.push(dual(cyan, vec![], green, vec![]));
    l.push(dual(
        cyan, vec![styled("  [Enter]", cyan), body("  Start course", theme)],
        green, vec![styled("  [e]", cyan), body("      Inline editor", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [→]", cyan), body("      Browse lessons", theme)],
        green, vec![styled("  [Enter]", cyan), body("  Run (no grading)", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [w]", cyan), body("      Welcome tour", theme)],
        green, vec![styled("  [t]", cyan), body("      Submit for grading", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [h]", cyan), body("      How To reference", theme)],
        green, vec![styled("  [a]", cyan), body("      AI chat (optional)", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [t]", cyan), body("      Stats overview", theme)],
        green, vec![styled("  [h]", cyan), body("      Reveal hint", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [p]", cyan), body("      Progress details", theme)],
        green, vec![styled("  [s]", cyan), body("      Skip exercise", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [s]", cyan), body("      Settings", theme)],
        green, vec![styled("  [Space]", cyan), body("  Reveal next section", theme)],
    ));
    l.push(dual(
        cyan, vec![styled("  [q]", cyan), body("      Quit", theme)],
        green, vec![styled("  [r]", cyan), body("      Reset starter code", theme)],
    ));
    l.push(dual(
        cyan, vec![],
        green, vec![styled("  [?]", cyan), body("      Help overlay", theme)],
    ));
    l.push(dual(
        cyan, vec![],
        green, vec![styled("  [←/→]", cyan), body("    Navigate lessons", theme)],
    ));
    l.push(dual(
        cyan, vec![],
        green, vec![styled("  [Esc]", cyan), body("    Return home", theme)],
    ));
    l.push(dual_bot(cyan, green));
    l.push(blank());

    // Shell mode compact reference
    l.push(Line::from(vec![
        body("   ", theme),
        styled("Shell Mode", Color::Blue),
        body(" (command exercises): ", theme),
        styled("[Enter]", cyan), body(" run  ", theme),
        styled("[↑/↓]", cyan), body(" history  ", theme),
        styled("[Ctrl+H]", cyan), body(" hint  ", theme),
        styled("[Esc]", cyan), body(" exit", theme),
    ]));
    l.push(blank());
    l.push(Line::from(vec![
        body("   Press ", theme),
        styled("[Esc]", yellow),
        body(" to return home and pick a course. Happy learning!", theme),
    ]));
    l.push(blank());
    l.push(Line::from(muted(
        &format!("   {}", "─".repeat(75)), theme,
    )));
    l.push(Line::from(muted(
        "   LearnLocal — learn programming offline, in your terminal.", theme,
    )));

    l
}
