use crate::utils::{Error, VersionOpt};
use cargo_metadata::Metadata;
use clap::Clap;
use console::Term;

#[derive(Clap, Debug)]
pub struct Publish {
    #[clap(flatten)]
    version: VersionOpt,

    #[clap(long)]
    from_git: bool,
}

impl Publish {
    pub fn run(self, metadata: Metadata, stdout: &Term, stderr: &Term) -> Result<(), Error> {
        if !self.from_git {
            self.version.do_versioning(&metadata, stderr)?;
        }

        Ok(())
    }
}
