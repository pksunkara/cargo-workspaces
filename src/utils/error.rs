use console::{style, Style, Term};
use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("package {id} is not inside workspace {ws}")]
    PackageNotInWorkspace { id: String, ws: String },
    #[error("unable to find package {id}")]
    PackageNotFound { id: String },
    #[error("did not find any package")]
    EmptyWorkspace,

    #[error("unable to run git command with args {args:?}, got {err}")]
    Git { err: io::Error, args: Vec<String> },
    #[error("not a git repository")]
    NotGit,
    #[error("no commits in this repository")]
    NoCommits,
    #[error("not on a git branch")]
    NotBranch,
    #[error("remote {remote} not found or branch {branch} not in {remote}")]
    NoRemote { remote: String, branch: String },
    #[error("local branch {branch} is behind upstream {upstream}")]
    BehindRemote { upstream: String, branch: String },
    #[error("not allowed to run on branch {branch} because it doesn't match pattern {pattern}")]
    BranchNotAllowed { branch: String, pattern: String },

    #[error("{0}")]
    Glob(#[from] glob::PatternError),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("cannot convert command output to string, {0}")]
    FromUtf8(#[from] FromUtf8Error),
}

impl Error {
    pub fn print_err(&self) -> io::Result<()> {
        let term = Term::stderr();
        self.print(&term)
    }

    pub fn print(&self, term: &Term) -> io::Result<()> {
        let yellow = Style::new().for_stderr().yellow();

        term.write_str(&format!("{}: ", style("error").for_stderr().red().bold()))?;

        let msg = match self {
            Self::PackageNotInWorkspace { id, ws } => format!(
                "package {} is not inside workspace {}",
                yellow.apply_to(id),
                yellow.apply_to(ws)
            ),
            Self::PackageNotFound { id } => {
                format!("unable to find package {}", yellow.apply_to(id))
            }
            Self::NoRemote { remote, branch } => format!(
                "remote {} not found or branch {} not found in {}",
                yellow.apply_to(remote),
                yellow.apply_to(branch),
                yellow.apply_to(remote)
            ),
            Self::BehindRemote { upstream, branch } => format!(
                "local branch {} is behind upstream {}",
                yellow.apply_to(branch),
                yellow.apply_to(upstream)
            ),
            Self::BranchNotAllowed { branch, pattern } => format!(
                "not allowed to run on branch {} because it doesn't match pattern {}",
                yellow.apply_to(branch),
                yellow.apply_to(pattern)
            ),
            _ => format!("{}", self),
        };

        term.write_line(&msg)?;
        term.flush()
    }
}
