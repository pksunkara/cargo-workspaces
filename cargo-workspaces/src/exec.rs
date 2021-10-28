use crate::utils::{dag, info, Error, Result, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::{AppSettings, Parser};
use std::process::Command;

/// Execute an arbitrary command in each crate
#[derive(Debug, Parser)]
#[clap(setting = AppSettings::TrailingVarArg)]
pub struct Exec {
    /// Continue executing command despite non-zero exit in a given crate
    #[clap(long)]
    no_bail: bool,

    #[clap(required = true)]
    args: Vec<String>,
}

impl Exec {
    pub fn run(&self, metadata: Metadata) -> Result {
        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect::<Vec<_>>();

        let (names, visited) = dag(&pkgs);

        for p in &visited {
            let (pkg, _) = names.get(p).expect(INTERNAL_ERR);

            let dir = pkg
                .manifest_path
                .parent()
                .ok_or_else(|| Error::ManifestHasNoParent(pkg.name.clone()))?;

            let status = Command::new(self.args.get(0).expect(INTERNAL_ERR))
                .args(&self.args[1..])
                .current_dir(dir)
                .status()?;

            if !self.no_bail && !status.success() {
                return Err(Error::Bail);
            }
        }

        info!("success", "ok");
        Ok(())
    }
}
