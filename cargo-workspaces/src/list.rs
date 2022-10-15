use crate::utils::{get_pkgs, ListOpt, Listable, Result};
use cargo_metadata::Metadata;
use clap::Parser;

/// List crates in the project
#[derive(Debug, Parser)]
#[clap(alias = "ls")]
pub struct List {
    #[clap(flatten)]
    list: ListOpt,
}

impl List {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = get_pkgs(&metadata, self.list.all, self.list.exclude_lib)?;
        pkgs.list(self.list)
    }
}
