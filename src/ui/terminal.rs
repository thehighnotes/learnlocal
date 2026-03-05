use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

static SIGNAL_FLAG: std::sync::OnceLock<Arc<AtomicBool>> = std::sync::OnceLock::new();

/// Check if a termination signal (SIGTERM/SIGINT/SIGHUP) has been received.
pub fn signal_received() -> bool {
    SIGNAL_FLAG
        .get()
        .map(|f| f.load(Ordering::Relaxed))
        .unwrap_or(false)
}

fn register_signal_handlers() {
    let flag = SIGNAL_FLAG.get_or_init(|| Arc::new(AtomicBool::new(false)));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(flag));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(flag));
    let _ = signal_hook::flag::register(signal_hook::consts::SIGHUP, Arc::clone(flag));
}

/// Set up the terminal for TUI mode.
pub fn setup() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Register signal handlers for clean terminal restoration
    register_signal_handlers();

    // Install panic hook to restore terminal and write crash report on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        write_crash_report(panic_info);
        original_hook(panic_info);
    }));

    Ok(terminal)
}

/// Write a crash report to `learnlocal-crash.log` in the current directory.
#[allow(deprecated)] // PanicInfo renamed to PanicHookInfo in 1.82; keep PanicInfo for MSRV 1.74
fn write_crash_report(panic_info: &std::panic::PanicInfo<'_>) {
    use std::fmt::Write;

    let mut report = String::new();
    let _ = writeln!(report, "=== LearnLocal Crash Report ===");
    let _ = writeln!(report, "Version: {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(report, "Timestamp: {}", chrono::Local::now().to_rfc3339());
    let _ = writeln!(
        report,
        "OS: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );

    if let Ok((cols, rows)) = crossterm::terminal::size() {
        let _ = writeln!(report, "Terminal: {}x{}", cols, rows);
    }

    let _ = writeln!(report);

    // Panic message
    if let Some(msg) = panic_info.payload().downcast_ref::<&str>() {
        let _ = writeln!(report, "Panic: {}", msg);
    } else if let Some(msg) = panic_info.payload().downcast_ref::<String>() {
        let _ = writeln!(report, "Panic: {}", msg);
    } else {
        let _ = writeln!(report, "Panic: (unknown payload)");
    }

    if let Some(location) = panic_info.location() {
        let _ = writeln!(
            report,
            "Location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    let _ = writeln!(report);
    let _ = writeln!(report, "Backtrace:");
    let _ = writeln!(report, "{}", std::backtrace::Backtrace::force_capture());

    let crash_path = "learnlocal-crash.log";
    if std::fs::write(crash_path, &report).is_ok() {
        eprintln!("Crash report written to {}", crash_path);
    }
}

/// Check that the terminal meets minimum size requirements.
/// Returns Ok if size is sufficient or undetectable, Err with a message otherwise.
pub fn check_minimum_size(min_cols: u16, min_rows: u16) -> std::result::Result<(), String> {
    match crossterm::terminal::size() {
        Ok((cols, rows)) if cols < min_cols || rows < min_rows => Err(format!(
            "Terminal too small: {}x{} (minimum {}x{}).\n\
             Please resize your terminal and try again.",
            cols, rows, min_cols, min_rows
        )),
        Err(_) => Ok(()), // Can't detect = proceed (piped, etc.)
        _ => Ok(()),
    }
}

/// Restore the terminal to normal mode.
pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Temporarily leave alternate screen (e.g., for $EDITOR).
pub fn leave_alternate_screen() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

/// Re-enter alternate screen after $EDITOR returns.
pub fn enter_alternate_screen() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    Ok(())
}
