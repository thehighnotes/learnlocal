use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "learnlocal", version, about = "Offline terminal-based programming tutorials")]
pub struct Cli {
    /// Custom courses directory
    #[arg(long = "courses", global = true)]
    pub courses_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List available courses
    List,

    /// Start or resume a course
    Start {
        /// Course name
        course: String,

        /// Jump to a specific lesson
        #[arg(long)]
        lesson: Option<String>,
    },

    /// Show progress for a course
    Progress {
        /// Course name
        course: String,
    },

    /// Reset progress for a course
    Reset {
        /// Course name
        course: String,
    },

    /// Validate a course directory (for course authors)
    Validate {
        /// Path to the course directory
        path: PathBuf,

        /// Also run all solutions against validation
        #[arg(long, default_value_t = false)]
        run_solutions: bool,
    },
}
