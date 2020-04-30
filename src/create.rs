use crate::utils::Error;
use cargo_metadata::Metadata;
use clap::Clap;
use console::Term;

/// Create a new workspace crate
#[derive(Debug, Clap)]
pub struct Create {}

impl Create {
    pub fn run(&self, metadata: Metadata, stdout: &Term, stderr: &Term) -> Result<(), Error> {
        Ok(())
    }
}
