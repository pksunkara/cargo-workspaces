use crate::utils::{cargo, check_index, info, Error, Result, VersionOpt, INTERNAL_ERR};
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

    /// Skip already published crate versions
    #[clap(long)]
    skip_published: bool,

    /// Skip crate verification (not recommended)
    #[clap(long)]
    no_verify: bool,

    /// Allow dirty working directories to be published
    #[clap(long)]
    allow_dirty: bool,
}

impl Publish {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = if !self.from_git {
            self.version
                .do_versioning(&metadata)?
                .iter()
                .map(|x| {
                    (
                        metadata
                            .packages
                            .iter()
                            .find(|y| x.0 == &y.name)
                            .expect(INTERNAL_ERR)
                            .clone(),
                        x.1.to_string(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            metadata
                .packages
                .iter()
                .map(|x| (x.clone(), x.version.to_string()))
                .collect()
        };

        let mut names = Map::new();
        let mut visited = Set::new();

        for (pkg, version) in &pkgs {
            names.insert(&pkg.manifest_path, (pkg, version));
            ins(&pkgs, pkg, &mut visited);
        }

        // Filter out private packages
        let visited = visited
            .into_iter()
            .filter(|x| {
                if let Some((pkg, _)) = pkgs.iter().find(|(p, _)| p.manifest_path == *x) {
                    return !pkg.publish.is_some()
                        || !pkg.publish.as_ref().expect(INTERNAL_ERR).is_empty();
                }

                false
            })
            .collect::<Set<_>>();

        for p in &visited {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();
            let path = p.to_string_lossy();
            let mut args = vec!["publish"];

            if self.no_verify {
                args.push("--no-verify");
            }

            if self.allow_dirty {
                args.push("--allow-dirty");
            }

            args.push("--manifest-path");
            args.push(&path);

            let output = cargo(&metadata.workspace_root, &args)?;

            if !output.1.contains("Uploading")
                || (output.1.contains("error:")
                    && !(self.skip_published && output.1.contains("is already uploaded")))
            {
                return Err(Error::Publish(name));
            }

            // TODO: How to update index for non crates.io
            if pkg.publish.is_none() {
                check_index(&name, version)?;
            }

            info!("published", name)?;
        }

        info!("success", "ok")?;
        Ok(())
    }
}

fn ins(pkgs: &[(Package, String)], pkg: &Package, visited: &mut Set<PathBuf>) {
    if visited.contains(&pkg.manifest_path) {
        return;
    }

    for d in &pkg.dependencies {
        match d.kind {
            DependencyKind::Normal | DependencyKind::Build => {
                if let Some((dep, _)) = pkgs.iter().find(|(p, _)| d.name == p.name) {
                    ins(pkgs, dep, visited);
                }
            }
            _ => {}
        }
    }

    visited.insert(pkg.manifest_path.clone());
}
