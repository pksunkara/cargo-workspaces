use std::{thread, time::Duration};

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

    /// Assert that `Cargo.lock` will remain unchanged
    #[clap(long)]
    locked: bool,

    /// Number of seconds to wait between publish attempts
    #[clap(long, value_name = "SECONDS")]
    publish_interval: Option<u64>,
}

impl Publish {
    pub fn run(mut self, metadata: Metadata) -> Result {
        eprintln!("[publish] Starting publish command");
        eprintln!("[publish] Options: dry_run={}, publish_as_is={}, skip_published={}, no_verify={}",
            self.dry_run, self.publish_as_is, self.skip_published, self.no_verify);

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

        eprintln!("[publish] Collecting packages...");
        let pkgs = if !self.publish_as_is {
            eprintln!("[publish] Running versioning...");
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
            eprintln!("[publish] Using packages as-is (no versioning)");
            metadata
                .packages
                .iter()
                .map(|x| (x.clone(), x.version.to_string()))
                .collect()
        };
        eprintln!("[publish] Collected {} packages", pkgs.len());
        for (idx, (pkg, ver)) in pkgs.iter().enumerate() {
            eprintln!("[publish]   {}: {} v{}", idx + 1, pkg.name, ver);
        }

        eprintln!("[publish] Building dependency DAG...");
        let (names, visited) = dag(&pkgs);
        eprintln!("[publish] DAG built. names={}, visited={}", names.len(), visited.len());

        // Filter out private packages
        eprintln!("[publish] Filtering private packages...");
        let visited = filter_private(visited, &pkgs);
        eprintln!("[publish] After filtering: {} packages to publish", visited.len());
        for (idx, path) in visited.iter().enumerate() {
            if let Some((pkg, ver)) = names.get(path) {
                eprintln!("[publish]   {}: {} v{}", idx + 1, pkg.name, ver);
            }
        }

        eprintln!("[publish] Creating HTTP client...");
        let http_client = create_http_client(&metadata.workspace_root, &self.registry.token)?;
        eprintln!("[publish] HTTP client created");

        eprintln!("[publish] Starting publish loop for {} packages", visited.len());
        for (pkg_idx, p) in visited.iter().enumerate() {
            let (pkg, version) = names.get(p).expect(INTERNAL_ERR);
            let name = pkg.name.clone();
            eprintln!("[publish] [{}/{}] Processing {} v{}", pkg_idx + 1, visited.len(), name, version);

            if self.dry_run {
                info!("checking", name);

                eprintln!("[publish] [{}/{}] Running build verification for {}", pkg_idx + 1, visited.len(), name);
                if !self.no_verify && !self.build(&metadata.workspace_root, p)? {
                    warn!("build failed", "");
                }

                eprintln!("[publish] [{}/{}] Running basic checks for {}", pkg_idx + 1, visited.len(), name);
                basic_checks(pkg)?;
            }

            let mut args = vec!["publish"];

            let name_ver = format!("{} v{}", name, version);
            eprintln!("[publish] [{}/{}] Getting package registry for {}", pkg_idx + 1, visited.len(), name);
            let index_url = package_registry(&metadata, self.registry.registry.as_ref(), pkg)?;
            eprintln!("[publish] [{}/{}] Registry URL obtained for {}", pkg_idx + 1, visited.len(), name);

            eprintln!("[publish] [{}/{}] Checking if {} v{} is already published...", pkg_idx + 1, visited.len(), name, version);
            if is_published(&http_client, index_url, &name, version)? {
                info!("already published", name_ver);
                eprintln!("[publish] [{}/{}] {} already published, skipping", pkg_idx + 1, visited.len(), name);
                continue;
            }
            eprintln!("[publish] [{}/{}] {} not yet published, proceeding", pkg_idx + 1, visited.len(), name);

            if self.dry_run {
                args.push("--dry-run");
            }

            if self.no_verify || self.dry_run {
                args.push("--no-verify");
            }

            if self.locked {
                args.push("--locked");
            }

            if let Some(ref registry) = self.registry.registry {
                args.push("--registry");
                args.push(registry);
            }

            if let Some(ref token) = self.registry.token {
                args.push("--token");
                args.push(token);
            }

            if let Some(interval) = self.publish_interval {
                if interval > 0 && !self.dry_run {
                    info!(
                        "waiting",
                        format!("{} seconds before publishing {}", interval, name_ver)
                    );
                    thread::sleep(Duration::from_secs(interval));
                }
            }

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

            if dev_deps_remover.is_some() || self.allow_dirty {
                args.push("--allow-dirty");
            }

            args.push("--manifest-path");
            args.push(p.as_str());

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

        info!("success", "ok");
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
