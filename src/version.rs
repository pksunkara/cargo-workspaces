use crate::utils::{info, Error, VersionOpt};
use cargo_metadata::Metadata;
use clap::Clap;
use console::Term;

#[derive(Debug, Clap)]
pub struct Version {
    #[clap(flatten)]
    version: VersionOpt,
}

impl Version {
    pub fn run(self, metadata: Metadata, _: &Term, stderr: &Term) -> Result<(), Error> {
        self.version.do_versioning(&metadata, stderr)?;

        info!("success", "ok")?;
        Ok(())
    }
}
