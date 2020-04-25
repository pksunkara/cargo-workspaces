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
                stderr.cyan(id)?;
                stderr.none(" is not inside workspace ")?;
                stderr.cyan(ws)?;
                stderr.none("\n")?;
            }
            Self::PackageNotFound { id } => {
                stderr.none("unable to find package ")?;
                stderr.cyan(id)?;
                stderr.none("\n")?;
            }
            Self::EmptyWorkspace => stderr.none("did not find any package\n")?,
            _ => stderr.none(&format!("{}", self))?,
        }

        return Ok(());
    }
}
