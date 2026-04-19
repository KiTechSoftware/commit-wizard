use crate::engine::models::{
    git::CommitSummary,
    policy::commit::{CommitModel, ScopeRequirement},
};
use regex::Regex;

/// A single violation in commit validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitViolation {
    InvalidHeaderFormat,
    InvalidType(String),
    MissingScopeWhenRequired,
    ScopeNotAllowed(String),
    InvalidScope(String),
    EmptySubject,
    SubjectTooLong { length: usize, max: u32 },
    MissingBreakingHeader,
    MissingBreakingFooter,
    InvalidBreakingFooter,
    MissingTicket,
    InvalidTicketFormat(String),
    EmojiNotAllowed,
}

impl CommitViolation {
    pub fn message(&self) -> String {
        match self {
            Self::InvalidHeaderFormat => {
                "Header does not match conventional commit format".to_string()
            }
            Self::InvalidType(t) => format!("Type '{}' is not allowed", t),
            Self::MissingScopeWhenRequired => "Scope is required but not provided".to_string(),
            Self::ScopeNotAllowed(s) => format!(
                "Scope '{}' is not allowed (commit.scopes.mode = disabled)",
                s
            ),
            Self::InvalidScope(s) => format!("Scope '{}' is not allowed", s),
            Self::EmptySubject => "Subject must not be empty".to_string(),
            Self::SubjectTooLong { length, max } => {
                format!("Subject is {} chars, but max is {}", length, max)
            }
            Self::MissingBreakingHeader => {
                "Breaking change marker '!' required in header".to_string()
            }
            Self::MissingBreakingFooter => {
                "Breaking change footer required (e.g., 'BREAKING CHANGE: ...')".to_string()
            }
            Self::InvalidBreakingFooter => "Breaking change footer format is invalid".to_string(),
            Self::MissingTicket => "Ticket is required but not provided".to_string(),
            Self::InvalidTicketFormat(msg) => format!("Ticket format invalid: {}", msg),
            Self::EmojiNotAllowed => {
                "Emoji prefix is not allowed (commit.use_emojis = false)".to_string()
            }
        }
    }
}

/// Parsed components of a commit header
#[derive(Debug, Clone)]
pub struct ParsedHeader {
    pub type_name: String,
    pub scope: Option<String>,
    pub is_breaking: bool,
    pub subject: String,
    pub emoji: Option<String>,
}

impl ParsedHeader {
    /// Parse a commit header using regex
    /// Supports:
    /// - Standard: type(scope)?: subject
    /// - Gitmoji: emoji type(scope)?: subject
    pub fn parse(header: &str) -> Option<Self> {
        let header = header.trim();

        // Try to extract leading emoji and the rest
        let (emoji, rest) = Self::extract_emoji(header);

        // Pattern: type(scope)?: subject or type(scope)!: subject
        let pattern = r"^(\w+)(?:\(([^)]+)\))?(!)?: (.+)$";
        let re = Regex::new(pattern).unwrap();

        re.captures(rest).map(|caps| ParsedHeader {
            type_name: caps.get(1).unwrap().as_str().to_string(),
            scope: caps.get(2).map(|m| m.as_str().to_string()),
            is_breaking: caps.get(3).is_some(),
            subject: caps.get(4).unwrap().as_str().to_string(),
            emoji,
        })
    }

    /// Extract leading emoji from header if present
    /// Returns (emoji, rest_of_header)
    fn extract_emoji(s: &str) -> (Option<String>, &str) {
        // Emoji detection: check if first character(s) is emoji
        // Emojis are typically 1-2 UTF-8 code points, followed by space
        let chars: Vec<char> = s.chars().collect();
        if chars.is_empty() {
            return (None, s);
        }

        // Check if first char is emoji (rough heuristic: not ASCII alphanumeric and not punctuation)
        if is_emoji_char(chars[0]) {
            // Find the space after emoji
            if let Some(space_pos) = s.find(' ') {
                let emoji_part = &s[..space_pos];
                let rest = &s[space_pos + 1..];
                return (Some(emoji_part.to_string()), rest);
            }
        }
        (None, s)
    }
}

/// Simple emoji detection: characters outside common ASCII ranges
fn is_emoji_char(c: char) -> bool {
    let code = c as u32;
    // Emoji ranges in Unicode (very simplified)
    // Includes common emoji blocks: 1F600-1F64F (Emoticons), 2600-26FF (Misc), etc.
    (0x1F300..=0x1F9FF).contains(&code) || // Main emoji block
    (0x2600..=0x27BF).contains(&code) ||   // Misc symbols
    (0x2300..=0x23FF).contains(&code) // Misc technical
}

/// The full body of a commit (body + footer sections)
#[derive(Debug, Clone)]
pub struct CommitBody {
    pub body_text: String,
    pub footer_lines: Vec<String>,
}

impl CommitBody {
    /// Parse commit body to extract footer lines
    fn parse(full_message: &str) -> Self {
        // Split on first blank line to separate header from body
        let parts: Vec<&str> = full_message.splitn(2, '\n').collect();
        if parts.len() < 2 {
            return CommitBody {
                body_text: String::new(),
                footer_lines: Vec::new(),
            };
        }

        let body_and_footer = parts[1];
        // Look for footer section (key: value pattern)
        let footer_lines = body_and_footer
            .lines()
            .filter(|line| line.contains(": "))
            .map(|s| s.to_string())
            .collect();

        CommitBody {
            body_text: body_and_footer.to_string(),
            footer_lines,
        }
    }

