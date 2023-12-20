use crate::utils::{info, Error, Result};

use cargo_metadata::MetadataCommand;
use clap::Parser;
use dunce::canonicalize;
use glob::glob;

use std::{collections::HashSet, fs::write, path::PathBuf};

/// Initializes a new cargo workspace
#[derive(Debug, Parser)]
pub struct Init {
    /// Path to the workspace root
    #[clap(parse(from_os_str), default_value = ".")]
    path: PathBuf,
}

impl Init {
    pub fn run(&self) -> Result {
        if !self.path.is_dir() {
            return Err(Error::WorkspaceRootNotDir(
                self.path.to_string_lossy().to_string(),
            ));
        }

        let cargo_toml = self.path.join("Cargo.toml");

        // TODO: Append to existing toml file
        if cargo_toml.is_file() {
            return Err(Error::Init("'Cargo.toml' exists".into()));
        }

        // NOTE: Globset is not used here because it does not support file iterator
        let pkgs = glob(&format!("{}/**/Cargo.toml", self.path.display()))?.filter_map(|e| e.ok());

        let mut workspace_roots = HashSet::new();

        for path in pkgs {
            let metadata = MetadataCommand::default()
                .manifest_path(path)
                .exec()
                .map_err(|e| Error::Init(e.to_string()))?;

            workspace_roots.insert(metadata.workspace_root);
        }

        let ws = canonicalize(&self.path)?;

        let mut content = "[workspace]\nmembers = [".to_string();

        let mut members: Vec<_> = workspace_roots
            .iter()
            .filter_map(|m| m.strip_prefix(&ws).ok())
            .collect();

        members.sort();

        if !members.is_empty() {
            content.push('\n');
        }

        members
            .into_iter()
            .for_each(|m| content.push_str(&format!("    \"{m}\",\n")));

        content.push_str("]\n");

        write(cargo_toml, content)?;

        info!("initialized", self.path.display());
        Ok(())
    }
}
