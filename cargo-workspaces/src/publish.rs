use crate::utils::{cargo, check_index, dag, info, Error, Result, VersionOpt, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::{ArgSettings, Parser};
use indexmap::IndexSet as Set;

/// Publish crates in the project
#[derive(Debug, Parser)]
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

    /// The token to use for publishing
    #[clap(long, setting(ArgSettings::ForbidEmptyValues))]
    token: Option<String>,
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

        let (names, visited) = dag(&pkgs);

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

            if let Some(ref token) = self.token {
                args.push("--token");
                args.push(token);
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

            info!("published", name);
        }

        info!("success", "ok");
        Ok(())
    }
}
