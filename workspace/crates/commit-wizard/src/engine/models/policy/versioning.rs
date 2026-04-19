use crate::engine::models::policy::enforcement::BumpLevel;
use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    /// Parse a semantic version from a tag string with optional prefix.
    /// Tries to parse with prefix first, then without prefix.
    /// Examples: "v1.2.3" with prefix "v" -> Some(Version { 1, 2, 3 })
    ///           "1.2.3" with prefix "v" -> Some(Version { 1, 2, 3 }) (fallback)
    pub fn parse(tag_str: &str, tag_prefix: &str) -> Option<Self> {
        // Try with prefix first
        if let Some(version_part) = tag_str.strip_prefix(tag_prefix)
            && let Some(version) = Self::parse_semver(version_part)
        {
            return Some(version);
        }

        // Fallback: try without prefix
        Self::parse_semver(tag_str)
    }

    /// Parse semantic version from "major.minor.patch" string
    fn parse_semver(version_str: &str) -> Option<Self> {
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse::<u64>().ok()?;
        let minor = parts[1].parse::<u64>().ok()?;
        let patch = parts[2].parse::<u64>().ok()?;

        Some(Version {
            major,
            minor,
            patch,
        })
    }

    /// Format version as a semantic version string with tag prefix.
    /// Examples: Version { 1, 2, 3 } with prefix "v" -> "v1.2.3"
    pub fn format(&self, tag_prefix: &str) -> String {
        format!("{}{}", tag_prefix, self.to_semver())
    }

    /// Format version as semantic version string without prefix.
    /// Examples: Version { 1, 2, 3 } -> "1.2.3"
    pub fn to_semver(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Increment version based on bump level.
    /// - Major: increment major, reset minor and patch
    /// - Minor: increment minor, reset patch
    /// - Patch: increment patch
    /// - None: return current version unchanged
    pub fn with_bump(&self, bump: BumpLevel) -> Self {
        match bump {
            BumpLevel::Major => Version {
                major: self.major + 1,
                minor: 0,
                patch: 0,
            },
            BumpLevel::Minor => Version {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
            },
            BumpLevel::Patch => Version {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
            },
            BumpLevel::None => self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersioningModel {
    pub tag_prefix: String,
    pub initial_version: Version,
}

impl Default for VersioningModel {
    fn default() -> Self {
        Self {
            tag_prefix: "v".to_string(),
            initial_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        }
    }
}

impl VersioningModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        Self {
            tag_prefix: base.versioning_tag_prefix(),
            initial_version: Version {
                major: 0,
                minor: 1,
                patch: 0,
            },
        }
    }
}
