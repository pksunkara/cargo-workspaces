use std::{
    fs::{read_to_string, write},
    path::Path,
};

use cargo_metadata::{Dependency, DependencyKind, Package};
use semver::VersionReq;
use toml_edit::Document;

use crate::utils::Result;

/// Removes all dev-dependencies from a Cargo.toml then restores the file when dropped.
pub struct DevDependencyRemover {
    manifest_path: std::path::PathBuf,
    original_toml: String,
}

impl DevDependencyRemover {
    pub fn remove_dev_deps(manifest_path: &Path) -> Result<Self> {
        let original_toml = read_to_string(manifest_path)?;
        let mut document = original_toml.parse::<Document>()?;

        document.as_table_mut().remove("dev-dependencies");

        if let Some(table) = document.as_table_mut().get_mut("target")
            && let Some(table) = table.as_table_mut() {
                table.iter_mut().for_each(|(_, value)| {
                    if let Some(table) = value.as_table_mut() {
                        table.remove("dev-dependencies");
                    }
                });
            }

        write(manifest_path, document.to_string())?;

        Ok(Self {
            manifest_path: manifest_path.to_owned(),
            original_toml,
        })
    }
}

impl Drop for DevDependencyRemover {
    fn drop(&mut self) {
        let _ = write(&self.manifest_path, &self.original_toml);
    }
}

pub fn should_remove_dev_deps(deps: &[Dependency], pkgs: &[(Package, String)]) -> bool {
    let mut names = vec![];
    let no_version = VersionReq::parse("*").unwrap();

    for (pkg, _) in pkgs {
        names.push(&pkg.name);
    }

    for dep in deps {
        if dep.kind == DependencyKind::Development
            && names.contains(&&dep.name)
            && dep.source.is_none()
            && dep.req != no_version
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir, read_to_string},
        path::PathBuf,
    };

    use cargo_metadata::MetadataCommand;

    use super::*;

    #[test]
    fn test_remove_dev_deps() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [dependencies]
        dep1 = "1.0.0"

        [dev-dependencies]
        dep2 = "2.0.1"

        [workspace.metadata.workspaces]
        "#;

        write(&manifest_path, original_toml).unwrap();

        let remover = DevDependencyRemover::remove_dev_deps(&manifest_path).unwrap();

        assert_eq!(
            read_to_string(&manifest_path).unwrap(),
            r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [dependencies]
        dep1 = "1.0.0"

        [workspace.metadata.workspaces]
        "#
        );

        drop(remover);

        assert_eq!(read_to_string(&manifest_path).unwrap(), original_toml);
    }

    #[test]
    fn test_remove_dev_deps_target() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [dependencies]
        dep1 = "1.0.0"

        [target.'cfg(unix)'.dev-dependencies]
        dep2 = "2.0.1"

        [workspace.metadata.workspaces]
        "#;

        write(&manifest_path, original_toml).unwrap();

        let remover = DevDependencyRemover::remove_dev_deps(&manifest_path).unwrap();

        assert_eq!(
            read_to_string(&manifest_path).unwrap(),
            r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [dependencies]
        dep1 = "1.0.0"

        [workspace.metadata.workspaces]
        "#
        );

        drop(remover);

        assert_eq!(read_to_string(&manifest_path).unwrap(), original_toml);
    }

    #[test]
    fn test_should_remove_dev_deps_normal() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [lib]
        path = "lib.rs"

        [dev-dependencies]
        syn = "2"
        "#;

        write(&manifest_path, original_toml).unwrap();

        let (deps, pkgs) = args(&manifest_path, "foo");

        assert!(!should_remove_dev_deps(&deps, &pkgs))
    }

    #[test]
    fn test_should_remove_dev_deps_member_path() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [lib]
        path = "lib.rs"

        [dev-dependencies]
        bar = { path = "./bar" }

        [workspace]
        members = [".", "bar"]
        "#;

        write(&manifest_path, original_toml).unwrap();

        let member_toml = r#"
        [package]
        name = "bar" # A comment
        version = "0.1.0"

        [lib]
        path = "lib.rs"
        "#;

        create_dir(tempdir.path().join("bar")).unwrap();
        write(tempdir.path().join("bar").join("Cargo.toml"), member_toml).unwrap();

        let (deps, pkgs) = args(&manifest_path, "foo");

        assert!(!should_remove_dev_deps(&deps, &pkgs))
    }

    #[test]
    fn test_should_remove_dev_deps_member_version() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = "0.1.0"

        [lib]
        path = "lib.rs"

        [dev-dependencies]
        bar = "0.1.0"

        [workspace]
        members = [".", "bar"]
        "#;

        write(&manifest_path, original_toml).unwrap();

        let member_toml = r#"
        [package]
        name = "bar" # A comment
        version = "0.1.0"

        [lib]
        path = "lib.rs"
        "#;

        create_dir(tempdir.path().join("bar")).unwrap();
        write(tempdir.path().join("bar").join("Cargo.toml"), member_toml).unwrap();

        let (deps, pkgs) = args(&manifest_path, "foo");

        // This won't remove it because it's reading from crates.io
        assert!(!should_remove_dev_deps(&deps, &pkgs))
    }

    #[test]
    fn test_should_remove_dev_deps_member_workspace_dependency() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = { workspace = true }

        [lib]
        path = "lib.rs"

        [dev-dependencies]
        bar = { workspace = true }

        [workspace]
        members = [".", "bar"]

        [workspace.package]
        version = "0.1.0"

        [workspace.dependencies]
        bar = { version = "0.1.0", path = "./bar" }
        "#;

        write(&manifest_path, original_toml).unwrap();

        let member_toml = r#"
        [package]
        name = "bar" # A comment
        version = { workspace = true }

        [lib]
        path = "lib.rs"
        "#;

        create_dir(tempdir.path().join("bar")).unwrap();
        write(tempdir.path().join("bar").join("Cargo.toml"), member_toml).unwrap();

        let (deps, pkgs) = args(&manifest_path, "foo");

        assert!(should_remove_dev_deps(&deps, &pkgs))
    }

    #[test]
    fn test_should_remove_dev_deps_member_workspace_dependency_target() {
        let tempdir = tempfile::tempdir().unwrap();
        let manifest_path = tempdir.path().join("Cargo.toml");

        let original_toml = r#"
        [package]
        name = "foo" # A comment
        version = { workspace = true }

        [lib]
        path = "lib.rs"

        [target.'cfg(unix)'.dev-dependencies]
        bar = { workspace = true }

        [workspace]
        members = [".", "bar"]

        [workspace.package]
        version = "0.1.0"

        [workspace.dependencies]
        bar = { version = "0.1.0", path = "./bar" }
        "#;

        write(&manifest_path, original_toml).unwrap();

        let member_toml = r#"
        [package]
        name = "bar" # A comment
        version = { workspace = true }

        [lib]
        path = "lib.rs"
        "#;

        create_dir(tempdir.path().join("bar")).unwrap();
        write(tempdir.path().join("bar").join("Cargo.toml"), member_toml).unwrap();

        let (deps, pkgs) = args(&manifest_path, "foo");

        assert!(should_remove_dev_deps(&deps, &pkgs))
    }

    fn args(manifest_path: &PathBuf, dep: &str) -> (Vec<Dependency>, Vec<(Package, String)>) {
        let mut cmd = MetadataCommand::new();

        cmd.manifest_path(manifest_path);
        cmd.no_deps();

        let metadata = cmd.exec().unwrap();

        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect();

        let pkg = metadata.packages.iter().find(|x| x.name == dep).unwrap();

        (pkg.dependencies.clone(), pkgs)
    }
}
