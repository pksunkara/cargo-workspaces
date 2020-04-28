use crate::utils::Result;
use clap::Clap;
use console::Term;
use serde::Serialize;

pub trait Listable: Serialize {
    fn json(&self, term: &Term) -> Result {
        term.write_line(&serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    fn list(&self, term: &Term, list: ListOpt) -> Result;
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
