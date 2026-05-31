//! `mw` — thin entry point over the `meta_workspace` library.
//!
//! All logic lives in the library crate so it can be unit-, integration-, and
//! doc-tested. See `docs/workspace-contract.md`.

use clap::Parser;
use meta_workspace::cli::Cli;

fn main() -> anyhow::Result<()> {
    meta_workspace::run(Cli::parse())
}
