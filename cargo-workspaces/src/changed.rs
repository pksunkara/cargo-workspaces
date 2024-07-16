use crate::utils::{list, ChangeData, ChangeOpt, Error, ListOpt, Result};

use cargo_metadata::Metadata;
use clap::Parser;
use oclif::term::TERM_OUT;

/// List crates that have changed since the last tagged release
#[derive(Debug, Parser)]
pub struct Changed {
    #[clap(flatten)]
    list: ListOpt,

    #[clap(flatten)]
    change: ChangeOpt,

    /// Use this git reference instead of the last tag
    #[clap(
        long,
        conflicts_with = "include-merged-tags",
        forbid_empty_values(true)
    )]
    since: Option<String>,

    /// Return non-zero exit code if no changes detected
    #[clap(long)]
    error_on_empty: bool,
}

impl Changed {
    pub fn run(self, metadata: Metadata) -> Result {
        let mut since = self.since.clone();

        if self.since.is_none() {
            let change_data = ChangeData::new(&metadata, &self.change)?;

            if change_data.count == "0" {
                TERM_OUT
                    .write_line("Current HEAD is already released, skipping change detection")?;
                return self.finish();
            }

            since = change_data.since;
        }

        let pkgs = self
            .change
            .get_changed_pkgs(&metadata, &since, self.list.all)?;

        if pkgs.0.is_empty() && self.error_on_empty {
            return self.finish();
        }

        list(&pkgs.0, self.list)
    }

    fn finish(self) -> Result {
        if self.error_on_empty {
            return Err(Error::NoChanges);
        }

        return Ok(());
    }
}
