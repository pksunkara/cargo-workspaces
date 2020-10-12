use cargo_metadata::{DependencyKind, Package};
use indexmap::IndexSet as Set;
use std::{collections::BTreeMap as Map, path::PathBuf};

pub fn dag(pkgs: &[(Package, String)]) -> (Map<&PathBuf, (&Package, &String)>, Set<PathBuf>) {
    let mut names = Map::new();
    let mut visited = Set::new();

    for (pkg, version) in pkgs {
        names.insert(&pkg.manifest_path, (pkg, version));
        dag_insert(&pkgs, pkg, &mut visited);
    }

    (names, visited)
}

fn dag_insert(pkgs: &[(Package, String)], pkg: &Package, visited: &mut Set<PathBuf>) {
    if visited.contains(&pkg.manifest_path) {
        return;
    }

    for d in &pkg.dependencies {
        match d.kind {
            DependencyKind::Normal | DependencyKind::Build => {
                if let Some((dep, _)) = pkgs.iter().find(|(p, _)| d.name == p.name) {
                    dag_insert(pkgs, dep, visited);
                }
            }
            _ => {}
        }
    }

    visited.insert(pkg.manifest_path.clone());
}
