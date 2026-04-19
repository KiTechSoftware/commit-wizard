pub mod runtime;
pub mod types;

pub struct ResolvedRuntime {
    pub behavior: ResolvedBehavior,
    pub rules: ResolvedRules,
    pub sources: ResolutionSources,
    pub repo_config_path: Option<PathBuf>,
    pub global_config_path: Option<PathBuf>,
    pub registry: Option<ResolvedRegistry>,
}