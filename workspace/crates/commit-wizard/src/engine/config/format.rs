//! Config file formats.
//!
//! Config documents can be written in TOML, YAML or JSON. The format is chosen
//! by the file extension, and every format maps onto the same typed structures,
//! so nothing else in the program needs to know which one a file used.

use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};

use crate::engine::error::{ErrorCode, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigFormat {
    #[default]
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    /// Every format, in the order competing files are preferred.
    pub const ALL: [ConfigFormat; 3] = [ConfigFormat::Toml, ConfigFormat::Yaml, ConfigFormat::Json];

    /// Format for a file extension (without the dot), if it is a config format.
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_ascii_lowercase().as_str() {
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// Format of a file, from its extension. Unknown extensions are TOML,
    /// which is what config files have always been.
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|extension| extension.to_str())
            .and_then(Self::from_extension)
            .unwrap_or_default()
    }

    /// Extensions this format is stored under, preferred first.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Toml => &["toml"],
            Self::Yaml => &["yaml", "yml"],
            Self::Json => &["json"],
        }
    }

    pub fn extension(self) -> &'static str {
        self.extensions()[0]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Json => "json",
        }
    }

    /// Parse a document in this format.
    pub fn parse<T: DeserializeOwned>(self, input: &str) -> Result<T> {
        let parsed = match self {
            Self::Toml => toml::from_str(input).map_err(|err| err.to_string()),
            Self::Yaml => yaml_serde::from_str(input).map_err(|err| err.to_string()),
            Self::Json => serde_json::from_str(input).map_err(|err| err.to_string()),
        };
        parsed.map_err(|error| {
            ErrorCode::ConfigInvalid
                .error()
                .with_context("format", self.name())
                .with_context("error", error)
        })
    }

    /// Render a document in this format, ending in a newline.
    pub fn render<T: Serialize>(self, value: &T) -> Result<String> {
        let rendered = match self {
            Self::Toml => toml::to_string_pretty(value).map_err(|err| err.to_string()),
            Self::Yaml => yaml_serde::to_string(value).map_err(|err| err.to_string()),
            Self::Json => serde_json::to_string_pretty(value).map_err(|err| err.to_string()),
        };
        rendered
            .map(|mut text| {
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                text
            })
            .map_err(|error| {
                ErrorCode::SerializationFailure
                    .error()
                    .with_context("format", self.name())
                    .with_context("error", error)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::path::PathBuf;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Doc {
        name: String,
        count: u32,
        nested: Nested,
        list: Vec<String>,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Nested {
        enabled: bool,
    }

    fn doc() -> Doc {
        Doc {
            name: "cw".into(),
            count: 7,
            nested: Nested { enabled: true },
            list: vec!["a".into(), "b".into()],
        }
    }

    #[test]
    fn every_format_round_trips_the_same_document() {
        for format in ConfigFormat::ALL {
            let text = format.render(&doc()).unwrap();
            assert!(text.ends_with('\n'), "{format:?}");
            assert_eq!(format.parse::<Doc>(&text).unwrap(), doc(), "{format:?}");
        }
    }

    #[test]
    fn the_extension_picks_the_format() {
        let cases = [
            (".cwizard.toml", ConfigFormat::Toml),
            (".cwizard.yaml", ConfigFormat::Yaml),
            ("cwizard.YML", ConfigFormat::Yaml),
            ("config.json", ConfigFormat::Json),
            ("config", ConfigFormat::Toml),
            ("notes.txt", ConfigFormat::Toml),
        ];
        for (name, expected) in cases {
            assert_eq!(
                ConfigFormat::from_path(&PathBuf::from(name)),
                expected,
                "{name}"
            );
        }
    }

    #[test]
    fn parse_errors_name_the_format() {
        let err = ConfigFormat::Json.parse::<Doc>("{ not json").unwrap_err();
        assert!(
            err.context
                .iter()
                .any(|(k, v)| k == "format" && v == "json"),
            "{err:?}"
        );
    }

    #[test]
    fn yaml_and_json_accept_hand_written_documents() {
        let yaml = "name: cw\ncount: 7\nnested:\n  enabled: true\nlist: [a, b]\n";
        assert_eq!(ConfigFormat::Yaml.parse::<Doc>(yaml).unwrap(), doc());

        let json = r#"{"name":"cw","count":7,"nested":{"enabled":true},"list":["a","b"]}"#;
        assert_eq!(ConfigFormat::Json.parse::<Doc>(json).unwrap(), doc());
    }
}
