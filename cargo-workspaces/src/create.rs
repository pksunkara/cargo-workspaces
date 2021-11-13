use crate::utils::{cargo, change_versions, info, Error, Result, INTERNAL_ERR};

use cargo_metadata::Metadata;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use oclif::term::TERM_ERR;
use semver::Version;

use std::{collections::BTreeMap as Map, fs};

/// Create a new workspace crate
#[derive(Debug, Parser)]
pub struct Create {
    /// Path for the crate relative to the workspace manifest
    path: String,

    /// The crate edition, currently either 2015 or 2018
    #[clap(long)]
    edition: Option<String>,

    /// Whether this is a binary crate
    #[clap(long, conflicts_with = "lib")]
    bin: bool,

    /// Whether this is a library crate
    #[clap(long, conflicts_with = "bin")]
    lib: bool,

    /// The name of the crate
    #[clap(long)]
    name: Option<String>,
}

impl Create {
    pub fn run(&self, metadata: Metadata) -> Result {
        let theme = ColorfulTheme::default();
        let path = &metadata.workspace_root.join(&self.path);

        let name:String =
            match self.name.clone() {
                Some(n) => n,
                None => {
                    Input::with_theme(&theme)
                        .with_prompt("Name of the crate")
                        .interact_on(&TERM_ERR)?
                }
            };

        let types = vec!["library", "binary"];

        let template =
            if self.lib {
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

        let editions = vec!["2015", "2018"];

        let edition =
            match self.edition.clone() {
                Some(edition) => match edition.as_str() {
                    "2015" => 0,
                    "2018" => 1,
                    _ => {
                        Select::with_theme(&theme)
                            .items(&editions)
                            .default(1)
                            .with_prompt("Rust edition")
                            .interact_on(&TERM_ERR)?
                    }
                },
                None => {
                    Select::with_theme(&theme)
                        .items(&editions)
                        .default(1)
                        .with_prompt("Rust edition")
                        .interact_on(&TERM_ERR)?
                }
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

        info!("success", "ok");
        Ok(())
    }
}
