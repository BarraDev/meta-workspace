//! Command implementations for `mw`.
//!
//! Phase 2 ships compiling, contract-shaped stubs. Each command documents the
//! behavior it will gain when its interim bash+python script is retired
//! (see docs/workspace-contract.md and the project HANDOFF).

pub mod add_project;
pub mod doctor;
pub mod hook;
pub mod init;
pub mod links;
pub mod memory;
pub mod migrate;
pub mod policy;
pub mod sdd;
