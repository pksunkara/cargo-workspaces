use crate::utils::Error;
use cargo_metadata::Metadata;
use clap::Clap;
use console::Term;

/// Execute an arbitrary command in each crate
#[derive(Debug, Clap)]
pub struct Exec {}

impl Exec {
    pub fn run(&self, metadata: Metadata, stdout: &Term, stderr: &Term) -> Result<(), Error> {
        Ok(())
    }
}
