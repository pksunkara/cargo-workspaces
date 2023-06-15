use crate::utils::Result;
use std::path::Path;

/// Removes all dev-dependencies from a Cargo.toml then restores the file when dropped.
pub(crate) struct DevDependencyRemover {
    manifest_path: std::path::PathBuf,
    original_toml: String,
}

impl DevDependencyRemover {
    pub(crate) fn remove_dev_deps(manifest_path: &Path) -> Result<Self> {
        let original_toml = std::fs::read_to_string(manifest_path)?;
        let mut document: toml_edit::Document = original_toml.parse()?;
        document.as_table_mut().remove("dev-dependencies");
        std::fs::write(manifest_path, document.to_string())?;
        Ok(Self {
            manifest_path: manifest_path.to_owned(),
            original_toml,
        })
    }
}

impl Drop for DevDependencyRemover {
    fn drop(&mut self) {
        let _ = std::fs::write(&self.manifest_path, &self.original_toml);
    }
}

#[test]
fn test_remove_dev_deps() {
    let tempdir = tempfile::tempdir().unwrap();
    let manifest_path = tempdir.path().join("Cargo.toml");
    let original_toml = r#"
        [package]
        name = "test-crate" # A comment
        version = "0.1.0"
        
        [dependencies]
        dep1 = "1.0.0"
        
        [dev-dependencies]
        dep2 = "2.0.1"

        [workspace.metadata.workspaces]
        "#;
    std::fs::write(&manifest_path, original_toml).unwrap();

    let remover = DevDependencyRemover::remove_dev_deps(&manifest_path).unwrap();

    assert_eq!(
        std::fs::read_to_string(&manifest_path).unwrap(),
        r#"
        [package]
        name = "test-crate" # A comment
        version = "0.1.0"
        
        [dependencies]
        dep1 = "1.0.0"

        [workspace.metadata.workspaces]
        "#
    );

    drop(remover);

    assert_eq!(
        std::fs::read_to_string(&manifest_path).unwrap(),
        original_toml
    );
}
