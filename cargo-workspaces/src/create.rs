use crate::utils::{cargo, change_versions, info, Error, Result, INTERNAL_ERR};

use cargo_metadata::Metadata;
use clap::{ArgEnum, Parser};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use globset::Glob;
use oclif::term::TERM_ERR;
use semver::Version;
use toml_edit::{Array, Document, Item, Table, Value};

use std::{collections::BTreeMap as Map, fs, io};

#[derive(Debug, Clone, ArgEnum)]
enum Edition {
    #[clap(name = "2015")]
    Fifteen,
    #[clap(name = "2018")]
    Eighteen,
    #[clap(name = "2021")]
    TwentyOne,
}

/// Create a new workspace crate
#[derive(Debug, Parser)]
pub struct Create {
    /// Path for the crate relative to the workspace manifest
    path: String,

    /// The crate edition
    #[clap(long, arg_enum)]
    edition: Option<Edition>,

    /// Whether this is a binary crate
    #[clap(long, conflicts_with = "lib")]
    bin: bool,

    /// Whether this is a library crate
    #[clap(long)]
    lib: bool,

    /// The name of the crate
    #[clap(long)]
    name: Option<String>,
}

impl Create {
    pub fn run(&self, metadata: Metadata) -> Result {
        let theme = ColorfulTheme::default();
        let path = &metadata.workspace_root.join(&self.path);

        let name = match &self.name {
            Some(n) => n.clone(),
            None => Input::with_theme(&theme)
                .with_prompt("Name of the crate")
                .interact_on(&TERM_ERR)?,
        };

        let types = vec!["library", "binary"];

        let template = if self.lib {
            0
        } else if self.bin {
            1
        } else {
            Select::with_theme(&theme)
                .items(&types)
                .default(1)
                .with_prompt("Type of the crate")
                .interact_on(&TERM_ERR)?
        };

        let editions = Edition::value_variants()
            .iter()
            .map(|x| x.to_possible_value().unwrap().get_name())
            .collect::<Vec<_>>();

        let edition = match &self.edition {
            Some(edition) => match edition {
                &Edition::Fifteen => 0,
                &Edition::Eighteen => 1,
                &Edition::TwentyOne => 2,
            },
            None => Select::with_theme(&theme)
                .items(&editions)
                .default(2)
                .with_prompt("Rust edition")
                .interact_on(&TERM_ERR)?,
        };

        let mut args = vec![
            "new",
            "--name",
            name.as_str(),
            "--edition",
            editions[edition],
        ];

        if template == 0 {
            args.push("--lib");
        } else {
            args.push("--bin");
        }

        args.push(path.as_str());

        let created = cargo(&metadata.workspace_root, &args, &[])?;

        if !created.1.contains("Created") {
            return Err(Error::Create);
        }

        let manifest = path.join("Cargo.toml");
        let mut versions = Map::new();

        versions.insert(name.clone(), Version::parse("0.0.0").expect(INTERNAL_ERR));

        fs::write(
            &manifest,
            change_versions(fs::read_to_string(&manifest)?, &name, &versions, false)?,
        )?;

        // TODO: If none of the globs in workspace `members` match, add a new entry

        let workspace_manifest = &metadata.workspace_root.join("Cargo.toml");

        let mut workspace_manifest_document =
            std::fs::read_to_string(workspace_manifest)?.parse::<Document>()?;

        add_ws_member(&mut workspace_manifest_document, &self.path)?;

        fs::write(workspace_manifest, workspace_manifest_document.to_string())?;

        info!("success", "ok");
        Ok(())
    }
}

// Don't have an error type to use, just using Error::Io
fn add_ws_member(manifest: &mut Document, new_member: &str) -> Result {
    match manifest.get("workspace") {
        Some(item) => {
            if !item.is_table() {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "workspace manifest item must be a table",
                )));
            };
        }
        None => {
            manifest["workspace"] = Item::Table(Table::new());
        }
    };

    if let Some(item) = manifest["workspace"].get("exclude") {
        for exclude in item
            .as_array()
            .ok_or(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "workspace.exclude manifest item must be an array",
            )))?
            .iter()
            .map(|e| e.as_str())
        {
            let pattern = exclude.ok_or(Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "workspace.exclude manifest items must be strings",
            )))?;

            if Glob::new(pattern)?.compile_matcher().is_match(new_member) {
                return Ok(());
            }
        }
    }

    match manifest["workspace"].get("members") {
        Some(item) => {
            if !item.is_array() {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "workspace.members manifest item must be an array",
                )));
            }
        }
        None => {
            manifest["workspace"]["members"] = Item::Value(Value::Array(Array::new()));
        }
    }

    let members_list = manifest["workspace"]["members"].as_array_mut().unwrap();

    for member in members_list.iter() {
        match member.as_str() {
            Some(s) => {
                if Glob::new(s)?.compile_matcher().is_match(new_member) {
                    return Ok(());
                }
            }
            None => {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::Other,
                    "workspace.members manifest items must be strings",
                )));
            }
        }
    }

    members_list.push_formatted(new_member.into());

    Ok(())
}
