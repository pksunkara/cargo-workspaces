use console::{style, Style, Term};
use std::io;
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
    #[error("unable to add files to git index, out = {0}, err = {1}")]
    NotAdded(String, String),
    #[error("unable to commit to git, out = {0}, err = {1}")]
    NotCommitted(String, String),
    #[error("unable to tag {0}, out = {1}, err = {2}")]
    NotTagged(String, String, String),
    #[error("unable to push to remote, out = {0}, err = {1}")]
    NotPushed(String, String),

    #[error("{0}")]
    Semver(#[from] semver::ReqParseError),
    #[error("{0}")]
    Glob(#[from] glob::PatternError),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("cannot convert command output to string, {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}

impl Error {
    pub fn print_err(self) -> io::Result<()> {
        let term = Term::stderr();
        self.print(&term)
    }

    fn color(self) -> Self {
        let yellow = Style::new().for_stderr().yellow();

        match self {
            Self::PackageNotInWorkspace { id, ws } => Self::PackageNotInWorkspace {
                id: format!("{}", yellow.apply_to(id)),
                ws: format!("{}", yellow.apply_to(ws)),
            },
            Self::PackageNotFound { id } => Self::PackageNotFound {
                id: format!("{}", yellow.apply_to(id)),
            },
            Self::NoRemote { remote, branch } => Self::NoRemote {
                remote: format!("{}", yellow.apply_to(remote)),
                branch: format!("{}", yellow.apply_to(branch)),
            },
            Self::BehindRemote { upstream, branch } => Self::BehindRemote {
                upstream: format!("{}", yellow.apply_to(upstream)),
                branch: format!("{}", yellow.apply_to(branch)),
            },
            Self::BranchNotAllowed { branch, pattern } => Self::BranchNotAllowed {
                branch: format!("{}", yellow.apply_to(branch)),
                pattern: format!("{}", yellow.apply_to(pattern)),
            },
            Self::NotTagged(tag, out, err) => {
                Self::NotTagged(format!("{}", yellow.apply_to(tag)), out, err)
            }
            _ => self,
        }
    }

    pub fn print(self, term: &Term) -> io::Result<()> {
        term.write_str(&format!("{}: ", style("error").for_stderr().red().bold()))?;

        let msg = format!("{}", self.color());

        term.write_line(&msg)?;
        term.flush()
    }
}
