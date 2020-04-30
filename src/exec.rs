use crate::utils::Error;
use cargo_metadata::Metadata;
use clap::Clap;

/// Execute an arbitrary command in each crate
#[derive(Debug, Clap)]
pub struct Exec {}

impl Exec {
    pub fn run(&self, metadata: Metadata) -> Result<(), Error> {
        Ok(())
    }
}
