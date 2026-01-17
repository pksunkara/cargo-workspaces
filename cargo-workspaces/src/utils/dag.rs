use camino::Utf8PathBuf;
use cargo_metadata::{DependencyKind, Package};
use indexmap::IndexSet as Set;

use std::collections::BTreeMap as Map;

pub fn dag(
    pkgs: &[(Package, String)],
) -> (Map<&Utf8PathBuf, (&Package, &String)>, Set<Utf8PathBuf>) {
    let mut names = Map::new();
    let mut visited = Set::new();

    for (pkg, version) in pkgs {
        names.insert(&pkg.manifest_path, (pkg, version));
        dag_insert(pkgs, pkg, &mut visited);
    }

    (names, visited)
}

fn dag_insert(pkgs: &[(Package, String)], pkg: &Package, visited: &mut Set<Utf8PathBuf>) {
    if visited.contains(&pkg.manifest_path) {
        return;
    }

    for d in &pkg.dependencies {
        // Only follow path dependencies (workspace members), not external registry deps.
        // This prevents issues where a renamed external dep like:
        //   const-serialize-07 = { package = "const-serialize", version = "0.7.2" }
        // would match a workspace package with the same name, causing infinite recursion.
        if d.path.is_none() {
            continue;
        }

        if let Some((dep, _)) = pkgs.iter().find(|(p, _)| d.name == p.name) {
            match d.kind {
                DependencyKind::Normal | DependencyKind::Build => {
                    dag_insert(pkgs, dep, visited);
                }
                _ => {}
            }
        }
    }

    visited.insert(pkg.manifest_path.clone());
}
