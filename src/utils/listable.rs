use crate::utils::{Result, TERM_OUT};
use clap::Clap;
use serde::Serialize;

pub trait Listable: Serialize {
    fn json(&self) -> Result {
        TERM_OUT.write_line(&serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    fn list(&self, list: ListOpt) -> Result;
}

#[derive(Debug, Clap)]
pub struct ListOpt {
    /// Show extended information
    #[clap(short, long)]
    pub long: bool,

    /// Show private crates that are normally hidden
    #[clap(short, long)]
    pub all: bool,

    /// Show information as a JSON array
    #[clap(long)]
    pub json: bool,
}
