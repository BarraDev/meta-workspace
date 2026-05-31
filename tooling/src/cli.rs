//! Command-line surface for `mw`.
//!
//! This mirrors the command contract in `docs/workspace-contract.md`. Every
//! interactive prompt must have a non-interactive flag equivalent so the tool
//! is automatable. `eject` is intentionally absent (backlog, gated on demand).

use clap::{Args, Parser, Subcommand};

/// The current `workspace.yaml` schema version this binary targets.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Parser)]
#[command(
    name = "mw",
    version,
    about = "Maintain a generic one-company-at-a-time meta-workspace.",
    long_about = "mw is the maintenance CLI for the meta-workspace template.\n\
                  The workspace files require no runtime; mw is only needed to\n\
                  create or maintain a workspace. See docs/workspace-contract.md."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Materialize or repair a workspace from the content template.
    Init(InitArgs),
    /// Validate the workspace against the contract.
    Doctor(DoctorArgs),
    /// Create or repair agent compatibility symlinks and adapters.
    Links(LinksArgs),
    /// Add an entry to projects/registry.yaml.
    AddProject(AddProjectArgs),
    /// Configure the memory profile (none|mempalace|prism|full).
    Memory(MemoryArgs),
    /// Controlled cc-sdd install/update (staged by default).
    Sdd(SddArgs),
    /// Optional, non-blocking session warm-up hook.
    Hook(HookArgs),
    /// Cross-harness policy enforcement engine.
    Policy(PolicyArgs),
    /// Upgrade an older workspace to the current schemaVersion.
    Migrate(MigrateArgs),
}

/// Shared flag for non-interactive automation.
#[derive(Debug, Args, Clone, Copy)]
pub struct CommonFlags {
    /// Never prompt; fail instead of asking. Required for automation.
    #[arg(long, global = true)]
    pub yes: bool,
    /// Describe actions without writing anything.
    #[arg(long, global = true)]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Workspace root to initialize (defaults to the current directory).
    #[arg(long)]
    pub path: Option<String>,
    /// Company identifier to stamp into workspace.yaml.
    #[arg(long)]
    pub company_id: Option<String>,
    /// Human-readable company name.
    #[arg(long)]
    pub company_name: Option<String>,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Exit non-zero on warnings, not just errors.
    #[arg(long)]
    pub strict: bool,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Debug, Args)]
pub struct LinksArgs {
    /// Repair/replace existing links and adapters instead of skipping them.
    #[arg(long)]
    pub force: bool,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Debug, Args)]
pub struct AddProjectArgs {
    /// Project id (slug). Required for non-interactive use.
    #[arg(long)]
    pub id: Option<String>,
    /// Human-readable project name.
    #[arg(long)]
    pub name: Option<String>,
    /// Git remote URL.
    #[arg(long)]
    pub repo_url: Option<String>,
    /// Default branch (e.g. main).
    #[arg(long, default_value = "main")]
    pub default_branch: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

/// Memory profiles supported by the workspace contract.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum MemoryProfile {
    None,
    Mempalace,
    Prism,
    Full,
}

#[derive(Debug, Args)]
pub struct MemoryArgs {
    /// Memory profile to configure. Omit to print the current profile.
    #[arg(long, value_enum)]
    pub profile: Option<MemoryProfile>,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Debug, Args)]
pub struct SddArgs {
    #[command(subcommand)]
    pub action: SddAction,
}

#[derive(Debug, Subcommand)]
pub enum SddAction {
    /// Install cc-sdd (staged in a temp dir by default).
    Install(SddInstallArgs),
    /// Update an existing cc-sdd install.
    Update(SddInstallArgs),
    /// Show the current SDD install state.
    Status,
}

#[derive(Debug, Args)]
pub struct SddInstallArgs {
    /// Install mode: staged review (default) or direct write.
    #[arg(long, value_enum, default_value = "staged")]
    pub mode: SddMode,
    /// Memory document policy: vendor (preserve symlink) or replace.
    #[arg(long, value_enum, default_value = "vendor")]
    pub memory_policy: SddMemoryPolicy,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SddMode {
    Staged,
    Direct,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum SddMemoryPolicy {
    Vendor,
    Replace,
}

#[derive(Debug, Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub event: HookEvent,
}

#[derive(Debug, Subcommand)]
pub enum HookEvent {
    /// Optional, always non-blocking session warm-up.
    SessionStart,
}

#[derive(Debug, Args)]
pub struct PolicyArgs {
    #[command(subcommand)]
    pub action: PolicyAction,
}

#[derive(Debug, Subcommand)]
pub enum PolicyAction {
    /// Read an event as JSON on stdin, return a decision as JSON on stdout.
    Check,
}

#[derive(Debug, Args)]
pub struct MigrateArgs {
    /// Target schema version (defaults to this binary's supported version).
    #[arg(long)]
    pub to: Option<u32>,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// Catches conflicting flags, duplicate args, and malformed subcommands at
    /// test time instead of at runtime.
    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_core_subcommands() {
        assert!(matches!(
            Cli::parse_from(["mw", "doctor", "--strict"]).command,
            Command::Doctor(_)
        ));
        assert!(matches!(
            Cli::parse_from(["mw", "memory", "--profile", "mempalace"]).command,
            Command::Memory(_)
        ));
        assert!(matches!(
            Cli::parse_from(["mw", "policy", "check"]).command,
            Command::Policy(_)
        ));
        assert!(matches!(
            Cli::parse_from(["mw", "hook", "session-start"]).command,
            Command::Hook(_)
        ));
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(Cli::try_parse_from(["mw", "eject"]).is_err());
    }
}
