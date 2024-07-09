use crate::utils::{
    create_http_client, dag, filter_private, info, is_published, package_registry, Result,
    INTERNAL_ERR,
};

use cargo_metadata::Metadata;
use clap::Parser;

/// Prepare a plan for publishing crates in the project
#[derive(Debug, Parser)]
#[clap(next_help_heading = "PUBLISH PLAN OPTIONS")]
pub struct Plan {
    /// The token to use for accessing the registry
    #[clap(long, forbid_empty_values(true))]
    token: Option<String>,

    /// The Cargo registry to check against
    #[clap(long, forbid_empty_values(true))]
    registry: Option<String>,

    /// Check if the crates are already published.
    #[clap(long, short)]
    check_published: bool,
}

impl Plan {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs: Vec<_> = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect();

        let (names, visited) = dag(&pkgs);

        // Filter out private packages
        let visited = filter_private(visited, &pkgs);

        let http_client = create_http_client(&metadata.workspace_root, &self.token)?;
        for p in &visited {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();

            let name_ver = format!("{} v{}", name, version);

            let index_url = package_registry(&metadata, self.registry.as_ref(), pkg)?;

            let already_published = if self.check_published {
                is_published(&http_client, index_url, &name, version)?
            } else {
                false
            };

            let msg = if already_published {
                format!("{name_ver} (already published)")
            } else {
                name_ver
            };
            info!("- ", msg);
        }

        info!("success", "ok");
        Ok(())
    }
}
