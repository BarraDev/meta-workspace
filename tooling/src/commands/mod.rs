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

/// Locate an executable on `PATH`, returning the first match.
///
/// On Windows, executables carry an extension (e.g. `npx` is installed as
/// `npx.cmd`), so a bare-name probe misses them. This tries the bare name plus
/// each `PATHEXT` extension. On Unix the bare name is used as-is.
pub(crate) fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for name in exe_candidates(bin) {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(windows)]
fn exe_candidates(bin: &str) -> Vec<String> {
    // A caller-supplied extension is trusted as-is; otherwise probe PATHEXT.
    if std::path::Path::new(bin).extension().is_some() {
        return vec![bin.to_string()];
    }
    let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
    let mut out = vec![bin.to_string()];
    out.extend(
        pathext
            .split(';')
            .filter(|e| !e.is_empty())
            .map(|ext| format!("{bin}{}", ext.to_ascii_lowercase())),
    );
    out
}

#[cfg(not(windows))]
fn exe_candidates(bin: &str) -> Vec<String> {
    vec![bin.to_string()]
}
