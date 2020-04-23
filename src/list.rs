use crate::utils::{Listable, Pkg, Writer};
use cargo_metadata::Metadata;
use clap::Clap;
use std::io::Result;

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
    pub fn run(self, metadata: Metadata, stdout: &mut Writer, stderr: &mut Writer) -> Result<()> {
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
                let loc = loc.trim_end_matches("Cargo.toml").trim_end_matches("/");

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
        pkgs.list(stdout, self.json, self.long, self.all)
    }
}
