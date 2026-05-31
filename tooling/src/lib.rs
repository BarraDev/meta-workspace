//! `meta_workspace` — library core behind the `mw` binary.
//!
//! The workspace **content layer** (files, symlinks, git) requires no runtime.
//! This crate is the **tooling layer**: it is only needed to create or maintain
//! a workspace, never to "use" one. See `docs/workspace-contract.md`.
//!
//! Keeping the logic in a library (with a thin `mw` binary on top) lets us test
//! it three ways: in-module unit tests, library/integration tests against the
//! public API, and doc tests on the documented helpers.

pub mod cli;
pub mod commands;
pub mod embed;
pub mod links;
pub mod registry;
pub mod sdd;
pub mod workspace;

use cli::{Cli, Command};

/// Dispatch a parsed CLI invocation to its command handler.
pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Doctor(args) => commands::doctor::run(args),
        Command::Links(args) => commands::links::run(args),
        Command::AddProject(args) => commands::add_project::run(args),
        Command::Memory(args) => commands::memory::run(args),
        Command::Sdd(args) => commands::sdd::run(args),
        Command::Hook(args) => commands::hook::run(args),
        Command::Policy(args) => commands::policy::run(args),
        Command::Migrate(args) => commands::migrate::run(args),
    }
}
