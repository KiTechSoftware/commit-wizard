use crate::ports::config::ConfigPort;
use anyhow::Result;

pub struct FileConfig;

impl ConfigPort for FileConfig {
    fn get(&self, _key: &str) -> Result<Option<String>> {
        Ok(None)
    }
}
