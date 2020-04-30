mod cargo;
mod changable;
pub mod error;
mod git;
mod listable;
mod pkg;
mod version;

pub use cargo::{cargo, change_versions};
pub use changable::{ChangeData, ChangeOpt};
pub use error::Error;
pub(crate) use error::{debug, info};
pub use git::{git, GitOpt};
pub use listable::{ListOpt, Listable};
pub use pkg::{get_pkgs, Pkg};
pub use version::VersionOpt;

pub type Result = std::result::Result<(), Error>;

pub const INTERNAL_ERR: &'static str = "Internal error message. Please create an issue on https://github.com/pksunkara/cargo-workspaces";
