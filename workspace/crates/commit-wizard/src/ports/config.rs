use anyhow::Result;
pub trait ConfigPort {
    fn get(&self, key: &str) -> Result<Option<String>>;
}
