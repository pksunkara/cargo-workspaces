use crate::utils::{get_pkgs, ListOpt, Listable, Result, Writer};
use cargo_metadata::Metadata;
use clap::Clap;

/// List local packages
#[derive(Debug, Clap)]
pub struct List {
    #[clap(flatten)]
    list: ListOpt,
}

impl List {
    pub fn run(self, metadata: Metadata, stdout: Writer, _: Writer) -> Result {
        let pkgs = get_pkgs(&metadata, self.list.all)?;
        pkgs.list(stdout, self.list)
    }
}
