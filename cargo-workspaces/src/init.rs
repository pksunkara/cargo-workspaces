use crate::utils::info;
use crate::utils::{Error, Result};
use cargo_metadata::MetadataCommand;
use clap::Clap;
use glob::glob;
use std::{collections::HashSet, fs, path::PathBuf};

/// Create a new cargo workspace in an existing directory
#[derive(Debug, Clap)]
pub struct Init {
    /// Path to the workspace root
    #[clap(parse(from_os_str), default_value = ".")]
    path: PathBuf,
}

impl Init {
    pub fn run(&self) -> Result {
        if !self.path.is_dir() {
            return Err(Error::Init(format!(
                "the path `{}` does not exist",
                self.path.display()
            )));
        }

        let cargo_toml = self.path.join("Cargo.toml");

        if cargo_toml.is_file() {
            return Err(Error::Init(format!(
                "`init` cannot be run on existing Cargo packages."
            )));
        }

        let ws = fs::canonicalize(&self.path)?;

        let mut workspace_roots = HashSet::new();

        for path in glob(&format!("{}/**/Cargo.toml", self.path.display()))?.filter_map(|e| e.ok())
        {
            let metadata = MetadataCommand::default()
                .manifest_path(path)
                .exec()
                .map_err(|e| Error::Init(e.to_string()))?;
            workspace_roots.insert(metadata.workspace_root);
        }

        let mut content = "[workspace]\nmembers = [".to_string();

        let mut members: Vec<_> = workspace_roots
            .iter()
            .filter_map(|m| m.strip_prefix(&ws).ok())
            .collect();

        members.sort();
        if !members.is_empty() {
            content.push_str("\n");
        }

        members
            .into_iter()
            .for_each(|m| content.push_str(&format!("    \"{}\",\n", m.display())));
        content.push_str("]");

        fs::write(cargo_toml, content)?;

        info!(
            "success",
            format!("Initialized workspace `{}`.", self.path.display())
        )?;
        Ok(())
    }
}
