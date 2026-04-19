use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ErrorKind {
    User,
    Config,
    State,
    Registry,
    Git,
    Validation,
    Release,
    Hook,
    Ai,
    Io,
    Process,
    Runtime,
}

impl ErrorKind {
    pub const fn prefix(self) -> u16 {
        match self {
            Self::User => 0,
            Self::Config => 1,
            Self::State => 2,
            Self::Registry => 3,
            Self::Git => 4,
            Self::Validation => 5,
            Self::Release => 6,
            Self::Hook => 7,
            Self::Ai => 8,
            Self::Io => 9,
            Self::Process => 10,
            Self::Runtime => 11,
        }
    }
}
