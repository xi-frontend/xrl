use serde::{Deserialize, Serialize};

use std::path::PathBuf;

/// Enumerates the many possible ways xi-core can be called
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum XiLocation {
    /// Embed's xi-core in a seperate thread. This can be used without having xi-core installed.
    Embeded,
    /// Will launch xi-core as a child process. Takes the file path to the xi-core executable.
    File { path: PathBuf },
    /// Will launch xi-core as a child process passing the `cmd` through the shell.
    Path { cmd: String },
}
