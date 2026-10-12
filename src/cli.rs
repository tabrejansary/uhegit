use clap::{Parser, Subcommand};
use clap_complete::Shell;

/// UheGit — Automatic Git workspace overview and work-session memory.
///
/// Run without arguments to see your workspace overview.
#[derive(Debug, Parser)]
#[command(
    name = "uhegit",
    version,
    author,
    about = "Automatic Git repository discovery, status overview, and work-session memory",
    long_about = "UheGit automatically discovers your local Git repositories and gives you a\nconcise overview of their status. It also remembers where you left off,\nso you can quickly resume work across multiple projects.\n\nRun 'uhegit' with no arguments to see your workspace overview."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output as JSON (applies to overview/status)
    #[arg(long, global = true)]
    pub json: bool,

    /// Show only dirty repositories (uncommitted changes)
    #[arg(long, conflicts_with_all = ["unpushed", "behind", "conflicts", "clean"])]
    pub dirty: bool,

    /// Show only repositories with unpushed commits
    #[arg(long, conflicts_with_all = ["dirty", "behind", "conflicts", "clean"])]
    pub unpushed: bool,

    /// Show only repositories behind upstream
    #[arg(long, conflicts_with_all = ["dirty", "unpushed", "conflicts", "clean"])]
    pub behind: bool,

    /// Show only repositories with merge conflicts
    #[arg(long, conflicts_with_all = ["dirty", "unpushed", "behind", "clean"])]
    pub conflicts: bool,

    /// Show only clean repositories
    #[arg(long, conflicts_with_all = ["dirty", "unpushed", "behind", "conflicts"])]
    pub clean: bool,

    /// Show only repositories with no UheGit session activity for N days (e.g. 7d)
    #[arg(long, value_name = "DURATION")]
    pub stale: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show workspace overview (default command)
    #[command(alias = "st")]
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
        #[arg(long, conflicts_with_all = ["unpushed", "behind", "conflicts", "clean"])]
        dirty: bool,
        #[arg(long, conflicts_with_all = ["dirty", "behind", "conflicts", "clean"])]
        unpushed: bool,
        #[arg(long, conflicts_with_all = ["dirty", "unpushed", "conflicts", "clean"])]
        behind: bool,
        #[arg(long, conflicts_with_all = ["dirty", "unpushed", "behind", "clean"])]
        conflicts: bool,
        #[arg(long, conflicts_with_all = ["dirty", "unpushed", "behind", "conflicts"])]
        clean: bool,
        #[arg(long, value_name = "DURATION")]
        stale: Option<String>,
    },

    /// Discover Git repositories in default or specified locations
    Scan {
        /// Scan the home directory
        #[arg(long)]
        home: bool,
        /// Scan a specific path
        #[arg(long, value_name = "PATH")]
        path: Option<String>,
        /// Maximum search depth
        #[arg(long, value_name = "N", default_value = "6")]
        depth: usize,
    },

    /// Add a repository path explicitly
    Add {
        /// Path to the repository
        path: String,
    },

    /// Remove a repository from tracking (does NOT delete files)
    Remove {
        /// Repository name or path
        repository: String,
    },

    /// List all tracked repositories
    #[command(alias = "ls")]
    Repos {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Save a note for a repository's current branch
    Note {
        /// Repository name
        repository: String,
        /// Note text (use "" to clear)
        text: String,
    },

    /// Set the next task for a repository
    Next {
        /// Repository name
        repository: String,
        /// Task description (use "" to clear)
        text: String,
    },

    /// Checkpoint the current state of a repository
    Checkpoint {
        /// Repository name
        repository: String,
    },

    /// Resume work on a repository (shows last session info)
    Resume {
        /// Repository name
        repository: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Print the absolute path of a repository (for scripting)
    Path {
        /// Repository name
        repository: String,
    },

    /// Remove stored session memory for a repository
    Forget {
        /// Repository name
        repository: String,
    },

    /// Manage UheGit configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Print version information
    Version,

    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Show configuration file path
    Path,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Value to set
        value: String,
    },
}
