use camino::Utf8PathBuf;
use cargo_metadata::{DependencyKind, Package};
use indexmap::IndexSet as Set;

use std::collections::BTreeMap as Map;

pub fn dag(
    pkgs: &[(Package, String)],
) -> (Map<&Utf8PathBuf, (&Package, &String)>, Set<Utf8PathBuf>) {
    // eprintln!("[dag] Starting DAG construction with {} packages", pkgs.len());

    let mut names = Map::new();
    let mut visited = Set::new();
    // let mut max_depth_seen = 0usize;

    for (_idx, (pkg, version)) in pkgs.iter().enumerate() {
        // eprintln!("[dag] Processing package {}/{}: {} ({})", _idx + 1, pkgs.len(), pkg.name, version);
        names.insert(&pkg.manifest_path, (pkg, version));
        let _depth = dag_insert(pkgs, pkg, &mut visited, 0);
        // if _depth > max_depth_seen {
        //     max_depth_seen = _depth;
        //     eprintln!("[dag] New max recursion depth: {}", max_depth_seen);
        // }
    }

    // eprintln!("[dag] DAG construction complete. Max depth: {}, Visited: {} packages", max_depth_seen, visited.len());
    (names, visited)
}

fn dag_insert(pkgs: &[(Package, String)], pkg: &Package, visited: &mut Set<Utf8PathBuf>, _depth: usize) -> usize {
    // Debug logging (uncomment to debug stack overflow issues):
    // if _depth % 10 == 0 || _depth > 50 {
    //     eprintln!("[dag_insert] depth={}, pkg={}, visited_count={}", _depth, pkg.name, visited.len());
    // }
    // if _depth > 100 {
    //     eprintln!("[dag_insert] WARNING: Very deep recursion depth={} for pkg={}. Stack overflow risk!", _depth, pkg.name);
    // }
    // if _depth > 500 {
    //     eprintln!("[dag_insert] CRITICAL: Extremely deep recursion depth={} for pkg={}. Likely infinite loop or circular dependency!", _depth, pkg.name);
    // }

    if visited.contains(&pkg.manifest_path) {
        // eprintln!("[dag_insert] depth={}, pkg={} already visited, returning", _depth, pkg.name);
        return _depth;
    }

    let mut max_child_depth = _depth;

    for d in &pkg.dependencies {
        // Only follow path dependencies (workspace members), not external registry deps
        // This prevents issues like const-serialize depending on const-serialize-07 = { package = "const-serialize", version = "0.7.2" }
        // where d.name would be "const-serialize" (the package name) but it's actually an external dep
        //
        // For workspace = true dependencies, cargo resolves them:
        // - If workspace defines it as path dep: path is Some(...)
        // - If workspace defines it as registry dep: path is None
        if d.path.is_none() {
            continue;
        }

        if let Some((dep, _)) = pkgs.iter().find(|(p, _)| d.name == p.name) {
            match d.kind {
                DependencyKind::Normal | DependencyKind::Build => {
                    // eprintln!("[dag_insert] depth={}, pkg={} -> recursing into dep={} (path={:?})",
                    //     _depth, pkg.name, dep.name, d.path);
                    let child_depth = dag_insert(pkgs, dep, visited, _depth + 1);
                    if child_depth > max_child_depth {
                        max_child_depth = child_depth;
                    }
                }
                _ => {}
            }
        }
    }

    visited.insert(pkg.manifest_path.clone());
    // eprintln!("[dag_insert] depth={}, pkg={} inserted into visited set", _depth, pkg.name);
    max_child_depth
}
