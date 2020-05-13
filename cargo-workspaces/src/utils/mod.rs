mod cargo;
mod changable;
mod error;
mod git;
mod listable;
mod pkg;
mod version;

pub use cargo::{cargo, change_versions};
pub use changable::{ChangeData, ChangeOpt};
pub(crate) use error::{debug, info};
pub use error::{get_debug, set_debug, Error, GREEN, MAGENTA, TERM_ERR, TERM_OUT};
pub use git::{git, GitOpt};
pub use listable::{ListOpt, Listable};
pub use pkg::{get_pkgs, Pkg};
pub use version::VersionOpt;

pub type Result<T = ()> = std::result::Result<T, Error>;

pub const INTERNAL_ERR: &'static str = "Internal error message. Please create an issue on https://github.com/pksunkara/cargo-workspaces";
