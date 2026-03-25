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

    /// Browse available community courses
    #[command(
        after_help = "EXAMPLES:\n  learnlocal browse\n  learnlocal browse --search python\n  learnlocal browse --search beginner"
    )]
    Browse {
        /// Filter courses by name, language, or tag
        #[arg(long, short = 's')]
        search: Option<String>,
    },

    /// Download and install a community course
    #[command(
        after_help = "EXAMPLES:\n  learnlocal install python-fundamentals\n  learnlocal install cpp-fundamentals"
    )]
    Install {
        /// Course ID from the community registry
        course_id: String,
    },

    /// Log in with GitHub for community features (rate, review, publish)
    #[command(after_help = "EXAMPLES:\n  learnlocal login")]
    Login,

    /// Log out and remove stored token
    #[command(after_help = "EXAMPLES:\n  learnlocal logout")]
    Logout,

    /// Rate a community course (1-5 stars)
    #[command(after_help = "EXAMPLES:\n  learnlocal rate cpp-fundamentals 5")]
    Rate {
        /// Course ID
        course_id: String,
        /// Star rating (1-5)
        stars: u8,
    },

    /// Review a community course
    #[command(
        after_help = "EXAMPLES:\n  learnlocal review cpp-fundamentals \"Great course for beginners!\""
    )]
    Review {
        /// Course ID
        course_id: String,
        /// Review text
        body: String,
    },

    /// Course authoring tools
    #[command(
        after_help = "EXAMPLES:\n  learnlocal author run-solution courses/cpp-fundamentals --lesson variables --exercise declare\n  learnlocal author run-all-solutions courses/cpp-fundamentals --update"
    )]
    Author {
        #[command(subcommand)]
        subcommand: AuthorCommand,
    },

    /// Run the community server
    #[cfg(feature = "server")]
    #[command(after_help = "EXAMPLES:\n  learnlocal server\n  learnlocal server --port 3001")]
    Server {
        /// Port to listen on
        #[arg(long, default_value = "3001")]
        port: u16,

        /// Data directory for SQLite database
        #[arg(long, default_value = "/opt/learnlocal/data")]
        data_dir: PathBuf,

        /// Package storage directory
        #[arg(long, default_value = "/opt/learnlocal/packages")]
        packages_dir: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuthorCommand {
    /// Run a specific exercise's solution and show output
    #[command(
        after_help = "EXAMPLES:\n  learnlocal author run-solution courses/cpp-fundamentals --lesson variables --exercise declare\n  learnlocal author run-solution courses/cpp-fundamentals --lesson variables --exercise declare --update"
    )]
    RunSolution {
        /// Path to the course directory
        path: PathBuf,

        /// Lesson ID
        #[arg(long)]
        lesson: String,

        /// Exercise ID
        #[arg(long)]
        exercise: String,

        /// Auto-update expected_output in the exercise YAML
        #[arg(long)]
        update: bool,
    },

    /// Run ALL solutions and report results
    #[command(
        after_help = "EXAMPLES:\n  learnlocal author run-all-solutions courses/cpp-fundamentals\n  learnlocal author run-all-solutions courses/cpp-fundamentals --update"
    )]
    RunAllSolutions {
        /// Path to the course directory
        path: PathBuf,

        /// Auto-update expected_output in exercise YAML files
        #[arg(long)]
        update: bool,
    },

    /// Package and publish a course to the community server
    #[command(
        after_help = "EXAMPLES:\n  learnlocal author publish courses/cpp-fundamentals\n  learnlocal author publish courses/cpp-fundamentals --dry-run"
    )]
    Publish {
        /// Path to the course directory
        path: PathBuf,

        /// Run checks and package only, don't upload
        #[arg(long)]
        dry_run: bool,
    },

    /// Open the interactive course designer (web UI)
    #[cfg(feature = "author")]
    #[command(
        after_help = "EXAMPLES:\n  learnlocal author design\n  learnlocal author design courses/cpp-fundamentals\n  learnlocal author design --port 8080"
    )]
    Design {
        /// Course directory path (omit to start with welcome screen)
        path: Option<PathBuf>,

        /// Port for the local server (0 = auto-select)
        #[arg(long, default_value = "0")]
        port: u16,

        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },
}
