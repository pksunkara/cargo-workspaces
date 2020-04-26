use crate::utils::Writer;
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
    pub fn print(&self) -> io::Result<()> {
        let mut stderr = Writer::new(true);

        stderr.b_red("error")?;
        stderr.none(": ")?;

        match self {
            Self::PackageNotInWorkspace { id, ws } => {
                stderr.none("package ")?;
                stderr.yellow(id)?;
                stderr.none(" is not inside workspace ")?;
                stderr.yellow(ws)?;
                stderr.none("\n")?;
            }
            Self::PackageNotFound { id } => {
                stderr.none("unable to find package ")?;
                stderr.yellow(id)?;
                stderr.none("\n")?;
            }
            Self::NoRemote { remote, branch } => {
                stderr.none("remote ")?;
                stderr.yellow(remote)?;
                stderr.none(" not found or branch ")?;
                stderr.yellow(branch)?;
                stderr.none(" not found in ")?;
                stderr.yellow(remote)?;
                stderr.none("\n")?;
            }
            Self::BehindRemote { upstream, branch } => {
                stderr.none("local branch ")?;
                stderr.yellow(branch)?;
                stderr.none(" is behind upstream ")?;
                stderr.yellow(upstream)?;
                stderr.none("\n")?;
            }
            Self::BranchNotAllowed { branch, pattern } => {
                stderr.none("not allowed to run on branch ")?;
                stderr.yellow(branch)?;
                stderr.none(" because it doesn't match pattern ")?;
                stderr.yellow(pattern)?;
                stderr.none("\n")?;
            }
            _ => stderr.none(&format!("{}\n", self))?,
        }

        return Ok(());
    }
}
