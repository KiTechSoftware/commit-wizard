use crate::ports::editor::EditorPort;
use anyhow::Result;

#[derive(Default)]
pub struct NoopEditor;

impl EditorPort for NoopEditor {
    fn edit(&self, initial: &str) -> Result<String> {
        Ok(initial.to_string())
    }
}
