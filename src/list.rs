use crate::utils::Writer;
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
    // TODO:
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

                let path = pkg.manifest_path.strip_prefix(&metadata.workspace_root);

                if path.is_err() {
                    stderr.b_red("error")?;
                    stderr.none(": package ")?;
                    stderr.cyan(&pkg.id.repr)?;
                    stderr.none(" is not inside workspace ")?;
                    stderr.cyan(metadata.workspace_root.to_string_lossy().as_ref())?;

                    return Ok(());
                }

                let path = path.unwrap().to_string_lossy();
                let mut path = path.trim_end_matches("Cargo.toml").trim_end_matches("/");

                if path.is_empty() {
                    path = ".";
                }

                pkgs.push((
                    &pkg.name,
                    format!("v{}", pkg.version),
                    format!("{}", path),
                    private,
                ));
            } else {
                stderr.b_red("error")?;
                stderr.none(": unable to find package ")?;
                stderr.cyan(&id.repr)?;
            }
        }

        if pkgs.is_empty() {
            stderr.b_red("error")?;
            stderr.none(": found no packages")?;
            return Ok(());
        }

        pkgs.sort();

        let first = pkgs.iter().map(|x| x.0.len()).max().unwrap();
        let second = pkgs.iter().map(|x| x.1.len()).max().unwrap();
        let third = pkgs.iter().map(|x| x.2.len()).max().unwrap();

        for pkg in pkgs {
            stdout.none(pkg.0)?;
            let mut width = first - pkg.0.len();

            if self.long {
                stdout.none(&format!("{:w$} ", "", w = width))?;
                stdout.green(&pkg.1)?;
                stdout.none(&format!("{:w$} ", "", w = second - pkg.1.len()))?;
                stdout.br_black(&pkg.2)?;

                width = third - pkg.2.len();
            }

            if self.all && pkg.3 {
                stdout.none(&format!("{:w$} (", "", w = width))?;
                stdout.red("PRIVATE")?;
                stdout.none(")")?;
            }

            stdout.none("\n")?;
        }

        Ok(())
    }
}
