use crate::utils::{info, Error, Result, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::{AppSettings, Clap};
use std::process::Command;

/// Execute an arbitrary command in each crate
#[derive(Debug, Clap)]
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
        for p in &metadata.packages {
            let dir = p
                .manifest_path
                .parent()
                .ok_or(Error::ManifestHasNoParent(p.name.clone()))?;

            let status = Command::new(self.args.get(0).expect(INTERNAL_ERR))
                .args(&self.args[1..])
                .current_dir(dir)
                .status()?;

            if !self.no_bail && !status.success() {
                return Err(Error::Bail);
            }
        }

        info!("success", "ok")?;
        Ok(())
    }
}
