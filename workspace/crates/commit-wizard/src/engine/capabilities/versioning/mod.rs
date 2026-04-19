use crate::engine::capabilities::commit::check::ParsedHeader;
use crate::engine::models::{
    git::CommitSummary,
    policy::{commit::CommitModel, enforcement::BumpLevel, versioning::Version},
};

/// Classify commits into their bump levels based on commit type and policy.
/// Implements the versioning algorithm from SRS §5.2:
/// - breaking (has ! or BREAKING CHANGE:) → major
/// - feat → minor
/// - others → patch
pub fn classify_commits(
    commits: &[CommitSummary],
    policy: &CommitModel,
) -> Vec<(CommitSummary, BumpLevel)> {
    commits
        .iter()
        .map(|commit| {
            let bump = classify_single_commit(commit, policy);
            (commit.clone(), bump)
        })
        .collect()
}

/// Classify a single commit into its bump level.
fn classify_single_commit(commit: &CommitSummary, policy: &CommitModel) -> BumpLevel {
    // Parse the commit header
    let header = &commit.summary;

    // Try to parse as conventional commit
    if let Some(parsed) = ParsedHeader::parse(header) {
        // Check for breaking changes in body/footer
        let is_breaking_footer = commit
            .full_message
            .as_ref()
            .map(|msg| msg.contains(&format!("{}:", policy.breaking_footer_key)))
            .unwrap_or(false);

        if parsed.is_breaking || is_breaking_footer {
            return BumpLevel::Major;
        }

        // Find the commit type and get its bump level
        if let Some(type_model) = policy.find_type(&parsed.type_name) {
            return type_model.bump;
        }

        // Type not found - return default
        return BumpLevel::None;
    }

    // Parsing failed - return None
    BumpLevel::None
}

/// Calculate the next version based on commits since last tag.
///
/// Algorithm (SRS §5.3):
/// 1. Get current version from last tag
/// 2. Classify commits to find highest bump
/// 3. Increment version using highest bump
/// 4. Handle edge cases: no tag → use initial version (0.1.0)
pub fn calculate_next_version(
    current_version: Option<Version>,
    classified_commits: &[(CommitSummary, BumpLevel)],
) -> Version {
    // Determine the highest bump level across all commits
    let highest_bump = classified_commits
        .iter()
        .map(|(_, bump)| bump)
        .max_by_key(|bump| match bump {
            BumpLevel::Major => 3,
            BumpLevel::Minor => 2,
            BumpLevel::Patch => 1,
            BumpLevel::None => 0,
        })
        .copied()
        .unwrap_or(BumpLevel::None);

    // Use current version if available, otherwise start from initial (0.1.0)
    let base_version = current_version.unwrap_or(Version {
        major: 0,
        minor: 1,
        patch: 0,
    });

    // Apply bump to the base version
    base_version.with_bump(highest_bump)
}
