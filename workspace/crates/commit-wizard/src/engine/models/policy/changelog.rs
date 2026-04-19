use crate::engine::models::{policy::enforcement::ChangelogFormat, runtime::ResolvedConfig};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ChangelogHeaderModel {
    pub use_header: bool,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChangelogSectionModel {
    pub title: Option<String>,
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangelogOutputFormat {
    Markdown,
    Json,
}

impl From<ChangelogFormat> for ChangelogOutputFormat {
    fn from(value: ChangelogFormat) -> Self {
        match value {
            ChangelogFormat::Markdown => Self::Markdown,
            ChangelogFormat::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChangelogModel {
    pub output: String,
    pub format: ChangelogOutputFormat,
    pub header: ChangelogHeaderModel,
    pub group_by: Vec<String>,
    pub section_order: Vec<String>,
    pub scope_order: Vec<String>,
    pub show_scope: bool,
    pub show_empty_sections: Option<bool>,
    pub show_empty_scopes: Option<bool>,
    pub misc_section: Option<String>,
    pub unreleased_label: String,
    pub date_format: Option<String>,
    pub sections: BTreeMap<String, ChangelogSectionModel>,
}

impl Default for ChangelogModel {
    fn default() -> Self {
        Self {
            output: "CHANGELOG.md".to_string(),
            format: ChangelogOutputFormat::Markdown,
            header: ChangelogHeaderModel {
                use_header: true,
                title: "Changelog".to_string(),
                description: None,
            },
            group_by: vec!["type".to_string()],
            section_order: vec![
                "feat".to_string(),
                "fix".to_string(),
                "docs".to_string(),
                "style".to_string(),
                "refactor".to_string(),
                "perf".to_string(),
                "test".to_string(),
                "chore".to_string(),
            ],
            scope_order: Vec::new(),
            show_scope: true,
            show_empty_sections: Some(false),
            show_empty_scopes: Some(false),
            misc_section: Some("Miscellaneous".to_string()),
            unreleased_label: "Unreleased".to_string(),
            date_format: None,
            sections: BTreeMap::new(),
        }
    }
}

impl ChangelogModel {
    pub fn from_config(config: &ResolvedConfig) -> Self {
        let base = &config.base;

        // Load sections from config, converting ChangelogSectionConfig to ChangelogSectionModel
        let sections_from_config = base.changelog_sections();
        let sections = sections_from_config
            .into_iter()
            .map(|(key, cfg)| {
                (
                    key,
                    ChangelogSectionModel {
                        title: cfg.title,
                        order: cfg.order,
                    },
                )
            })
            .collect();

        Self {
            output: base.changelog_output(),
            format: base.changelog_format().into(),
            header: ChangelogHeaderModel {
                use_header: base.changelog_header_use(),
                title: base.changelog_header_title(),
                description: base.changelog_header_description(),
            },
            group_by: base.changelog_group_by(),
            section_order: base.changelog_section_order(),
            scope_order: base.changelog_scope_order(),
            show_scope: base.changelog_show_scope(),
            show_empty_sections: base.changelog_show_empty_sections(),
            show_empty_scopes: base.changelog_show_empty_scopes(),
            misc_section: base.changelog_misc_section(),
            unreleased_label: base.changelog_unreleased_label(),
            date_format: base.changelog_date_format(),
            sections,
        }
    }
}

pub fn section_for_type<'a>(sections: &'a BTreeMap<String, Vec<String>>, typ: &'a str) -> &'a str {
    for (section, types) in sections {
        if types.iter().any(|t| t == typ) {
            return section;
        }
    }
    "Uncategorized"
}
