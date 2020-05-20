use console::{style, Style, Term};
use lazy_static::lazy_static;
use std::{
    io,
    sync::atomic::{AtomicBool, Ordering},
};
use thiserror::Error;

lazy_static! {
    pub static ref TERM_ERR: Term = Term::stderr();
    pub static ref TERM_OUT: Term = Term::stdout();
    static ref YELLOW: Style = Style::new().for_stderr().yellow();
    pub static ref GREEN: Style = Style::new().for_stderr().green();
    pub static ref MAGENTA: Style = Style::new().for_stderr().magenta();
    static ref DEBUG: AtomicBool = AtomicBool::new(false);
}

pub fn get_debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

pub fn set_debug() {
    DEBUG.store(true, Ordering::Relaxed);
}

macro_rules! _info {
    ($desc:literal, $val:expr) => {
        $crate::utils::TERM_ERR.write_line(&format!(
            "{} {} {}",
            $crate::utils::GREEN.apply_to("info"),
            $crate::utils::MAGENTA.apply_to($desc),
            $val
        ))
    };
}

macro_rules! _debug {
    ($desc:literal, $val:expr) => {
        if $crate::utils::get_debug() {
            $crate::utils::TERM_ERR.write_line(&format!(
                "{} {} {}",
                $crate::utils::GREEN.apply_to("debug"),
                $crate::utils::MAGENTA.apply_to($desc),
                $val
            ))
        } else {
            Ok(())
        }
    };
}

pub(crate) use _debug as debug;
pub(crate) use _info as info;

#[derive(Error, Debug)]
pub enum Error {
    #[error("package {id} is not inside workspace {ws}")]
    PackageNotInWorkspace { id: String, ws: String },
    #[error("unable to find package {id}")]
    PackageNotFound { id: String },
    #[error("did not find any package")]
    EmptyWorkspace,
    #[error("unable to verify package {0}")]
    Verify(String),
    #[error("unable to publish package {0}")]
    Publish(String),
    #[error("unable to update Cargo.lock")]
    Update,
    #[error("unable to create crate")]
    Create,
    #[error("package {0}'s manifest has not parent directory")]
    ManifestHasNoParent(String),

    #[error("unable to run cargo command with args {args:?}, got {err}")]
    Cargo { err: io::Error, args: Vec<String> },
    #[error("unable to run git command with args {args:?}, got {err}")]
    Git { err: io::Error, args: Vec<String> },

    #[error("child command failed to exit successfully")]
    Bail,

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
        self.print(&TERM_ERR)
    }

    fn color(self) -> Self {
        match self {
            Self::PackageNotInWorkspace { id, ws } => Self::PackageNotInWorkspace {
                id: format!("{}", YELLOW.apply_to(id)),
                ws: format!("{}", YELLOW.apply_to(ws)),
            },
            Self::PackageNotFound { id } => Self::PackageNotFound {
                id: format!("{}", YELLOW.apply_to(id)),
            },
            Self::Verify(pkg) => Self::Verify(format!("{}", YELLOW.apply_to(pkg))),
            Self::Publish(pkg) => Self::Publish(format!("{}", YELLOW.apply_to(pkg))),
            Self::NoRemote { remote, branch } => Self::NoRemote {
                remote: format!("{}", YELLOW.apply_to(remote)),
                branch: format!("{}", YELLOW.apply_to(branch)),
            },
            Self::BehindRemote { upstream, branch } => Self::BehindRemote {
                upstream: format!("{}", YELLOW.apply_to(upstream)),
                branch: format!("{}", YELLOW.apply_to(branch)),
            },
            Self::BranchNotAllowed { branch, pattern } => Self::BranchNotAllowed {
                branch: format!("{}", YELLOW.apply_to(branch)),
                pattern: format!("{}", YELLOW.apply_to(pattern)),
            },
            Self::NotTagged(tag, out, err) => {
                Self::NotTagged(format!("{}", YELLOW.apply_to(tag)), out, err)
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
