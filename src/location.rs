use serde::{Deserialize, Serialize};

/// Enumerates the possible ways xi-core can be called. This is only used by the `Client` struct but
/// is left outside the client module to simplify implementing new clients.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum XiLocation {
    /// Embed's xi-core in a seperate thread. This can be used without having xi-core installed.
    Embeded,
    /// Will launch xi-core as a child process passing the `cmd` through the shell.
    Path { cmd: String },
}
