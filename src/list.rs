use crate::utils::{Error, ListOpt, Listable, Pkg, Writer};
use cargo_metadata::Metadata;
use clap::Clap;

/// List local packages
#[derive(Debug, Clap)]
pub struct List {
    #[clap(flatten)]
    list: ListOpt,
}

impl List {
    pub fn run(self, metadata: Metadata, stdout: Writer) -> Result<(), Error> {
        let mut pkgs = vec![];

        for id in metadata.workspace_members {
            if let Some(pkg) = metadata.packages.iter().find(|x| x.id == id) {
                let private = pkg.publish.is_some() && pkg.publish.as_ref().unwrap().is_empty();

                if !self.list.all && private {
                    continue;
                }

                let loc = pkg.manifest_path.strip_prefix(&metadata.workspace_root);

                if loc.is_err() {
                    return Err(Error::PackageNotInWorkspace {
                        id: pkg.id.repr.clone(),
                        ws: metadata.workspace_root.to_string_lossy().to_string(),
                    });
                }

                let loc = loc.unwrap().to_string_lossy();
                let loc = loc.trim_end_matches("Cargo.toml").trim_end_matches("/");

                pkgs.push(Pkg {
                    name: pkg.name.clone(),
                    version: format!("{}", pkg.version),
                    location: metadata.workspace_root.join(loc),
                    path: loc.to_string(),
                    private,
                });
            } else {
                Error::PackageNotFound {
                    id: id.repr.clone(),
                }
                .print()?;
            }
        }

        if pkgs.is_empty() {
            return Err(Error::EmptyWorkspace);
        }

        pkgs.sort();
        pkgs.list(stdout, self.list)
    }
}