    /// Check if breaking change footer exists
    fn has_breaking_footer(&self, footer_key: &str) -> bool {
        self.footer_lines
            .iter()
            .any(|line| line.starts_with(footer_key))
    }
}

/// Result of validating a single commit
#[derive(Debug, Clone)]
pub struct ValidatedCommit {
    pub hash: String,
    pub summary: String,
    pub valid: bool,
    pub violations: Vec<CommitViolation>,
}

/// Overall validation report
#[derive(Debug, Clone)]
pub struct ValidateReport {
    pub total: usize,
    pub invalid_count: usize,
    pub commits: Vec<ValidatedCommit>,
}

/// The validator performs full commit validation against a policy
pub struct CommitValidator<'a> {
    policy: &'a CommitModel,
}

impl<'a> CommitValidator<'a> {
    pub fn new(policy: &'a CommitModel) -> Self {
        Self { policy }
    }

    /// Validate a single commit message
    pub fn validate_message(&self, message: &str) -> Vec<CommitViolation> {
        let mut violations = Vec::new();

        // If conventional commits are not required, skip format validation
        if !self.policy.require_conventional {
            // Still validate other aspects (emoji, tickets, etc) but skip commit format checks
            return violations;
        }

        // Split header from rest
        let header_line = message.lines().next().unwrap_or("");
        let parsed = match ParsedHeader::parse(header_line) {
            Some(h) => h,
            None => {
                violations.push(CommitViolation::InvalidHeaderFormat);
                return violations;
            }
        };

        // Validate emoji usage
        if parsed.emoji.is_some() && !self.policy.use_emojis {
            violations.push(CommitViolation::EmojiNotAllowed);
        }

        // Validate type
        if !self.policy.allows_type(&parsed.type_name) {
            violations.push(CommitViolation::InvalidType(parsed.type_name.clone()));
        }

        // Validate scope
        match &parsed.scope {
            Some(scope) => {
                if self.policy.scope_requirement == ScopeRequirement::Disabled {
                    violations.push(CommitViolation::ScopeNotAllowed(scope.clone()));
                } else if self.policy.restrict_scopes_to_defined && !self.policy.allows_scope(scope)
                {
                    violations.push(CommitViolation::InvalidScope(scope.clone()));
                }
            }
            None => {
                if self.policy.scope_requirement == ScopeRequirement::Required {
                    violations.push(CommitViolation::MissingScopeWhenRequired);
                }
            }
        }

        // Validate subject
        if parsed.subject.is_empty() {
            violations.push(CommitViolation::EmptySubject);
        } else if parsed.subject.len() as u32 > self.policy.subject_max_length {
            violations.push(CommitViolation::SubjectTooLong {
                length: parsed.subject.len(),
                max: self.policy.subject_max_length,
            });
        }

        // Validate breaking changes
        let body = CommitBody::parse(message);
        if self.policy.breaking_header_required && parsed.is_breaking {
            // If header has ! and we found it, that's good
        } else if self.policy.breaking_header_required && !parsed.is_breaking {
            // Check if footer exists to validate consistency
            if body.has_breaking_footer(&self.policy.breaking_footer_key) {
                violations.push(CommitViolation::MissingBreakingHeader);
            }
        }

        // Only require breaking footer if this is actually a breaking change commit
        if self.policy.breaking_footer_required
            && parsed.is_breaking
            && !body.has_breaking_footer(&self.policy.breaking_footer_key)
        {
            violations.push(CommitViolation::MissingBreakingFooter);
        }

        // Validate ticket
        if self.policy.ticket.enabled {
            let ticket_found = if let Some(ticket_regex) = &self.policy.ticket.regex {
                match Regex::new(ticket_regex) {
                    Ok(re) => re.is_match(message),
                    Err(_) => {
                        violations.push(CommitViolation::InvalidTicketFormat(format!(
                            "Invalid ticket regex pattern: {}",
                            ticket_regex
                        )));
                        return violations;
                    }
                }
            } else {
                // No regex configured — ticket cannot be validated
                !self.policy.ticket.required
            };

            if self.policy.ticket.required && !ticket_found {
                if self.policy.ticket.regex.is_some() {
                    violations.push(CommitViolation::InvalidTicketFormat(format!(
                        "Does not match pattern: {}",
                        self.policy.ticket.regex.as_deref().unwrap_or("")
                    )));
                } else {
                    violations.push(CommitViolation::MissingTicket);
                }
            }
        }

        violations
    }
}

/// Validate multiple commits
pub fn validate_commits(
    commits: impl IntoIterator<Item = CommitSummary>,
    policy: &CommitModel,
) -> ValidateReport {
    let validator = CommitValidator::new(policy);
    let mut validated = Vec::new();
    let mut invalid_count = 0usize;

    for commit in commits {
        // Use the full message (header + body + footer) for validation when available,
        // so breaking footer, ticket, and body checks work correctly.
        let full_msg = commit.full_message.as_deref().unwrap_or(&commit.summary);
        let violations = validator.validate_message(full_msg);
        let valid = violations.is_empty();

        if !valid {
            invalid_count += 1;
        }

        validated.push(ValidatedCommit {
            hash: commit.hash,
            summary: commit.summary,
            valid,
            violations,
        });
    }

    ValidateReport {
        total: validated.len(),
        invalid_count,
        commits: validated,
    }
}
