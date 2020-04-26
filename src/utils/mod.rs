mod changable;
mod error;
mod git;
mod listable;
mod pkg;
mod version;
mod writer;

pub use changable::{get_changed_pkgs, ChangeData, ChangeOpt};
pub use error::Error;
pub use git::{git, GitOpt};
pub use listable::{ListOpt, Listable};
pub use pkg::{get_pkgs, Pkg};
pub use version::ask_version;
pub use writer::Writer;

pub type Result = std::result::Result<(), Error>;

pub const INTERNAL_ERR: &'static str = "Internal error message. Please create an issue on https://github.com/pksunkara/cargo-workspaces";
