use anyhow::Result;

#[derive(Debug, Default, Clone, Copy)]
pub struct CommitOptions {
    pub allow_empty: bool,
}

pub trait GitPort {
    fn commit(&self, message: &str, opts: &CommitOptions) -> Result<()>;
}
