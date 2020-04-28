use crate::utils::{
    ask_version, change_versions, confirm_versions, get_changed_pkgs, ChangeData, ChangeOpt, Error,
    GitOpt, Pkg, INTERNAL_ERR,
};
use cargo_metadata::Metadata;
use clap::Clap;
use console::{Style, Term};
use std::collections::BTreeMap as Map;
use std::fs;

#[derive(Clap, Debug)]
pub struct Version {
    #[clap(flatten)]
    change: ChangeOpt,

    #[clap(flatten)]
    git: GitOpt,
    // TODO: tag_version_prefix, exact
}

impl Version {
    pub fn run(self, metadata: Metadata, _: &Term, stderr: &Term) -> Result<(), Error> {
        self.git.validate(&metadata.workspace_root)?;

        let change_data = ChangeData::new(&metadata, &self.change)?;

        if change_data.count == "0" {
            return Ok(stderr.write_line("Current HEAD is already released, skipping versioning")?);
        }

        let pkgs = get_changed_pkgs(&metadata, &self.change, &change_data.since, false)?;

        if pkgs.is_empty() {
            return Ok(stderr.write_line("No changes detected, skipping versioning")?);
        }

        let new_versions = Self::get_new_versions(&metadata, pkgs, stderr)?;

        for p in &metadata.packages {
            if new_versions.get(&p.name).is_none()
                && p.dependencies
                    .iter()
                    .all(|x| new_versions.get(&x.name).is_none())
            {
                continue;
            }

            fs::write(
                &p.manifest_path,
                format!(
                    "{}\n",
                    change_versions(
                        fs::read_to_string(&p.manifest_path)?,
                        &p.name,
                        &new_versions,
                    )
                ),
            )?;
        }

        Ok(())
    }

    fn get_new_versions(
        metadata: &Metadata,
        pkgs: Vec<Pkg>,
        stderr: &Term,
    ) -> Result<Map<String, semver::Version>, Error> {
        let mut new_versions = vec![];

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

            let style = Style::new().for_stderr();

            stderr.write_line(&format!(
                "{} {}",
                style.clone().magenta().apply_to("current version"),
                style.cyan().apply_to(cur_version)
            ))?;

            let new_version = ask_version(cur_version, None, stderr)?;

            for p in &same_pkgs {
                new_versions.push((p.name.to_string(), new_version.clone(), cur_version));
            }
        }

        for p in &independent_pkgs {
            let new_version = ask_version(&p.version, Some(&p.name), stderr)?;
            new_versions.push((p.name.to_string(), new_version, &p.version));
        }

        confirm_versions(new_versions, stderr)
    }
}
