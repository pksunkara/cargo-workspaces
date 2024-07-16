use crate::utils::{
    create_http_client, dag, filter_private, get_pkgs, is_published, list, package_registry,
    ListOpt, ListPublicOpt, RegistryOpt, Result, INTERNAL_ERR,
};

use cargo_metadata::Metadata;
use clap::Parser;

/// List the crates in publishing order
#[derive(Debug, Parser)]
pub struct Plan {
    /// Skip already published crate versions
    #[clap(long)]
    skip_published: bool,

    #[clap(flatten)]
    registry: RegistryOpt,

    #[clap(flatten)]
    list: ListPublicOpt,
}

impl Plan {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect::<Vec<_>>();

        let (names, visited) = dag(&pkgs);

        let http_client = create_http_client(&metadata.workspace_root, &self.registry.token)?;

        let pkg_ids = filter_private(visited, &pkgs)
            .into_iter()
            .map(|p| {
                let (pkg, version) = names.get(&p).expect(INTERNAL_ERR);

                let published = if self.skip_published {
                    let index_url =
                        package_registry(&metadata, self.registry.registry.as_ref(), pkg)?;
                    is_published(&http_client, index_url, &pkg.name, version)?
                } else {
                    false
                };

                Ok((pkg.id.clone(), published))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter_map(|(id, published)| (!published).then_some(id));

        let pkgs = get_pkgs(&metadata, false)?;

        let ordered_pkgs = pkg_ids
            .into_iter()
            .filter_map(|id| pkgs.iter().find(|p| p.id == id))
            .cloned()
            .collect::<Vec<_>>();

        list(
            &ordered_pkgs,
            ListOpt {
                all: false,
                list: self.list,
            },
        )
    }
}
