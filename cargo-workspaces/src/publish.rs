use crate::utils::{cargo, info, Error, Result, VersionOpt, INTERNAL_ERR};
use cargo_metadata::{DependencyKind, Metadata, Package};
use clap::Clap;
use indexmap::IndexSet as Set;
use std::{collections::BTreeMap as Map, path::PathBuf};

/// Publish crates in the project
#[derive(Clap, Debug)]
pub struct Publish {
    #[clap(flatten)]
    version: VersionOpt,

    /// Publish crates from the current commit without versioning
    // TODO: conflicts_with = "version" (group)
    #[clap(long)]
    from_git: bool,

    /// Allow skipping already published crate versions
    #[clap(long)]
    skip_published: bool,
}

impl Publish {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = if !self.from_git {
            self.version
                .do_versioning(&metadata)?
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

        let mut names = Map::new();
        let mut visited = Set::new();

        for pkg in &pkgs {
            names.insert(&pkg.manifest_path, &pkg.name);
            ins(&pkgs, pkg, &mut visited);
        }

        // Filter out private packages
        let visited = visited
            .into_iter()
            .filter(|x| {
                if let Some(pkg) = pkgs.iter().find(|p| p.manifest_path == *x) {
                    return !pkg.publish.is_some()
                        || !pkg.publish.as_ref().expect(INTERNAL_ERR).is_empty();
                }

                false
            })
            .collect::<Set<_>>();

        info!("publish", "verifying crates")?;

        for p in &visited {
            let name = names.get(p).expect(INTERNAL_ERR).to_string();
            let output = cargo(
                &metadata.workspace_root,
                &[
                    "publish",
                    "--dry-run",
                    "--manifest-path",
                    &p.to_string_lossy(),
                ],
            )?;

            if !output.1.contains("aborting upload due to dry run") || output.1.contains("error:") {
                return Err(Error::Verify(name));
            }
        }

        for p in &visited {
            let name = names.get(p).expect(INTERNAL_ERR).to_string();
            let output = cargo(
                &metadata.workspace_root,
                &[
                    "publish",
                    "--no-verify",
                    "--manifest-path",
                    &p.to_string_lossy(),
                ],
            )?;

            if !output.1.contains("Uploading")
                || (output.1.contains("error:")
                    && !(self.skip_published && output.1.contains("is already uploaded")))
            {
                return Err(Error::Publish(name));
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
