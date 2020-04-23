use crate::utils::Writer;
use cargo_metadata::Metadata;
use clap::Clap;
use std::io::Result;

/// List local packages that have changed since the last tagged release
#[derive(Debug, Clap)]
pub struct Changed {
    /// Show extended information
    #[clap(short, long)]
    long: bool,

    /// Show private packages that are normally hidden
    #[clap(short, long)]
    all: bool,

    /// Show information as a JSON array
    #[clap(long)]
    json: bool,
}

impl Changed {
    pub fn run(self, metadata: Metadata, stdout: &mut Writer, stderr: &mut Writer) -> Result<()> {
        // let mut pkgs = vec![];

        Ok(())
    }
}
