//! Web links for changelog entries, derived from the repository's remote.

/// How a hosting service lays out its commit and compare pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    /// `/commit/<sha>`, `/compare/<a>...<b>`, `/releases/tag/<tag>` (GitHub and look-alikes)
    GitHub,
    /// The same pages under `/-/` (GitLab)
    GitLab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Links {
    base: String,
    style: Style,
}

impl Links {
    /// Links for a repository whose web address is `base`, such as
    /// `https://github.com/owner/repo`.
    pub fn new(base: &str) -> Self {
        let base = base.trim().trim_end_matches('/').to_string();
        let style = if host_of(&base).is_some_and(|host| host.contains("gitlab")) {
            Style::GitLab
        } else {
            Style::GitHub
        };
        Self { base, style }
    }

    /// Links for the repository behind a git remote URL, or `None` when the
    /// remote is not something with a web address (a local path, say).
    pub fn from_remote(remote: &str) -> Option<Self> {
        web_url(remote).map(|base| Self::new(&base))
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    fn page(&self, path: &str) -> String {
        match self.style {
            Style::GitHub => format!("{}/{path}", self.base),
            Style::GitLab => format!("{}/-/{path}", self.base),
        }
    }

    pub fn commit(&self, hash: &str) -> String {
        self.page(&format!("commit/{hash}"))
    }

    /// Changes between two revisions, or the tag's own page for the first release.
    pub fn release(&self, from: Option<&str>, to: &str) -> String {
        match from {
            Some(from) => self.page(&format!("compare/{from}...{to}")),
            None => self.page(&format!("tree/{to}")),
        }
    }
}

fn host_of(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    rest.split('/').next()?.rsplit('@').next()
}

/// `https://host/owner/repo` for the usual git remote spellings.
fn web_url(remote: &str) -> Option<String> {
    let remote = remote.trim();

    // scp-like: git@github.com:owner/repo.git
    let (host, path) = if let Some((_, rest)) = remote.split_once("://") {
        // https://[user[:token]@]host[:port]/owner/repo.git, ssh://git@host/owner/repo.git
        let (authority, path) = rest.split_once('/')?;
        let host = authority.rsplit('@').next()?;
        let host = host.split(':').next()?;
        (host.to_string(), path.to_string())
    } else if let Some((authority, path)) = remote.split_once(':') {
        if path.starts_with('/') || authority.contains('/') {
            return None;
        }
        (authority.rsplit('@').next()?.to_string(), path.to_string())
    } else {
        return None;
    };

    let path = path.trim_matches('/').trim_end_matches(".git");
    if host.is_empty() || !host.contains('.') || path.split('/').count() < 2 {
        return None;
    }
    Some(format!("https://{host}/{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_remote_spellings_become_the_same_web_address() {
        for remote in [
            "git@github.com:owner/repo.git",
            "https://github.com/owner/repo.git",
            "https://github.com/owner/repo",
            "ssh://git@github.com/owner/repo.git",
            "https://user:token@github.com/owner/repo.git",
            "https://github.com/owner/repo/",
        ] {
            assert_eq!(
                web_url(remote).as_deref(),
                Some("https://github.com/owner/repo"),
                "{remote}"
            );
        }
    }

    #[test]
    fn remotes_without_a_web_address_give_no_links() {
        for remote in [
            "/srv/git/repo.git",
            "../repo",
            "file:///tmp/repo",
            "repo",
            "",
        ] {
            assert_eq!(Links::from_remote(remote), None, "{remote}");
        }
    }

    #[test]
    fn github_pages() {
        let links = Links::new("https://github.com/owner/repo");
        assert_eq!(
            links.commit("abc"),
            "https://github.com/owner/repo/commit/abc"
        );
        assert_eq!(
            links.release(Some("v1.0.0"), "v1.1.0"),
            "https://github.com/owner/repo/compare/v1.0.0...v1.1.0"
        );
        assert_eq!(
            links.release(None, "v1.0.0"),
            "https://github.com/owner/repo/tree/v1.0.0"
        );
    }

    #[test]
    fn gitlab_pages_live_under_a_dash() {
        let links = Links::from_remote("git@gitlab.com:group/sub/repo.git").unwrap();
        assert_eq!(links.base(), "https://gitlab.com/group/sub/repo");
        assert_eq!(
            links.commit("abc"),
            "https://gitlab.com/group/sub/repo/-/commit/abc"
        );
        assert_eq!(
            links.release(Some("a"), "b"),
            "https://gitlab.com/group/sub/repo/-/compare/a...b"
        );
    }

    #[test]
    fn an_override_url_is_used_as_given() {
        let links = Links::new("https://git.example.com/team/app/");
        assert_eq!(
            links.commit("abc"),
            "https://git.example.com/team/app/commit/abc"
        );
    }
}
