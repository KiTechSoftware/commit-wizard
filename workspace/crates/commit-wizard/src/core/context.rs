use std::path::PathBuf;

use crate::{
    engine::{
        Error,
        models::{
            policy::Policy,
            runtime::{AvailableConfigOptions, ResolvedConfig, Runtime, mode::RunMode},
        },
    },
    infra::{git::Git, ui::Ui},
};

pub type AppResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Context {
    runtime: Runtime,
}

impl Context {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn set_workdir(&mut self, path: PathBuf) {
        self.runtime.set_cwd(path);
    }

    pub fn set_run_mode(&mut self, ci: bool, non_interactive: bool) {
        self.runtime
            .set_mode(RunMode::from_flags(ci, non_interactive));
    }

    pub(crate) fn set_in_git_repo(&mut self, value: bool) {
        self.runtime.set_in_git_repo(value);
    }

    pub(crate) fn set_repo_root(&mut self, path: PathBuf) {
        self.runtime.set_repo_root(path);
    }

    pub(crate) fn resolve_available_sources(&mut self) {
        let ui = self.ui();
        self.runtime.resolve_available_sources(&ui);
    }

    pub(crate) fn resolve_active_config(&mut self) -> crate::engine::error::Result<()> {
        let ui = self.ui();
        self.runtime.resolve_active_config(&ui)
    }

    pub fn is_interactive(&self) -> bool {
        self.runtime.is_interactive()
    }

    pub fn cwd(&self) -> &PathBuf {
        self.runtime.cwd()
    }

    pub fn repo_root(&self) -> &PathBuf {
        self.runtime.repo_root()
    }

    pub fn dry_run(&self) -> bool {
        self.runtime.options().dry_run()
    }

    pub fn auto_yes(&self) -> bool {
        self.runtime.options().auto_yes()
    }

    pub fn force(&self) -> bool {
        self.runtime.options().force()
    }

    pub fn in_git_repo(&self) -> bool {
        self.runtime.in_git_repo()
    }

    pub fn config(&self) -> Option<&ResolvedConfig> {
        self.runtime.config()
    }
    pub fn explicit_config_path(&self) -> Option<&PathBuf> {
        self.runtime.explicit_config_path()
    }

    pub fn project_config_path(&self) -> Option<PathBuf> {
        self.runtime.project_config_path()
    }

    pub fn sources(&self) -> &AvailableConfigOptions {
        self.runtime.sources()
    }

    pub fn policy(&self) -> &Policy {
        self.runtime.policy()
    }

    pub fn ui_config(&self) -> scriba::Config {
        self.runtime.output_config()
    }

    pub fn ui(&self) -> Ui {
        Ui::cached_with_config(self.ui_config(), self.runtime.options().output_envelope())
    }

    pub fn git(&self) -> Git {
        Git::new(self.cwd().clone())
    }
}
