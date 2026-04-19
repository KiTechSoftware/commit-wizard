use std::fmt;

use serde::{Deserialize, Serialize};

use crate::engine::error::ErrorKind;

pub mod exit_status {
    // User input errors
    pub const USER: i32 = 2;
    // Configuration/state errors (10-19)
    pub const CONFIG: i32 = 10;
    pub const STATE: i32 = 11;
    // External integration errors (20-29)
    pub const REGISTRY: i32 = 20;
    pub const GIT: i32 = 21;
    pub const IO: i32 = 25;
    pub const PROCESS: i32 = 26;
    // Validation/workflow errors (30-39)
    pub const VALIDATION: i32 = 30;
    pub const RELEASE: i32 = 31;
    pub const HOOK: i32 = 32;
    pub const AI: i32 = 33;
    // Runtime errors (50+)
    pub const RUNTIME: i32 = 50;
}

macro_rules! error_codes {
    (
        $(
            $variant:ident => ($kind:ident, $num:expr, $msg:expr)
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ErrorCode {
            $($variant),*
        }

        impl ErrorCode {
            pub const fn kind(self) -> ErrorKind {
                match self {
                    $(Self::$variant => ErrorKind::$kind),*
                }
            }

            pub const fn message(self) -> &'static str {
                match self {
                    $(Self::$variant => $msg),*
                }
            }

            pub const fn index(self) -> u16 {
                match self {
                    $(Self::$variant => $num),*
                }
            }

            pub const fn numeric(self) -> u16 {
                self.kind().prefix() * 1000 + self.index()
            }

            pub fn id(self) -> String {
                format!("E{:04}", self.numeric())
            }

            pub const fn exit_code(self) -> i32 {
                match self.kind() {
                    // User input errors
                    ErrorKind::User => exit_status::USER,
                    // Configuration/state errors (10-19)
                    ErrorKind::Config => exit_status::CONFIG,
                    ErrorKind::State => exit_status::STATE,
                    // External integration errors (20-29)
                    ErrorKind::Registry => exit_status::REGISTRY,
                    ErrorKind::Git => exit_status::GIT,
                    ErrorKind::Io => exit_status::IO,
                    ErrorKind::Process => exit_status::PROCESS,
                    // Validation/workflow errors (30-39)
                    ErrorKind::Validation => exit_status::VALIDATION,
                    ErrorKind::Release => exit_status::RELEASE,
                    ErrorKind::Hook => exit_status::HOOK,
                    ErrorKind::Ai => exit_status::AI,
                    // Runtime errors (50+)
                    ErrorKind::Runtime => exit_status::RUNTIME,
                }
            }
        }

        impl fmt::Display for ErrorCode {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{} ({})", self.message(), self.id())
            }
        }
    };
}

error_codes! {
    UserCancelled => (User, 1, "Operation cancelled"),
    UserInteractionRequired => (User, 2, "Interactive input required"),
    UserInvalidArgument => (User, 3, "Invalid argument"),
    FormatUnsupported => (User, 4, "Format is not supported for this command"),
    InvalidInput => (User, 5, "Input is invalid"),

    ConfigInvalid => (Config, 1, "Configuration is invalid"),
    ConfigUnreadable => (Config, 2, "Configuration is unreadable"),
    ConfigReferenceInvalid => (Config, 3, "Configuration reference is invalid"),

    StateInvalid => (State, 1, "State is invalid"),
    StateUnreadable => (State, 2, "State is unreadable"),

    RegistryUnknown => (Registry, 1, "Registry is unknown"),
    RegistryInvalid => (Registry, 2, "Registry is invalid"),
    RegistryUnreachable => (Registry, 3, "Registry is unreachable"),
    RegistrySyncFailed => (Registry, 4, "Registry sync failed"),
    RegistryRefNotFound => (Registry, 5, "Registry ref not found"),
    RegistrySectionMissing => (Registry, 6, "Registry section is missing"),
    RegistryOffline => (Registry, 7, "Registry unavailable in offline mode"),

    GitMissing => (Git, 1, "git is missing"),
    GitRepoNotFound => (Git, 2, "Git repository not found"),
    GitCommandFailed => (Git, 3, "Git command failed"),
    GitPushBlocked => (Git, 4, "Push blocked by policy"),

    ValidationFailed => (Validation, 1, "Validation failed"),
    CommitInvalid => (Validation, 2, "Commit is invalid"),
    PrInvalid => (Validation, 3, "Pull request is invalid"),
    BranchInvalid => (Validation, 4, "Branch is invalid"),

    ReleaseInvalid => (Release, 1, "Release is invalid"),
    ReleaseTagExists => (Release, 2, "Release tag already exists"),
    ReleaseBranchExists => (Release, 3, "Release branch already exists"),

    HookInstallFailed => (Hook, 1, "Hook installation failed"),
    HookExecutionFailed => (Hook, 2, "Hook execution failed"),

    AiUnsupported => (Ai, 1, "AI is not supported for this command"),
    AiProviderFailed => (Ai, 2, "AI provider failed"),

    IoFailure => (Io, 1, "I/O failure"),
    SerializationFailure => (Io, 2, "Serialization failure"),
    UiPromptFailed => (Io, 3, "Prompt failed"),
    OutputRenderFailure => (Io, 4, "Output render failure"),

    ProcessFailure => (Process, 1, "Process failure"),

    RuntimeFailure => (Runtime, 1, "Runtime failure"),
}
