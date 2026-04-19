use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RunMode {
    #[default]
    Interactive,
    NonInteractive,
    Ci,
}

impl RunMode {
    pub fn from_flags(ci: bool, non_interactive: bool) -> Self {
        if ci {
            Self::Ci
        } else if non_interactive {
            Self::NonInteractive
        } else {
            Self::Interactive
        }
    }

    pub fn allows_prompts(self) -> bool {
        matches!(self, Self::Interactive)
    }

    pub fn is_strict(self) -> bool {
        matches!(self, Self::Ci | Self::NonInteractive)
    }
}
