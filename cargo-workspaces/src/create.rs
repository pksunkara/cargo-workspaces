use crate::utils::{cargo, change_versions, info, Error, Result, INTERNAL_ERR};

use camino::Utf8PathBuf;
use cargo_metadata::Metadata;
use clap::{ArgEnum, Parser};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use dunce::canonicalize;
use glob::Pattern;
use oclif::term::TERM_ERR;
use semver::Version;
use toml_edit::{Array, Document, Formatted, Item, Table, Value};

use std::{
    collections::BTreeMap as Map,
    env::current_dir,
    fs::{create_dir_all, read_to_string, remove_dir_all, remove_file, write},
    path::Path,
};

#[derive(Debug, Clone, ArgEnum)]
enum Edition {
    #[clap(name = "2015")]
    Fifteen,
    #[clap(name = "2018")]
    Eighteen,
    #[clap(name = "2021")]
    TwentyOne,
    #[clap(name = "2024")]
    TwentyFour,
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
        if canonicalize(&metadata.workspace_root)? != canonicalize(current_dir()?)? {
            return Err(Error::MustBeRunFromWorkspaceRoot);
        }

        let path = metadata.workspace_root.join(&self.path);

        if Path::new(&path).exists() {
            return Err(Error::PathAlreadyExists);
        }

        create_dir_all(&path)?;

        if !canonicalize(&path)?.starts_with(canonicalize(&metadata.workspace_root)?) {
            return Err(Error::InvalidMemberPath);
        }

        remove_dir_all(&path)?;

        let workspace_root = metadata.workspace_root.join("Cargo.toml");
        let backup = read_to_string(&workspace_root)?;

        self.try_run(metadata).or_else(|e| {
            // cleanup itself may fail and we want to notify the user in that case
            // otherwise just propagate the error that caused the cleanup
            cleanup(&workspace_root, backup, &self.path).and(Err(e))
        })?;

        info!("success", "ok");

        Ok(())
    }

    fn try_run(&self, metadata: Metadata) -> Result {
        self.add_workspace_toml_entry(&metadata)?;
        self.create_new_workspace_member(&metadata)?;

        Ok(())
    }

    // adds info about new member to workspace's Cargo.toml file
    //
    // # Fails if
    //
    // - toml files are generally corrupted
    // - exclude list contains new member's name
    // - members list contains new member's name
    fn add_workspace_toml_entry(&self, metadata: &Metadata) -> Result {
        let workspace_root = metadata.workspace_root.join("Cargo.toml");
        let mut workspace_manifest = read_to_string(&workspace_root)?.parse::<Document>()?;

        add_workspace_member(metadata, &mut workspace_manifest, &self.path)?;

        write(workspace_root, workspace_manifest.to_string())?;

        Ok(())
    }

    // creates new member crate
    //
    // # Fails if
    //
    // - conflicting options were chosen
    // - `cargo new` fails
    // - another package with the same name was already created somewhere
    fn create_new_workspace_member(&self, metadata: &Metadata) -> Result {
        let theme = ColorfulTheme::default();
        let path = metadata.workspace_root.join(&self.path);

        let name = match self.name.as_ref() {
            Some(n) => n.to_owned(),
            None => Input::with_theme(&theme)
                .default(path.file_name().map(|s| s.to_owned()).unwrap_or_default())
                .with_prompt("Name of the crate")
                .interact_text_on(&TERM_ERR)?,
        };

        let template = if self.lib {
            0
        } else if self.bin {
            1
        } else {
            Select::with_theme(&theme)
                .items(&["library", "binary"])
                .default(1)
                .with_prompt("Type of the crate")
                .interact_on(&TERM_ERR)?
        };

        let editions = Edition::value_variants()
            .iter()
            .map(|x| x.to_possible_value().unwrap().get_name())
            .collect::<Vec<_>>();

        let edition = match &self.edition {
            Some(edition) => match *edition {
                Edition::Fifteen => 0,
                Edition::Eighteen => 1,
                Edition::TwentyOne => 2,
                Edition::TwentyFour => 3,
            },
            None => Select::with_theme(&theme)
                .items(&editions)
                .default(2)
                .with_prompt("Rust edition")
                .interact_on(&TERM_ERR)?,
        };

        let mut args = vec!["new", "--name", &name, "--edition", editions[edition]];

        if template == 0 {
            args.push("--lib");
        } else {
            args.push("--bin");
        }

        args.push(path.as_str());

        let (stdout, stderr) = cargo(&metadata.workspace_root, &args, &[])?;

        if [&stdout, &stderr]
            .iter()
            .any(|out| out.contains("two packages"))
        {
            return Err(Error::DuplicatePackageName);
        }

        if !stderr.contains("Created") && !stderr.contains("Creating") {
            return Err(Error::Create);
        }

        let manifest = path.join("Cargo.toml");
        let mut versions = Map::new();

        versions.insert(
            name.to_owned(),
            Version::parse("0.0.0").expect(INTERNAL_ERR),
        );

        write(
            &manifest,
            change_versions(read_to_string(&manifest)?, &name, &versions, false)?,
        )?;

        Ok(())
    }
}

