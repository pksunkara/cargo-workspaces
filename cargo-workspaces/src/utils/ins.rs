use cargo_metadata::{DependencyKind, Package};
use indexmap::IndexSet as Set;
use std::path::PathBuf;

pub fn ins(pkgs: &[(Package, String)], pkg: &Package, visited: &mut Set<PathBuf>) {
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
