//! Command implementations for `mw`.
//!
//! These commands are the source of truth for maintaining a workspace; the
//! interim bash/python scripts were retired after parity was reached.

pub mod add_project;
pub mod doctor;
pub mod hook;
pub mod init;
pub mod links;
pub mod memory;
pub mod migrate;
pub mod policy;
pub mod sdd;
