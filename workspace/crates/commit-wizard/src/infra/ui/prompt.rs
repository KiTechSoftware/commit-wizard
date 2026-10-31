use crate::{
    engine::{Error, ErrorCode, PromptTrait},
    infra::ui::{PromptPolicy, UiResult, map_scriba_error_to_cw_error},
};

impl super::Ui {
    /// The error for a question that cannot be asked or defaulted.
    fn unanswerable(&self, message: &str) -> Error {
        let hint = match self.prompt_policy {
            PromptPolicy::Defaults => {
                "this question has no default to accept; pass the value as a flag"
            }
            _ => "run on a terminal, or pass the value as a flag (see --help)",
        };
        ErrorCode::UserInteractionRequired
            .error()
            .with_context("prompt", message)
            .with_context("hint", hint)
    }
}

impl PromptTrait for super::Ui {
    fn text(&self, message: &str, default: Option<&str>, help: Option<&str>) -> UiResult<String> {
        match self.prompt_policy {
            PromptPolicy::Ask => {
                scriba::prompt::text(&self.config, message, default, help, &self.prompt_theme)
                    .map_err(map_scriba_error_to_cw_error)
            }
            PromptPolicy::Defaults => default
                .map(str::to_string)
                .ok_or_else(|| self.unanswerable(message)),
            PromptPolicy::Refuse => Err(self.unanswerable(message)),
        }
    }

    fn confirm(&self, message: &str, default: bool) -> UiResult<bool> {
        match self.prompt_policy {
            PromptPolicy::Ask => {
                scriba::prompt::confirm(&self.config, message, default, &self.prompt_theme)
                    .map_err(map_scriba_error_to_cw_error)
            }
            PromptPolicy::Defaults => Ok(default),
            PromptPolicy::Refuse => Err(self.unanswerable(message)),
        }
    }

    fn select(&self, request: &scriba::prompt::SelectRequest) -> UiResult<String> {
        match self.prompt_policy {
            PromptPolicy::Ask => scriba::prompt::select(&self.config, request, &self.prompt_theme)
                .map_err(map_scriba_error_to_cw_error),
            // A choice has no default: never pick one on the user's behalf.
            PromptPolicy::Defaults | PromptPolicy::Refuse => {
                Err(self.unanswerable(&request.message))
            }
        }
    }

    fn multiselect(&self, request: &scriba::prompt::MultiSelectRequest) -> UiResult<Vec<String>> {
        match self.prompt_policy {
            PromptPolicy::Ask => {
                scriba::prompt::multiselect(&self.config, request, &self.prompt_theme)
                    .map_err(map_scriba_error_to_cw_error)
            }
            PromptPolicy::Defaults | PromptPolicy::Refuse => {
                Err(self.unanswerable(&request.message))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ErrorCode;
    use scriba::prompt::{SelectOption, SelectRequest};

    fn ui(policy: PromptPolicy) -> super::super::Ui {
        super::super::Ui::new().with_prompt_policy(policy)
    }

    fn asks_for_input(result: UiResult<impl std::fmt::Debug>) -> bool {
        matches!(result, Err(ref e) if e.code == ErrorCode::UserInteractionRequired)
    }

    #[test]
    fn defaults_policy_accepts_the_default_where_there_is_one() {
        let ui = ui(PromptPolicy::Defaults);
        assert!(ui.confirm("Continue?", true).unwrap());
        assert!(!ui.confirm("Continue?", false).unwrap());
        assert_eq!(ui.text("Remote", Some("origin"), None).unwrap(), "origin");
    }

    #[test]
    fn defaults_policy_never_invents_an_answer() {
        let ui = ui(PromptPolicy::Defaults);
        let choose = SelectRequest::new("Pick", vec![SelectOption::new("a", "A")]);
        assert!(asks_for_input(ui.text("Summary", None, None)));
        assert!(asks_for_input(ui.select(&choose)));
    }

    #[test]
    fn refuse_policy_fails_every_question_even_ones_with_defaults() {
        let ui = ui(PromptPolicy::Refuse);
        let choose = SelectRequest::new("Pick", vec![SelectOption::new("a", "A")]);
        assert!(asks_for_input(ui.confirm("Continue?", true)));
        assert!(asks_for_input(ui.text("Remote", Some("origin"), None)));
        assert!(asks_for_input(ui.select(&choose)));
    }

    #[test]
    fn the_error_names_the_question_and_how_to_avoid_it() {
        let err = ui(PromptPolicy::Refuse)
            .confirm("Create this commit?", true)
            .unwrap_err();
        let context = |key: &str| {
            err.context
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        assert_eq!(context("prompt"), "Create this commit?");
        assert!(context("hint").contains("flag"));
    }
}
