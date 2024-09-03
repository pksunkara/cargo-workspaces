use crate::utils::{info, Error, Result};

use cargo_metadata::MetadataCommand;
use clap::{ArgEnum, Parser};
use dunce::canonicalize;
use glob::glob;
use toml_edit::{Array, Document, Formatted, Item, Table, Value};

use std::{
    collections::HashSet,
    fs::{read_to_string, write},
    io::ErrorKind,
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, ArgEnum)]
enum Resolver {
    #[clap(name = "1")]
    V1,
    #[clap(name = "2")]
    V2,
}

impl Resolver {
    fn name(&self) -> &str {
        match self {
            Resolver::V1 => "1",
            Resolver::V2 => "2",
        }
    }
}

/// Initializes a new cargo workspace
#[derive(Debug, Parser)]
pub struct Init {
    /// Path to the workspace root
    #[clap(parse(from_os_str), default_value = ".")]
    path: PathBuf,

    /// Workspace feature resolver version
    #[clap(long, arg_enum)]
    resolver: Option<Resolver>,
}

impl Init {
    pub fn run(&self) -> Result {
        if !self.path.is_dir() {
            return Err(Error::WorkspaceRootNotDir(
                self.path.to_string_lossy().to_string(),
            ));
        }

        let cargo_toml = self.path.join("Cargo.toml");

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

        let mut document = match read_to_string(cargo_toml.as_path()) {
            Ok(manifest) => manifest.parse()?,
            Err(err) if err.kind() == ErrorKind::NotFound => Document::default(),
            Err(err) => return Err(err.into()),
        };

        let workspace = document
            .entry("workspace")
            .or_insert_with(|| Item::Table(Table::default()))
            .as_table_mut()
            .ok_or_else(|| {
                Error::WorkspaceBadFormat(
                    "no workspace table found in workspace Cargo.toml".to_string(),
                )
            })?;

        // workspace members
        {
            let workspace_members = workspace
                .entry("members")
                .or_insert_with(|| Item::Value(Value::Array(Array::new())))
                .as_array_mut()
                .ok_or_else(|| {
                    Error::WorkspaceBadFormat(
                        "members was not an array in workspace Cargo.toml".to_string(),
                    )
                })?;

            let mut members: Vec<_> = workspace_roots
                .iter()
                .filter_map(|m| m.strip_prefix(&ws).ok())
                .map(|path| path.to_string())
                .collect();

            members.sort();

            info!("crates", members.join(", "));

            let max_member = members.len().saturating_sub(1);

            workspace_members.extend(members.into_iter().enumerate().map(|(i, val)| {
                let prefix = "\n    ";
                let suffix = if i == max_member { ",\n" } else { "" };
                Value::String(Formatted::new(val)).decorated(prefix, suffix)
            }));
        }

        // workspace resolver
        if let Some(resolver) = self.resolver {
            workspace.entry("resolver").or_insert_with(|| {
                Item::Value(Value::String(Formatted::new(resolver.name().to_owned())))
            });
        }

        write(cargo_toml, document.to_string())?;

        info!("initialized", self.path.display());
        Ok(())
    }
}
