use crate::utils::{dag, get_pkgs, ListOpt, Listable, Result, INTERNAL_ERR};
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
        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect::<Vec<_>>();

        let (names, visited) = dag(&pkgs);

        let pkg_ids = visited
            .into_iter()
            .map(|p| names.get(&p).expect(INTERNAL_ERR).0.id.clone())
            .collect::<Vec<_>>();

        let pkgs = get_pkgs(&metadata, self.list.all)?;

        pkg_ids
            .into_iter()
            .map(|id| {
                pkgs.iter()
                    .find(|p| p.id == id)
                    .expect(INTERNAL_ERR)
                    .clone()
            })
            .collect::<Vec<_>>()
            .list(self.list)
    }
}
