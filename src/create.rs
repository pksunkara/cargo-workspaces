use crate::utils::{Error, TERM_ERR};
use cargo_metadata::Metadata;
use clap::Clap;

/// Create a new workspace crate
#[derive(Debug, Clap)]
pub struct Create {}

impl Create {
    pub fn run(&self, metadata: Metadata) -> Result<(), Error> {
        Ok(())
    }
}
