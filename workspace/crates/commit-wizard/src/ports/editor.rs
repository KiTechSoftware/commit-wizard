use anyhow::Result;
pub trait EditorPort {
    fn edit(&self, initial: &str) -> Result<String>;
}
