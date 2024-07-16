use crate::utils::{dag, get_pkgs, list, ListOpt, Result, INTERNAL_ERR};
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
            .map(|p| names.get(&p).expect(INTERNAL_ERR).0.id.clone());

        let pkgs = get_pkgs(&metadata, self.list.all)?;

        let ordered_pkgs = pkg_ids
            .into_iter()
            .filter_map(|id| pkgs.iter().find(|p| p.id == id))
            .cloned()
            .collect::<Vec<_>>();

        list(&ordered_pkgs, self.list)
    }
}
