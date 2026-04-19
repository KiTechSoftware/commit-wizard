use crate::engine::Result;
pub trait PromptTrait {
    fn text(&self, message: &str, default: Option<&str>, help: Option<&str>) -> Result<String>;
    fn confirm(&self, message: &str, default: bool) -> Result<bool>;
    fn select(&self, request: &scriba::prompt::SelectRequest) -> Result<String>;
    fn multiselect(&self, request: &scriba::prompt::MultiSelectRequest) -> Result<Vec<String>>;
}
