use crate::utils::{cargo, info, Error, VersionOpt, INTERNAL_ERR};
use cargo_metadata::{DependencyKind, Metadata, Package};
use clap::Clap;
use console::Term;
use indexmap::IndexSet as Set;
use std::{collections::BTreeMap as Map, path::PathBuf};

/// Publish crates in the project
#[derive(Clap, Debug)]
pub struct Publish {
    #[clap(flatten)]
    version: VersionOpt,

    /// Publish crates from the current commit without versioning
    #[clap(long)]
    from_git: bool,
}

impl Publish {
    pub fn run(self, metadata: Metadata, _: &Term, stderr: &Term) -> Result<(), Error> {
        let pkgs = if !self.from_git {
            self.version
                .do_versioning(&metadata, stderr)?
                .iter()
                .map(|x| {
                    metadata
                        .packages
                        .iter()
                        .find(|y| x.0 == &y.name)
                        .expect(INTERNAL_ERR)
                        .clone()
                })
                .collect()
        } else {
            metadata.packages
        };

        info!("publish", "verifying crates")?;

        let mut names = Map::new();
        let mut visited = Set::new();

        for pkg in &pkgs {
            names.insert(&pkg.manifest_path, &pkg.name);
            ins(&pkgs, pkg, &mut visited);
        }

        for p in &visited {
            let output = cargo(
                &metadata.workspace_root,
                &[
                    "publish",
                    "--dry-run",
                    "--allow-dirty",
                    "--manifest-path",
                    &p.to_string_lossy(),
                ],
            )?;

            if !output.1.contains("Finished") {
                return Err(Error::Verify(
                    names.get(p).expect(INTERNAL_ERR).to_string(),
                    output.1,
                ));
            }
        }

        for p in &visited {
            let name = names.get(p).expect(INTERNAL_ERR).to_string();
            let output = cargo(
                &metadata.workspace_root,
                &[
                    "publish",
                    "--no-verify",
                    "--allow-dirty",
                    "--manifest-path",
                    &p.to_string_lossy(),
                ],
            )?;

            if !output.1.contains("Uploading") {
                return Err(Error::Publish(name, output.1));
            }

            info!("published", name)?;
        }

        info!("success", "ok")?;
        Ok(())
    }
}

fn ins(pkgs: &[Package], pkg: &Package, visited: &mut Set<PathBuf>) {
    if visited.contains(&pkg.manifest_path) {
        return;
    }

    for d in &pkg.dependencies {
        match d.kind {
            DependencyKind::Normal | DependencyKind::Build => {
                if let Some(dep) = pkgs.iter().find(|p| d.name == p.name) {
                    ins(pkgs, dep, visited);
                }
            }
            _ => {}
        }
    }

    visited.insert(pkg.manifest_path.clone());
}
