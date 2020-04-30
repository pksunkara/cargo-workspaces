use crate::utils::{get_pkgs, ListOpt, Listable, Result};
use cargo_metadata::Metadata;
use clap::Clap;
use console::Term;

/// List crates in the project
#[derive(Debug, Clap)]
pub struct List {
    #[clap(flatten)]
    list: ListOpt,
}

impl List {
    pub fn run(self, metadata: Metadata, stdout: &Term, _: &Term) -> Result {
        let pkgs = get_pkgs(&metadata, self.list.all)?;
        pkgs.list(stdout, self.list)
    }
}
