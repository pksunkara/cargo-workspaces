use crate::utils::Result;
use cargo_metadata::Metadata;
use clap::Clap;

/// Rename crates in the project
#[derive(Debug, Clap)]
pub struct Rename {}

impl Rename {
    pub fn run(self, metadata: Metadata) -> Result {
        Ok(())
    }
}
