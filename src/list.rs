use crate::utils::Writer;
use cargo_metadata::Metadata;
use clap::Clap;
use serde::Serialize;
use std::{io::Result, path::PathBuf};

#[derive(Serialize, Ord, Eq, PartialOrd, PartialEq)]
struct Pkg {
    name: String,
    version: String,
    location: PathBuf,
    #[serde(skip)]
    path: String,
    private: bool,
}

/// List local packages
#[derive(Debug, Clap)]
pub struct List {
    /// Show extended information
    #[clap(short, long)]
    long: bool,

    /// Show private packages that are normally hidden
    #[clap(short, long)]
    all: bool,

    /// Show information as a JSON array
    #[clap(long)]
    json: bool,
}

impl List {
    pub fn run(self, metadata: Metadata) -> Result<()> {
        let mut stdout = Writer::new(false);
        let mut stderr = Writer::new(true);
        let mut pkgs = vec![];

        for id in metadata.workspace_members {
            if let Some(pkg) = metadata.packages.iter().find(|x| x.id == id) {
                let private = pkg.publish.is_some() && pkg.publish.as_ref().unwrap().is_empty();

                if !self.all && private {
                    continue;
                }

                let loc = pkg.manifest_path.strip_prefix(&metadata.workspace_root);

                if loc.is_err() {
                    stderr.b_red("error")?;
                    stderr.none(": package ")?;
                    stderr.cyan(&pkg.id.repr)?;
                    stderr.none(" is not inside workspace ")?;
                    stderr.cyan(metadata.workspace_root.to_string_lossy().as_ref())?;
                    stderr.none("\n")?;

                    return Ok(());
                }

                let loc = loc.unwrap().to_string_lossy();
                let mut loc = loc.trim_end_matches("Cargo.toml").trim_end_matches("/");

                if loc.is_empty() {
                    loc = ".";
                }

                pkgs.push(Pkg {
                    name: pkg.name.clone(),
                    version: format!("{}", pkg.version),
                    location: metadata.workspace_root.join(loc),
                    path: loc.to_string(),
                    private,
                });
            } else {
                stderr.b_red("error")?;
                stderr.none(": unable to find package ")?;
                stderr.cyan(&id.repr)?;
                stderr.none("\n")?;
            }
        }

        if pkgs.is_empty() {
            stderr.b_red("error")?;
            stderr.none(": found no packages\n")?;
            return Ok(());
        }

        pkgs.sort();

        if self.json {
            stdout.none(&serde_json::to_string_pretty(&pkgs)?)?;
            stdout.none("\n")?;
            return Ok(());
        }

        let first = pkgs.iter().map(|x| x.name.len()).max().unwrap();
        let second = pkgs.iter().map(|x| x.version.len() + 1).max().unwrap();
        let third = pkgs.iter().map(|x| x.path.len()).max().unwrap();

        for pkg in pkgs {
            stdout.none(&pkg.name)?;
            let mut width = first - pkg.name.len();

            if self.long {
                stdout.none(&format!("{:w$} ", "", w = width))?;
                stdout.green(&format!("v{}", pkg.version))?;
                stdout.none(&format!("{:w$} ", "", w = second - pkg.version.len() - 1))?;
                stdout.br_black(&pkg.path)?;

                width = third - pkg.path.len();
            }

            if self.all && pkg.private {
                stdout.none(&format!("{:w$} (", "", w = width))?;
                stdout.red("PRIVATE")?;
                stdout.none(")")?;
            }

            stdout.none("\n")?;
        }

        Ok(())
    }
}