fn cleanup(workspace_root: &Utf8PathBuf, backup: String, path: &str) -> Result {
    // reset manifest doc
    remove_file(workspace_root)?;
    write(workspace_root, backup)?;

    // remove created crate, might not be there so ignore errors
    _ = remove_dir_all(path);

    // cleanup successful
    Ok(())
}

fn add_workspace_member(
    metadata: &Metadata,
    manifest: &mut Document,
    new_member_path: &str,
) -> Result {
    let path = metadata.workspace_root.join(new_member_path).to_string();

    let workspace_table = manifest
        .entry("workspace")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            Error::WorkspaceBadFormat("workspace manifest item must be a table".into())
        })?;

    if let Some(exclude_item) = workspace_table.get("exclude")
        && let Some(pattern) =
            exists_in_glob_list(metadata, exclude_item, &path, "workspace.exclude")?
        {
            return Err(Error::InWorkspaceExclude(pattern.into()));
        }

    let members_item = workspace_table
        .entry("members")
        .or_insert(Item::Value(Value::Array(Array::new())));

    // If the member is already in the members list, we don't need to do anything
    if exists_in_glob_list(metadata, members_item, &path, "workspace.members")?.is_some() {
        return Ok(());
    }

    let members_array = members_item.as_array_mut().expect(INTERNAL_ERR);

    let (prefix, suffix) = members_array
        .iter()
        .last()
        .map(|item| item.decor())
        .and_then(|decor| Some((decor.prefix()?.as_str()?, decor.suffix()?.as_str()?)))
        .unwrap_or(("\n    ", ",\n"));

    let new_elem =
        Value::String(Formatted::new(new_member_path.to_owned())).decorated(prefix, suffix);

    members_array.push_formatted(new_elem);

    Ok(())
}

fn exists_in_glob_list<'a>(
    metadata: &'a Metadata,
    array_item: &'a Item,
    path: &'a str,
    error_name: &'a str,
) -> Result<Option<&'a str>> {
    let paths = array_item
        .as_array()
        .ok_or_else(|| {
            Error::WorkspaceBadFormat(format!("{error_name} manifest item must be an array"))
        })?
        .iter()
        .map(|elem| {
            elem.as_str().ok_or_else(|| {
                Error::WorkspaceBadFormat(format!("{error_name} manifest items must be strings"))
            })
        })
        .collect::<Result<Vec<_>>>()?;

    for pattern in paths {
        if Pattern::new(&format!("{}/{pattern}", metadata.workspace_root))?.matches(path) {
            return Ok(Some(pattern));
        }
    }

    Ok(None)
}
