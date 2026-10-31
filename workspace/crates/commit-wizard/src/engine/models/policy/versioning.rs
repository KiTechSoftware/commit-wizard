use std::cmp::Ordering;

use crate::engine::models::policy::enforcement::BumpLevel;
use crate::engine::models::runtime::ResolvedConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    /// Prerelease identifier without the leading hyphen (e.g. "rc.1").
    pub pre: Option<String>,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                // A release outranks any of its prereleases.
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(a), Some(b)) => cmp_prerelease(a, b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compare prerelease identifiers per SemVer 2.0: numeric identifiers compare
/// numerically and sort below alphanumeric ones; a shorter list sorts first.
fn cmp_prerelease(a: &str, b: &str) -> Ordering {
    let mut a_parts = a.split('.');
    let mut b_parts = b.split('.');
    loop {
        match (a_parts.next(), b_parts.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let ord = match (x.parse::<u64>(), y.parse::<u64>()) {
                    (Ok(x), Ok(y)) => x.cmp(&y),
                    (Ok(_), Err(_)) => Ordering::Less,
                    (Err(_), Ok(_)) => Ordering::Greater,
                    (Err(_), Err(_)) => x.cmp(y),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
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

    /// Parse semantic version from "major.minor.patch[-prerelease][+build]" string.
    /// Build metadata is ignored.
    fn parse_semver(version_str: &str) -> Option<Self> {
        let without_build = version_str.split('+').next()?;
        let (core, pre) = match without_build.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (without_build, None),
        };

        if let Some(pre) = pre
            && (pre.is_empty()
                || pre.split('.').any(|id| {
                    id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                }))
        {
            return None;
        }

        let parts: Vec<&str> = core.split('.').collect();

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
            pre: pre.map(str::to_string),
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
        match &self.pre {
            Some(pre) => format!("{}.{}.{}-{}", self.major, self.minor, self.patch, pre),
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
        }
    }

    /// Whether this version is a prerelease (e.g. `1.2.3-rc.1`).
    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }

    /// The release this version leads up to (`1.2.3-rc.1` -> `1.2.3`).
    pub fn release(&self) -> Self {
        Version {
            pre: None,
            ..self.clone()
        }
    }

    /// Attach a prerelease label. A bare label such as `rc` is numbered one
    /// past the highest matching `rc.N` in `existing`; a label that already
    /// carries identifiers (`rc.2`, `beta.1`) is used as given.
    pub fn with_prerelease(&self, label: &str, existing: &[Version]) -> Self {
        let label = label.trim_start_matches('-');
        let pre = if label.contains('.') {
            label.to_string()
        } else {
            let next = existing
                .iter()
                .filter(|v| v.release() == self.release())
                .filter_map(|v| {
                    v.pre
                        .as_deref()?
                        .strip_prefix(label)?
                        .strip_prefix('.')?
                        .parse::<u64>()
                        .ok()
                })
                .max()
                .map_or(1, |n| n + 1);
            format!("{label}.{next}")
        };
        Version {
            pre: Some(pre),
            ..self.release()
        }
    }

    /// Increment version based on bump level.
    /// - Major: increment major, reset minor and patch
    /// - Minor: increment minor, reset patch
    /// - Patch: increment patch
    /// - None: return current version unchanged
    ///
    /// A prerelease is promoted to its release instead (`1.2.0-rc.2` -> `1.2.0`),
    /// since the prerelease already carries the bump.
    pub fn with_bump(&self, bump: BumpLevel) -> Self {
        if self.is_prerelease() {
            return self.release();
        }
        match bump {
            BumpLevel::Major => Version {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                pre: None,
            },
            BumpLevel::Minor => Version {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                pre: None,
            },
            BumpLevel::Patch => Version {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                pre: None,
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
                pre: None,
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
                pre: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s, "v").expect("valid version")
    }

    #[test]
    fn parses_and_formats_prereleases() {
        assert_eq!(v("v2.0.0-rc.1").pre.as_deref(), Some("rc.1"));
        assert_eq!(v("2.0.0-rc.1").to_semver(), "2.0.0-rc.1");
        assert_eq!(v("v1.2.3+build.5").to_semver(), "1.2.3");
        assert!(Version::parse("v1.2.3-", "v").is_none());
        assert!(Version::parse("v1.2.3rc.2", "v").is_none());
    }

    #[test]
    fn orders_per_semver() {
        assert!(v("1.0.0-rc.1") < v("1.0.0-rc.2"));
        assert!(v("1.0.0-rc.2") < v("1.0.0-rc.10"));
        assert!(v("1.0.0-alpha") < v("1.0.0-beta"));
        assert!(v("1.0.0-rc.1") < v("1.0.0"));
        assert!(v("1.1.0") > v("1.0.0"));
    }

    #[test]
    fn bumps_release_levels() {
        assert_eq!(v("1.2.3").with_bump(BumpLevel::Patch).to_semver(), "1.2.4");
        assert_eq!(v("1.2.3").with_bump(BumpLevel::Minor).to_semver(), "1.3.0");
        assert_eq!(v("1.2.3").with_bump(BumpLevel::Major).to_semver(), "2.0.0");
        assert_eq!(v("1.2.3").with_bump(BumpLevel::None).to_semver(), "1.2.3");
    }

    #[test]
    fn bumping_a_prerelease_promotes_it() {
        assert_eq!(
            v("2.0.0-rc.2").with_bump(BumpLevel::Patch).to_semver(),
            "2.0.0"
        );
        assert_eq!(
            v("2.0.0-rc.2").with_bump(BumpLevel::None).to_semver(),
            "2.0.0"
        );
    }

    #[test]
    fn numbers_bare_labels_past_existing_prereleases() {
        let base = v("2.0.0");
        assert_eq!(base.with_prerelease("rc", &[]).to_semver(), "2.0.0-rc.1");
        let existing = [v("2.0.0-rc.1"), v("2.0.0-rc.2"), v("1.9.0-rc.7")];
        assert_eq!(
            base.with_prerelease("rc", &existing).to_semver(),
            "2.0.0-rc.3"
        );
        assert_eq!(
            base.with_prerelease("-beta.4", &existing).to_semver(),
            "2.0.0-beta.4"
        );
    }
}
