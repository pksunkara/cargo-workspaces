use crate::utils::{cargo, change_versions, info, Error, Result, INTERNAL_ERR};

use cargo_metadata::Metadata;
use clap::Clap;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use oclif::term::TERM_ERR;
use semver::Version;

use std::{collections::BTreeMap as Map, fs};

/// Create a new workspace crate
#[derive(Debug, Clap)]
pub struct Create {
    /// Path for the crate relative to the workspace manifest
    path: String,
}

impl Create {
    pub fn run(&self, metadata: Metadata) -> Result {
        let theme = ColorfulTheme::default();
        let path = &metadata.workspace_root.join(&self.path);
        let strpath = path.to_string_lossy().to_string();

        let name: String = Input::with_theme(&theme)
            .with_prompt("Name of the crate")
            .interact_on(&TERM_ERR)?;

        let types = vec!["library", "binary"];

        let template = Select::with_theme(&theme)
            .items(&types)
            .default(1)
            .with_prompt("Type of the crate")
            .interact_on(&TERM_ERR)?;

        let editions = vec!["2015", "2018"];

        let edition = Select::with_theme(&theme)
            .items(&editions)
            .default(1)
            .with_prompt("Rust edition")
            .interact_on(&TERM_ERR)?;

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

        args.push(strpath.as_str());

        let created = cargo(&metadata.workspace_root, &args)?;

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

        info!("success", "ok")?;
        Ok(())
    }
}
