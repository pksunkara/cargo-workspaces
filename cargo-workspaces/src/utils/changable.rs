use crate::utils::{get_pkgs, git, info, Error, Pkg, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::Parser;
use globset::{Error as GlobsetError, Glob};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use toml::Value;

#[derive(Debug, Parser)]
pub struct ChangeOpt {
    // TODO: include_dirty
    /// Always include targeted crates matched by glob even when there are no changes
    #[clap(long, value_name = "PATTERN")]
    pub force: Option<String>,

    /// Ignore changes in files matched by glob
    #[clap(long, value_name = "PATTERN")]
    pub ignore_changes: Option<String>,

    /// Use this git reference instead of the last tag
    #[clap(long, forbid_empty_values(true))]
    pub since: Option<String>,
}

#[derive(Debug, Default)]
pub struct ChangeData {
    pub since: Option<String>,
    pub count: String,
    pub dirty: bool,
}

impl ChangeData {
    pub fn new(metadata: &Metadata, _change: &ChangeOpt) -> Result<Self, Error> {
        let (_, sha, _) = git(
            &metadata.workspace_root,
            &["rev-list", "--tags", "--max-count=1"],
        )?;

        if sha.is_empty() {
            return Ok(Self {
                count: "1".to_string(),
                since: None,
                ..Default::default()
            });
        }

        let (_, count, _) = git(
            &metadata.workspace_root,
            &["rev-list", "--count", &format!("HEAD...{sha}")],
        )?;

        let since = git(
            &metadata.workspace_root,
            &["describe", "--exact-match", "--tags", &sha],
        )
        .ok()
        .map(|x| x.1);

        Ok(Self {
            count,
            since,
            ..Default::default()
        })
    }
}

impl ChangeOpt {
    pub fn get_changed_pkgs(
        &self,
        metadata: &Metadata,
        // Optional because there can be no tags
        since: &Option<String>,
        private: bool,
    ) -> Result<(Vec<Pkg>, Vec<Pkg>), Error> {
        let pkgs = get_pkgs(metadata, private)?;

        let pkgs = if let Some(since) = since {
            info!("looking for changes since", since);

            let (_, changed_files, _) = git(
                &metadata.workspace_root,
                &["diff", "--name-only", "--relative", since],
            )?;

            let changed_files_vec = changed_files
                .split('\n')
                .filter(|f| !f.is_empty())
                .map(Path::new)
                .collect::<Vec<_>>();

            // --- Logic for workspace.dependencies changes ---
            let mut changed_workspace_deps: HashSet<String> = HashSet::new();
            let root_cargo_toml_path = metadata.workspace_root.join("Cargo.toml").into_std_path_buf();

            if changed_files_vec.iter().any(|f| *f == root_cargo_toml_path.as_path()) {
                info!("root Cargo.toml changed, checking workspace.dependencies");
                // Fetch old Cargo.toml content
                let old_root_toml_content = match git(
                    &metadata.workspace_root,
                    &["show", &format!("{}:Cargo.toml", since)],
                ) {
                    Ok((_, stdout, _)) => stdout,
                    Err(e) => {
                        // It's possible Cargo.toml didn't exist in the old commit
                        eprintln!("Warning: Failed to get old root Cargo.toml: {}", e);
                        String::new()
                    }
                };

                let old_toml: Value = old_root_toml_content.parse().unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to parse old root Cargo.toml: {}", e);
                    Value::Table(Default::default())
                });

                // Parse current Cargo.toml
                let current_root_toml_content = fs::read_to_string(&root_cargo_toml_path)
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to read current root Cargo.toml: {}", e);
                        String::new()
                    });
                let current_toml: Value = current_root_toml_content.parse().unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to parse current root Cargo.toml: {}", e);
                    Value::Table(Default::default())
                });

                let old_ws_deps = old_toml
                    .get("workspace")
                    .and_then(|v| v.get("dependencies"))
                    .and_then(|v| v.as_table());

                let current_ws_deps = current_toml
                    .get("workspace")
                    .and_then(|v| v.get("dependencies"))
                    .and_then(|v| v.as_table());

                match (old_ws_deps, current_ws_deps) {
                    (Some(old_deps), Some(new_deps)) => {
                        for (dep_name, old_val) in old_deps {
                            if !new_deps.contains_key(dep_name) || new_deps[dep_name] != *old_val {
                                changed_workspace_deps.insert(dep_name.clone());
                            }
                        }
                        for (dep_name, new_val) in new_deps {
                            if !old_deps.contains_key(dep_name) || old_deps[dep_name] != *new_val {
                                // Ensure not already added if value changed
                                changed_workspace_deps.insert(dep_name.clone());
                            }
                        }
                    }
                    (None, Some(new_deps)) => { // All new deps are changes
                        for dep_name in new_deps.keys() {
                            changed_workspace_deps.insert(dep_name.clone());
                        }
                    }
                    (Some(old_deps), None) => { // All old deps are changes (removed)
                        for dep_name in old_deps.keys() {
                            changed_workspace_deps.insert(dep_name.clone());
                        }
                    }
                    (None, None) => { /* No workspace.dependencies in either, no changes here */ }
                }
                if !changed_workspace_deps.is_empty() {
                    info!("changed workspace dependencies: {:?}", changed_workspace_deps);
                }
            }
            // --- End of logic for workspace.dependencies changes ---

            let force = self
                .force
                .clone()
                .map(|x| Glob::new(&x))
                .map_or::<Result<_, GlobsetError>, _>(Ok(None), |x| Ok(x.ok()))?;
            let ignore_changes = self
                .ignore_changes
                .clone()
                .map(|x| Glob::new(&x))
                .map_or::<Result<_, GlobsetError>, _>(Ok(None), |x| Ok(x.ok()))?;

            pkgs.into_iter().partition(|p: &Pkg| {
                if let Some(pattern) = &force {
                    if pattern.compile_matcher().is_match(&p.name) {
                        return true;
                    }
                }

                // Use a HashSet to collect names of changed packages to avoid duplicates
                let mut changed_pkg_names: HashSet<String> = HashSet::new();

                for p in &pkgs {
                    if let Some(pattern) = &force {
                        if pattern.compile_matcher().is_match(&p.name) {
                            changed_pkg_names.insert(p.name.clone());
                            continue; // Force included, no need for other checks
                        }
                    }

                    let mut is_changed_by_file = false;
                    for f in &changed_files_vec {
                        if let Some(pattern) = &ignore_changes {
                            if pattern
                                .compile_matcher()
                                .is_match(f.to_str().expect(INTERNAL_ERR))
                            {
                                continue; // Ignored file, skip
                            }
                        }
                        if f.starts_with(&p.path) {
                            is_changed_by_file = true;
                            break;
                        }
                    }

                    if is_changed_by_file {
                        changed_pkg_names.insert(p.name.clone());
                        continue;
                    }

                    // If not changed by file, check workspace dependencies
                    if !changed_workspace_deps.is_empty() {
                        // p.manifest_path is the canonical path to the Cargo.toml file.
                        let manifest_to_read = p.manifest_path.as_std_path();

                        match fs::read_to_string(manifest_to_read) {
                            Ok(content) => {
                                match content.parse::<Value>() {
                                    Ok(pkg_toml) => {
                                        let mut check_deps = |deps_table_key: &str| -> bool {
                                            if let Some(deps) = pkg_toml.get(deps_table_key).and_then(|v| v.as_table()) {
                                                for (dep_name, dep_details) in deps {
                                                    // If `dep_details` has `workspace = true`, then `dep_name` is the key
                                                    // that should exist in the root `[workspace.dependencies]`.
                                                    if dep_details.as_table().and_then(|t| t.get("workspace")).and_then(|v| v.as_bool()) == Some(true) {
                                                        if changed_workspace_deps.contains(dep_name) {
                                                            return true; // Package uses a changed workspace dependency
                                                        }
                                                    }
                                                }
                                            }
                                            false
                                        };

                                        if check_deps("dependencies") || check_deps("dev-dependencies") {
                                            changed_pkg_names.insert(p.name.clone());
                                        }
                                    }
                                    Err(e) => eprintln!("Warning: Failed to parse Cargo.toml for {}: {}", p.name, e),
                                }
                            }
                            Err(e) => eprintln!("Warning: Failed to read Cargo.toml for {}: {}", p.name, e),
                        }
                    }
                }
                
                pkgs.into_iter().partition(|p| changed_pkg_names.contains(&p.name))
            })
        } else {
            // No 'since' ref, so all packages are considered "changed" relative to nothing.
            // The original code returns (pkgs, vec![]), implying all are new/changed.
            (pkgs, vec![])
        };

        Ok(pkgs)
    }
}
