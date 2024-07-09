use crate::utils::{
    basic_checks, cargo, cargo_config_get, create_http_client, dag, filter_private, info,
    is_published, should_remove_dev_deps, warn, DevDependencyRemover, Error, Result, VersionOpt,
    INTERNAL_ERR,
};

use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use clap::Parser;
use tame_index::IndexUrl;

/// Publish crates in the project
#[derive(Debug, Parser)]
#[clap(next_help_heading = "PUBLISH OPTIONS")]
pub struct Publish {
    #[clap(flatten, next_help_heading = None)]
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

    /// The token to use for publishing
    #[clap(long, forbid_empty_values(true))]
    token: Option<String>,

    /// The Cargo registry to use for publishing
    #[clap(long, forbid_empty_values(true))]
    registry: Option<String>,

    /// Don't remove dev-dependencies while publishing
    #[clap(long)]
    no_remove_dev_deps: bool,

    /// Perform checks without uploading. WIP and performs fewer
    /// checks than `cargo publish --dry-run`
    #[clap(long)]
    dry_run: bool,
}

impl Publish {
    pub fn run(mut self, metadata: Metadata) -> Result {
        if self.dry_run {
            warn!(
                "Dry run option is WIP and performs fewer checks than `cargo publish --dry-run`.",
                ""
            );
            if !self.publish_as_is {
                info!("Dry run doesn't perform versioning. Perform `version` command manually if required", "");
                info!("Skipping versioning step", "");
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

        let http_client = create_http_client(&metadata.workspace_root, &self.token)?;

        for p in &visited {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();

            if self.dry_run {
                info!("Checking package", name);
                if !self.no_verify {
                    self.try_build(&metadata.workspace_root, &name, p)?;
                } else {
                    info!("Skipping build", name);
                }
                basic_checks(pkg)?;
                info!("Can be published", name);
                continue;
            }

            let mut args = vec!["publish"];

            let name_ver = format!("{} v{}", name, version);

            let index_url = if let Some(registry) = self
                .registry
                .as_ref()
                .or_else(|| pkg.publish.as_deref().and_then(|x| x.get(0)))
            {
                let registry_url = cargo_config_get(
                    &metadata.workspace_root,
                    &format!("registries.{}.index", registry),
                )?;
                IndexUrl::NonCratesIo(registry_url.into())
            } else {
                IndexUrl::crates_io(None, None, None)?
            };

            if is_published(&http_client, index_url, &name, version)? {
                info!("already published", name_ver);
                continue;
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
                return Err(Error::Publish(name));
            }

            info!("published", name_ver);
        }

        info!("success", "ok");
        Ok(())
    }

    fn try_build(
        &self,
        workspace_root: &Utf8PathBuf,
        name: &str,
        manifest_path: &Utf8PathBuf,
    ) -> Result<()> {
        let mut args = vec!["build"];
        args.push("--manifest-path");
        args.push(manifest_path.as_str());

        let (_stdout, stderr) = cargo(workspace_root, &args, &[])?;
        if stderr.contains("could not compile") {
            return Err(Error::Build(name.to_string()));
        }
        Ok(())
    }
}
