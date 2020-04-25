use crate::utils::{Error, Writer};
use clap::Clap;
use serde::Serialize;

pub trait Listable: Serialize {
    fn json(&self, mut w: Writer) -> Result<(), Error> {
        w.none(&serde_json::to_string_pretty(self)?)?;
        w.none("\n")?;

        Ok(())
    }

    fn list(&self, w: Writer, list: ListOpt) -> Result<(), Error>;
}

#[derive(Debug, Clap)]
pub struct ListOpt {
    /// Show extended information
    #[clap(short, long)]
    pub long: bool,

    /// Show private packages that are normally hidden
    #[clap(short, long)]
    pub all: bool,

    /// Show information as a JSON array
    #[clap(long)]
    pub json: bool,
}
