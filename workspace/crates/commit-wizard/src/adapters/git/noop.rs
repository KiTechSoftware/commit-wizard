use crate::ports::git::{CommitOptions, GitPort};
use anyhow::Result;

#[derive(Default)]
pub struct NoopGit;

impl GitPort for NoopGit {
    fn commit(&self, message: &str, opts: &CommitOptions) -> Result<()> {
        println!("---[commit-wizard] (dry-run) ---------------------");
        println!("{message}");
        if opts.allow_empty {
            println!("(note) --allow-empty requested");
        }
        println!(
            "(noop) Would run: git commit -F <msgfile>{}",
            " if allow-empty".replace(
                " if allow-empty",
                if opts.allow_empty {
                    " --allow-empty"
                } else {
                    ""
                }
            )
        );
        Ok(())
    }
}
