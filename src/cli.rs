use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "learnlocal",
    version,
    about = "Offline terminal-based programming tutorials"
)]
pub struct Cli {
    /// Custom courses directory
    #[arg(long = "courses", global = true)]
    pub courses_dir: Option<PathBuf>,

    /// Enable verbose output for troubleshooting
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List available courses
    #[command(
        after_help = "EXAMPLES:\n  learnlocal list\n  learnlocal --courses ~/my-courses list"
    )]
    List,

    /// Start or resume a course
    #[command(
        after_help = "EXAMPLES:\n  learnlocal start cpp-fundamentals\n  learnlocal start python-fundamentals --lesson 03-functions"
    )]
    Start {
        /// Course name
        course: String,

        /// Jump to a specific lesson
        #[arg(long)]
        lesson: Option<String>,
    },

    /// Show progress for a course
    #[command(after_help = "EXAMPLES:\n  learnlocal progress cpp-fundamentals")]
    Progress {
        /// Course name
        course: String,
    },

    /// Reset progress for a course
    #[command(after_help = "EXAMPLES:\n  learnlocal reset cpp-fundamentals")]
    Reset {
        /// Course name
        course: String,
    },

    /// Validate a course directory (for course authors)
    #[command(
        after_help = "EXAMPLES:\n  learnlocal validate courses/cpp-fundamentals\n  learnlocal validate courses/cpp-fundamentals --run-solutions"
    )]
    Validate {
        /// Path to the course directory
        path: PathBuf,

        /// Also run all solutions against validation
        #[arg(long, default_value_t = false)]
        run_solutions: bool,
    },

    /// Generate shell completions
    #[command(
        after_help = "EXAMPLES:\n  learnlocal completions bash > ~/.bash_completion.d/learnlocal\n  learnlocal completions zsh > ~/.zfunc/_learnlocal\n  learnlocal completions fish > ~/.config/fish/completions/learnlocal.fish"
    )]
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },

    /// Generate man page
    #[command(
        after_help = "EXAMPLES:\n  learnlocal man > learnlocal.1\n  learnlocal man | man -l -"
    )]
    Man,

    /// Check system readiness for running courses
    #[command(
        after_help = "EXAMPLES:\n  learnlocal doctor\n  learnlocal --courses ~/my-courses doctor"
    )]
    Doctor,

    /// Scaffold a new course directory for authors
    #[command(
        after_help = "EXAMPLES:\n  learnlocal init my-rust-course\n  learnlocal init python-web-basics"
    )]
    Init {
        /// Name for the new course directory
        name: String,
    },

    /// Export progress to stdout
    #[command(
        after_help = "EXAMPLES:\n  learnlocal export\n  learnlocal export --format csv\n  learnlocal export cpp-fundamentals"
    )]
    Export {
        /// Course name (all courses if omitted)
        course: Option<String>,

        /// Output format: json or csv
        #[arg(long, default_value = "json")]
        format: String,
    },
}
