use crate::utils::{
    ask_version, get_changed_pkgs, ChangeData, ChangeOpt, GitOpt, Result, Writer, INTERNAL_ERR,
};
use cargo_metadata::Metadata;
use clap::Clap;

#[derive(Clap, Debug)]
pub struct Version {
    #[clap(flatten)]
    change: ChangeOpt,

    #[clap(flatten)]
    git: GitOpt,
    // TODO: tag_version_prefix, exact
}

impl Version {
    pub fn run(self, metadata: Metadata, _: Writer, mut stderr: Writer) -> Result {
        self.git.validate(&metadata.workspace_root)?;

        let change_data = ChangeData::new(&metadata, &self.change)?;

        if change_data.count == "0" {
            return Ok(stderr.none("Current HEAD is already released, skipping versioning")?);
        }

        let pkgs = get_changed_pkgs(&metadata, &self.change, &change_data.since, false)?;

        let (independent_pkgs, same_pkgs) =
            pkgs.into_iter().partition::<Vec<_>, _>(|p| p.independent);

        if !same_pkgs.is_empty() {
            let cur_version = same_pkgs
                .iter()
                .map(|p| {
                    &metadata
                        .packages
                        .iter()
                        .find(|x| x.id == p.id)
                        .expect(INTERNAL_ERR)
                        .version
                })
                .max()
                .expect(INTERNAL_ERR);

            stderr.magenta("current version")?;
            stderr.none(" ")?;
            stderr.cyan(&format!("{}", cur_version))?;
            stderr.none("\n")?;

            let new_version = ask_version(cur_version, None)?;

            println!("{:#?}", new_version);
        }

        for p in independent_pkgs {
            let new_version = ask_version(&p.version, Some(p.name))?;

            println!("{:#?}", new_version);
        }

        Ok(())
    }
}
