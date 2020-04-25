use crate::utils::{get_changed_pkgs, ChangeData, ChangeOpt, ListOpt, Listable, Result, Writer};
use cargo_metadata::Metadata;
use clap::Clap;

/// List local packages that have changed since the last tagged release
#[derive(Debug, Clap)]
pub struct Changed {
    #[clap(flatten)]
    list: ListOpt,

    #[clap(flatten)]
    change: ChangeOpt,

    /// Use this git reference instead of the last tag
    #[clap(long, conflicts_with = "include-merged-tags")]
    since: Option<String>,
}

impl Changed {
    pub fn run(self, metadata: Metadata, stdout: Writer, mut stderr: Writer) -> Result {
        let mut since = self.since.clone();

        if self.since.is_none() {
            let change_data = ChangeData::new(&metadata, &self.change)?;

            if change_data.count == "0" {
                return Ok(
                    stderr.none("Current HEAD is already released, skipping change detection")?
                );
            }

            since = change_data.since;
        }

        let pkgs = get_changed_pkgs(&metadata, &self.change, &since, self.list.all)?;

        pkgs.list(stdout, self.list)
    }
}
