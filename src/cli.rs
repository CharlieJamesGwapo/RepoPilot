use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
#[derive(Parser)]
#[command(
    version,
    about = "A real shell workspace with local project diagnostics and optional AI chat",
    after_help = "GET STARTED\n  repopilot                              Open your real shell workspace\n  repopilot shell --plain-prompt          Keep your existing prompt\n  repopilot guide \"undo last commit\"     Read an offline workflow\n  repopilot doctor .                     Inspect a project locally\n  repopilot report . --format markdown    Print a shareable report\n  repopilot chat                         Open separate local AI chat\n\nShell commands run normally. Guides and checks preview commands; they do not run them.\nUse repopilot <command> --help for options. Documentation: docs/user-guide.md."
)]
pub struct Cli {
    /// Print structured JSON for commands that support it.
    #[arg(long, global = true)]
    pub json: bool,
    /// Suppress optional output, including the shell welcome screen.
    #[arg(long, short, global = true, conflicts_with = "verbose")]
    pub quiet: bool,
    /// Show additional local diagnostic details.
    #[arg(long, short, global = true)]
    pub verbose: bool,
    /// Explicit configuration file. Plain chat instead uses the local-only user config.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Cmd>,
}
#[derive(Args)]
pub struct ProjectArgs {
    #[arg(default_value = ".")]
    pub path: PathBuf,
}
#[derive(Clone, Copy, ValueEnum)]
pub enum Format {
    Terminal,
    Json,
    Markdown,
}
#[derive(Subcommand)]
pub enum Cmd {
    /// Preview the branded welcome screen without launching a shell.
    Welcome,
    /// Browse offline skills or explicitly export one into a project.
    Skills {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Show curated offline workflows; omit the task to list topics. Executes nothing.
    Guide { task: Vec<String> },
    /// Open your real shell with normal terminal support (also the default).
    #[command(
        after_help = "Interactive Zsh/Bash sessions show the RepoPilot folder and Git prompt.\nYour startup configuration, installed commands and terminal job control remain available.\nType exit to return. --json and --config do not apply to shell mode."
    )]
    Shell {
        /// Keep the user's existing prompt instead of the session-local RepoPilot theme.
        #[arg(long)]
        plain_prompt: bool,
    },
    /// Inventory languages, manifests, frameworks and entry points.
    Scan(ProjectArgs),
    /// Diagnose manifest and environment issues without changing files.
    Doctor(ProjectArgs),
    /// Summarize architecture using manifest and path evidence.
    Explain(ProjectArgs),
    /// Validate manifests and preview quality commands; executes no project code.
    Check(ProjectArgs),
    /// Generate a deterministic project report.
    Report {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value = "terminal")]
        format: Format,
    },
    /// Show or initialize configuration.
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// List available diagnostic rules.
    Rules,
    /// Generate completion definitions.
    Completions { shell: clap_complete::Shell },
    /// Configure or query provider integrations (network only for explicit test/models).
    Providers {
        #[command(subcommand)]
        action: ProviderAction,
    },
    /// Send only the supplied prompt, with explicit transfer/execution consent.
    Ask {
        prompt: String,
        #[command(flatten)]
        options: AiOptions,
    },
    /// Chat using saved user configuration in local-only mode, or an explicit --config.
    Chat {
        #[arg(long)]
        local: bool,
    },
}
#[derive(Subcommand)]
pub enum ConfigAction {
    Init,
}
#[derive(Args, Default)]
pub struct AiOptions {
    #[arg(long)]
    pub provider: Option<String>,
    /// Consent to sending the typed prompt or a probe to the named remote endpoint.
    #[arg(long)]
    pub allow_external: bool,
    /// Consent to invoking the explicitly configured executable with prompt on stdin.
    #[arg(long)]
    pub allow_exec: bool,
    /// Reject all non-loopback endpoints and process adapters.
    #[arg(long)]
    pub local: bool,
}
#[derive(Subcommand)]
pub enum ProviderAction {
    List,
    Add {
        name: String,
        #[arg(long,value_parser=["api","local","process"])]
        kind: String,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        key_env: Option<String>,
        #[arg(long)]
        executable: Option<String>,
        #[arg(long = "arg", allow_hyphen_values = true)]
        args: Vec<String>,
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    Use {
        name: String,
    },
    Remove {
        name: String,
        #[arg(long)]
        yes: bool,
    },
    Test {
        #[command(flatten)]
        options: AiOptions,
    },
    Models {
        #[command(flatten)]
        options: AiOptions,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// List the ten bundled skills; no configuration or network access.
    List {
        #[arg(long, value_enum)]
        pack: Option<SkillPack>,
    },
    /// Print a bundled SKILL.md for review.
    Show { name: String },
    /// Export one skill without overwriting existing files.
    Export {
        name: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value = "portable")]
        target: SkillTarget,
        #[arg(long)]
        dry_run: bool,
    },
}
#[derive(Clone, Copy, ValueEnum)]
pub enum SkillPack {
    Development,
    UiUx,
}
impl SkillPack {
    pub fn label(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::UiUx => "ui-ux",
        }
    }
}
#[derive(Clone, Copy, ValueEnum)]
pub enum SkillTarget {
    Portable,
    Codex,
    Claude,
}
impl SkillTarget {
    pub fn label(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }
    pub fn directory(self) -> &'static str {
        match self {
            Self::Portable => ".repopilot",
            Self::Codex => ".agents",
            Self::Claude => ".claude",
        }
    }
}
