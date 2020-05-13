use crate::utils::{info, Result, VersionOpt};
use cargo_metadata::Metadata;
use clap::Clap;

/// Bump version of crates
#[derive(Debug, Clap)]
pub struct Version {
    #[clap(flatten)]
    version: VersionOpt,
}

impl Version {
    pub fn run(self, metadata: Metadata) -> Result {
        self.version.do_versioning(&metadata)?;

        info!("success", "ok")?;
        Ok(())
    }
}
