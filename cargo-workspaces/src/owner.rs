use crate::utils::{cargo, dag, filter_private, info, Error, RegistryOpt, Result, INTERNAL_ERR};

use cargo_metadata::Metadata;
use clap::Parser;

/// Manage the owners of the workspaces crates
#[derive(Debug, Parser)]
#[clap(next_help_heading = "OWNER OPTIONS")]
pub struct Owner {
    /// Name of a user or team to invite as an owner
    #[clap(short, long)]
    add: Option<String>,

    /// Name of a user or team to remove as an owner
    #[clap(short, long)]
    remove: Option<String>,

    /// List owners for each crate in the workspace
    #[clap(short, long)]
    list: bool,

    #[clap(flatten)]
    registry: RegistryOpt,
}

impl Owner {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect::<Vec<_>>();

        let (names, visited) = dag(&pkgs);

        // Filter out private packages
        let visited = filter_private(visited, &pkgs);

        for p in &visited {
            let (pkg, _version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();

            let mut args = vec!["owner"];

            if let Some(ref registry) = self.registry.registry {
                args.push("--registry");
                args.push(registry);
            }

            if let Some(ref token) = self.registry.token {
                args.push("--token");
                args.push(token);
            }

            if let Some(ref add) = self.add {
                args.push("--add");
                args.push(add);
            }

            if let Some(ref remove) = self.remove {
                args.push("--remove");
                args.push(remove);
            }

            if self.list {
                args.push("--list");
            }

            // `cargo owner` doesn't support `manifest-path`
            let crate_path = pkg.manifest_path.parent().expect(INTERNAL_ERR);

            let (stdout, stderr) = cargo(crate_path, &args, &[])?;
            // `cargo owner` uses `stdout` to print the names.
            eprintln!("{}", stdout);
            if stderr.contains("error:") {
                return Err(Error::Owner(name));
            }
        }

        info!("success", "ok");
        Ok(())
    }
}
