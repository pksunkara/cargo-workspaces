use crate::utils::{ChangeData, ChangeOpt, ListOpt, Listable, Result};

use cargo_metadata::Metadata;
use clap::{ArgSettings, Clap};
use oclif::term::TERM_OUT;

/// List crates that have changed since the last tagged release
#[derive(Debug, Clap)]
pub struct Changed {
    #[clap(flatten)]
    list: ListOpt,

    #[clap(flatten)]
    change: ChangeOpt,

    /// Use this git reference instead of the last tag
    #[clap(
        long,
        conflicts_with = "include-merged-tags",
        setting(ArgSettings::ForbidEmptyValues)
    )]
    since: Option<String>,
}

impl Changed {
    pub fn run(self, metadata: Metadata) -> Result {
        let mut since = self.since.clone();

        if self.since.is_none() {
            let change_data = ChangeData::new(&metadata, &self.change)?;

            if change_data.count == "0" {
                return Ok(TERM_OUT
                    .write_line("Current HEAD is already released, skipping change detection")?);
            }

            since = change_data.since;
        }

        let pkgs = self
            .change
            .get_changed_pkgs(&metadata, &since, self.list.all)?;

        pkgs.0.list(self.list)
    }
}
