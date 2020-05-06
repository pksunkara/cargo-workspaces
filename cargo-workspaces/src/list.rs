use crate::utils::{get_pkgs, ListOpt, Listable, Result};
use cargo_metadata::Metadata;
use clap::Clap;

/// List crates in the project
#[derive(Debug, Clap)]
#[clap(alias = "ls")]
pub struct List {
    #[clap(flatten)]
    list: ListOpt,
}

impl List {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = get_pkgs(&metadata, self.list.all)?;
        pkgs.list(self.list)
    }
}
