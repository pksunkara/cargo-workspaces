use crate::utils::{get_changed_pkgs, ChangeData, ChangeOpt, Result, Writer};
use cargo_metadata::Metadata;
use clap::Clap;

#[derive(Clap, Debug)]
pub struct Version {
    #[clap(flatten)]
    change: ChangeOpt,

    #[clap(long, default_value = "master")]
    allow_branch: String,

    #[clap(long, default_value = "origin")]
    git_remote: String,
}

impl Version {
    pub fn run(self, metadata: Metadata, mut stdout: Writer, mut stderr: Writer) -> Result {
        // print current version

        let change_data = ChangeData::new(&metadata, &self.change)?;

        if change_data.count == "0" {
            return Ok(stderr.none("Current HEAD is already released, skipping versioning")?);
        }

        let pkgs = get_changed_pkgs(&metadata, &self.change, &change_data.since, false)?;

        Ok(())
    }
}
