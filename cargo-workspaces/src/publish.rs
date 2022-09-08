use crate::utils::{
    cargo, cargo_config_get, check_index, dag, info, is_published, Error, Result, VersionOpt,
    INTERNAL_ERR,
};
use cargo_metadata::Metadata;
use clap::Parser;
use crates_index::Index;
use indexmap::IndexSet as Set;

/// Publish crates in the project
#[derive(Debug, Parser)]
#[clap(next_help_heading = "PUBLISH OPTIONS")]
pub struct Publish {
    #[clap(flatten, next_help_heading = None)]
    version: VersionOpt,

    /// Publish crates from the current commit without versioning
    // TODO: conflicts_with = "version" (group)
    #[clap(long)]
    from_git: bool,

    /// Perform all checks without uploading, and if versioning without pushing
    #[clap(long)]
    dry_run: bool,

    /// Skip already published crate versions
    #[clap(long, hide = true)]
    skip_published: bool,

    /// Skip crate verification (not recommended)
    #[clap(long)]
    no_verify: bool,

    /// Allow dirty working directories to be published
    #[clap(long)]
    allow_dirty: bool,

    /// The token to use for publishing
    #[clap(long, forbid_empty_values(true))]
    token: Option<String>,

    /// The Cargo registry to use for publishing
    #[clap(long, forbid_empty_values(true))]
    registry: Option<String>,
}

impl Publish {
    pub fn run(self, metadata: Metadata) -> Result {
        let pkgs = if !self.from_git {
            let mut version = self.version;
            if self.dry_run {
                version.git.no_git_push = true;
            }
            version
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
                    return pkg.publish.is_none()
                        || !pkg.publish.as_ref().expect(INTERNAL_ERR).is_empty();
                }

                false
            })
            .collect::<Set<_>>();

        for p in &visited {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();
            let mut args = vec!["publish"];

            let name_ver = format!("{} v{}", name, version);

            let mut index =
                if let Some(publish) = pkg.publish.as_deref().and_then(|x| x.get(0)).as_deref() {
                    let registry_url = cargo_config_get(
                        &metadata.workspace_root,
                        &format!("registries.{}.index", publish),
                    )?;
                    Index::from_url(&format!("registry+{}", registry_url))?
                } else {
                    Index::new_cargo_default()?
                };

            if !self.dry_run && is_published(&mut index, &name, version)? {
                info!("already published", name_ver);
                continue;
            }

            if self.dry_run {
                args.push("--dry-run");
            }

            if self.no_verify {
                args.push("--no-verify");
            }

            if self.allow_dirty {
                args.push("--allow-dirty");
            }

            if let Some(ref registry) = self.registry {
                args.push("--registry");
                args.push(registry);
            }

            if let Some(ref token) = self.token {
                args.push("--token");
                args.push(token);
            }

            args.push("--manifest-path");
            args.push(p.as_str());

            let (_, stderr) = cargo(&metadata.workspace_root, &args, &[])?;

            if !stderr.contains("Uploading") || stderr.contains("error:") {
                return Err(Error::Publish(name));
            }

            if !self.dry_run {
                check_index(&mut index, &name, version)?;
            }

            if self.dry_run {
                info!("published (dry run)", name_ver);
            } else {
                info!("published", name_ver);
            }
        }

        info!("success", "ok");
        Ok(())
    }
}
