use crate::utils::{
    basic_checks, cargo, create_http_client, dag, filter_private, info, is_published,
    package_registry, should_remove_dev_deps, warn, DevDependencyRemover, Error, RegistryOpt,
    Result, VersionOpt, INTERNAL_ERR,
};

use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use clap::Parser;

/// Publish crates in the project
#[derive(Debug, Parser)]
#[clap(next_help_heading = "PUBLISH OPTIONS")]
pub struct Publish {
    #[clap(flatten)]
    version: VersionOpt,

    /// Publish crates from the current commit without versioning
    // TODO: conflicts_with = "version" (group)
    #[clap(long, alias = "from-git")]
    publish_as_is: bool,

    /// Skip already published crate versions
    #[clap(long, hide = true)]
    skip_published: bool,

    /// Skip crate verification (not recommended)
    #[clap(long)]
    no_verify: bool,

    /// Allow dirty working directories to be published
    #[clap(long)]
    allow_dirty: bool,

    /// Don't remove dev-dependencies while publishing
    #[clap(long)]
    no_remove_dev_deps: bool,

    /// Runs in dry-run mode
    #[clap(long)]
    dry_run: bool,

    #[clap(flatten)]
    registry: RegistryOpt,
}

impl Publish {
    pub fn run(mut self, metadata: Metadata) -> Result {
        if self.dry_run {
            warn!(
                "Dry run doesn't check that all dependencies have been published.",
                ""
            );

            if !self.publish_as_is {
                warn!("Dry run doesn't perform versioning.", "");
                self.publish_as_is = true;
            }
        }

        let pkgs = if !self.publish_as_is {
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
        let visited = filter_private(visited, &pkgs);

        let http_client = create_http_client(&metadata.workspace_root, &self.registry.token)?;

        for p in &visited {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();

            if self.dry_run {
                info!("checking", name);

                if !self.no_verify && !self.build(&metadata.workspace_root, p)? {
                    warn!("build failed", "");
                }

                basic_checks(pkg)?;
            }

            let mut args = vec!["publish"];

            let name_ver = format!("{} v{}", name, version);
            let index_url = package_registry(&metadata, self.registry.registry.as_ref(), pkg)?;

            if is_published(&http_client, index_url, &name, version)? {
                info!("already published", name_ver);
                continue;
            }

            if self.dry_run {
                args.push("--dry-run");
            }

            if self.no_verify || self.dry_run {
                args.push("--no-verify");
            }

            if self.allow_dirty {
                args.push("--allow-dirty");
            }

            if let Some(ref registry) = self.registry.registry {
                args.push("--registry");
                args.push(registry);
            }

            if let Some(ref token) = self.registry.token {
                args.push("--token");
                args.push(token);
            }

            args.push("--manifest-path");
            args.push(p.as_str());

            let dev_deps_remover =
                if self.no_remove_dev_deps || !should_remove_dev_deps(&pkg.dependencies, &pkgs) {
                    None
                } else {
                    warn!(
                        "removing dev-deps since some refer to workspace members with versions",
                        name_ver
                    );
                    Some(DevDependencyRemover::remove_dev_deps(p.as_std_path())?)
                };

            let (_, stderr) = cargo(&metadata.workspace_root, &args, &[])?;

            drop(dev_deps_remover);

            if !stderr.contains("Uploading") || stderr.contains("error:") {
                if self.dry_run {
                    warn!("publish failed", name_ver);
                } else {
                    return Err(Error::Publish(name));
                }
            }

            if !self.dry_run {
                info!("published", name_ver);
            }
        }

        if !self.dry_run {
            info!("success", "ok");
        }
        Ok(())
    }

    fn build(&self, workspace_root: &Utf8PathBuf, manifest_path: &Utf8PathBuf) -> Result<bool> {
        let mut args = vec!["build"];

        args.push("--manifest-path");
        args.push(manifest_path.as_str());

        let (_stdout, stderr) = cargo(workspace_root, &args, &[])?;

        if stderr.contains("could not compile") {
            return Ok(false);
        }

        Ok(true)
    }
}
