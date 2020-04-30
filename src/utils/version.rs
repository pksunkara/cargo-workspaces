use crate::utils::{
    change_versions, info, ChangeData, ChangeOpt, Error, GitOpt, Pkg, INTERNAL_ERR, TERM_ERR,
};
use cargo_metadata::Metadata;
use clap::Clap;
use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use semver::{Identifier, Version};
use std::collections::BTreeMap as Map;
use std::fs;
use std::process::exit;

#[derive(Debug, Clap)]
pub struct VersionOpt {
    #[clap(flatten)]
    pub change: ChangeOpt,

    #[clap(flatten)]
    pub git: GitOpt,
    // TODO: exact
}

impl VersionOpt {
    pub fn do_versioning(&self, metadata: &Metadata) -> Result<Map<String, Version>, Error> {
        let branch = self.git.validate(&metadata.workspace_root)?;

        let change_data = ChangeData::new(metadata, &self.change)?;

        if change_data.count == "0" && !change_data.dirty {
            TERM_ERR.write_line("Current HEAD is already released, skipping versioning")?;
            return Ok(Map::new());
        }

        let (mut changed_p, mut unchanged_p) =
            self.change
                .get_changed_pkgs(metadata, &change_data.since, false)?;

        if changed_p.is_empty() {
            TERM_ERR.write_line("No changes detected, skipping versioning")?;
            return Ok(Map::new());
        }

        let mut new_version = None;
        let mut new_versions = vec![];

        while !changed_p.is_empty() {
            get_new_versions(&metadata, changed_p, &mut new_version, &mut new_versions)?;

            let pkgs = unchanged_p.into_iter().partition::<Vec<_>, _>(|p| {
                let pkg = metadata
                    .packages
                    .iter()
                    .find(|x| x.name == p.name)
                    .expect(INTERNAL_ERR);

                pkg.dependencies.iter().any(|x| {
                    if let Some(version) = new_versions.iter().find(|y| x.name == y.0).map(|y| &y.1)
                    {
                        !x.req.matches(version)
                    } else {
                        false
                    }
                })
            });

            changed_p = pkgs.0;
            unchanged_p = pkgs.1;
        }

        let new_versions = confirm_versions(new_versions)?;

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
                    )?
                ),
            )?;
        }

        self.git.commit(
            &metadata.workspace_root,
            &new_version,
            &new_versions,
            branch,
        )?;

        Ok(new_versions)
    }
}

fn get_new_versions(
    metadata: &Metadata,
    pkgs: Vec<Pkg>,
    new_version: &mut Option<Version>,
    new_versions: &mut Vec<(String, Version, Version)>,
) -> Result<(), Error> {
    let (independent_pkgs, same_pkgs) = pkgs.into_iter().partition::<Vec<_>, _>(|p| p.independent);

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

        if new_version.is_none() {
            info!("current common version", cur_version)?;

            *new_version = Some(ask_version(cur_version, None)?);
        }

        for p in &same_pkgs {
            new_versions.push((
                p.name.to_string(),
                new_version.as_ref().expect(INTERNAL_ERR).clone(),
                cur_version.clone(),
            ));
        }
    }

    for p in &independent_pkgs {
        let new_version = ask_version(&p.version, Some(&p.name))?;
        new_versions.push((p.name.to_string(), new_version, p.version.clone()));
    }

    Ok(())
}

fn confirm_versions(
    versions: Vec<(String, Version, Version)>,
) -> Result<Map<String, Version>, Error> {
    let mut new_versions = Map::new();
    let style = Style::new().for_stderr();

    TERM_ERR.write_line("\nChanges:")?;

    for v in versions {
        TERM_ERR.write_line(&format!(
            " - {}: {} => {}",
            style.clone().yellow().apply_to(&v.0),
            v.2,
            style.clone().cyan().apply_to(&v.1),
        ))?;
        new_versions.insert(v.0, v.1);
    }

    TERM_ERR.write_line("")?;
    TERM_ERR.flush()?;

    let create = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you sure you want to create these versions?")
        .default(false)
        .interact_on(&TERM_ERR)?;

    if !create {
        exit(0);
    }

    Ok(new_versions)
}

fn ask_version(cur_version: &Version, pkg_name: Option<&str>) -> Result<Version, Error> {
    let mut items = version_items(cur_version);

    items.push(("Custom Prerelease".to_string(), None));
    items.push(("Custom Version".to_string(), None));

    let prompt = if let Some(name) = pkg_name {
        format!("for {} ", name)
    } else {
        "".to_string()
    };

    let theme = ColorfulTheme::default();

    let selected = Select::with_theme(&theme)
        .with_prompt(&format!(
            "Select a new version {}(currently {})",
            prompt, cur_version
        ))
        .items(&items.iter().map(|x| &x.0).collect::<Vec<_>>())
        .default(0)
        .interact_on(&TERM_ERR)?;

    let new_version = if selected == 6 {
        let custom = custom_pre(&cur_version);

        let preid = Input::with_theme(&theme)
            .with_prompt(&format!(
                "Enter a prerelease identifier (default: '{}', yielding {})",
                custom.0, custom.1
            ))
            .default(custom.0.to_string())
            .interact_on(&TERM_ERR)?;

        inc_preid(&cur_version, Identifier::AlphaNumeric(preid))
    } else if selected == 7 {
        Input::with_theme(&theme)
            .with_prompt("Enter a custom version")
            .interact_on(&TERM_ERR)?
    } else {
        items
            .get(selected)
            .expect(INTERNAL_ERR)
            .clone()
            .1
            .expect(INTERNAL_ERR)
    };

    Ok(new_version)
}

fn inc_pre(pre: &[Identifier]) -> Vec<Identifier> {
    match pre.get(0) {
        Some(Identifier::AlphaNumeric(id)) => {
            vec![Identifier::AlphaNumeric(id.clone()), Identifier::Numeric(0)]
        }
        Some(Identifier::Numeric(_)) => vec![Identifier::Numeric(0)],
        None => vec![
            Identifier::AlphaNumeric("alpha".to_string()),
            Identifier::Numeric(0),
        ],
    }
}

fn inc_preid(cur_version: &Version, preid: Identifier) -> Version {
    let mut version = cur_version.clone();

    if cur_version.pre.is_empty() {
        version.increment_patch();
        version.pre = vec![preid, Identifier::Numeric(0)];
    } else {
        match cur_version.pre.get(0).expect(INTERNAL_ERR) {
            Identifier::AlphaNumeric(id) => {
                version.pre = vec![preid.clone()];

                if preid.to_string() == id.to_string() {
                    match cur_version.pre.get(1) {
                        Some(Identifier::Numeric(n)) => {
                            version.pre.push(Identifier::Numeric(n + 1))
                        }
                        _ => version.pre.push(Identifier::Numeric(0)),
                    };
                } else {
                    version.pre.push(Identifier::Numeric(0));
                }
            }
            Identifier::Numeric(n) => {
                if preid.to_string() == n.to_string() {
                    version.pre = cur_version.pre.clone();

                    if let Some(Identifier::Numeric(n)) = version.pre.iter_mut().rfind(|x| {
                        if let Identifier::Numeric(_) = x {
                            true
                        } else {
                            false
                        }
                    }) {
                        *n = *n + 1;
                    }
                } else {
                    version.pre = vec![preid, Identifier::Numeric(0)];
                }
            }
        }
    }

    version
}

fn custom_pre(cur_version: &Version) -> (Identifier, Version) {
    let id = if let Some(id) = cur_version.pre.get(0) {
        id.clone()
    } else {
        Identifier::AlphaNumeric("alpha".to_string())
    };

    (id.clone(), inc_preid(cur_version, id))
}

fn version_items(cur_version: &Version) -> Vec<(String, Option<Version>)> {
    let mut items = vec![];

    let mut v = cur_version.clone();
    v.increment_patch();
    items.push((format!("Patch ({})", &v), Some(v)));

    let mut v = cur_version.clone();
    v.increment_minor();
    items.push((format!("Minor ({})", &v), Some(v)));

    let mut v = cur_version.clone();
    v.increment_major();
    items.push((format!("Major ({})", &v), Some(v)));

    let mut v = cur_version.clone();
    v.increment_patch();
    v.pre = inc_pre(&cur_version.pre);
    items.push((format!("Prepatch ({})", &v), Some(v)));

    let mut v = cur_version.clone();
    v.increment_minor();
    v.pre = inc_pre(&cur_version.pre);
    items.push((format!("Preminor ({})", &v), Some(v)));

    let mut v = cur_version.clone();
    v.increment_major();
    v.pre = inc_pre(&cur_version.pre);
    items.push((format!("Premajor ({})", &v), Some(v)));

    items
}

#[cfg(test)]
mod test_super {
    use super::*;

    #[test]
    fn test_inc_preid() {
        let v = inc_preid(
            &Version::parse("3.0.0").unwrap(),
            Identifier::AlphaNumeric("beta".to_string()),
        );
        assert_eq!(v.to_string(), "3.0.1-beta.0");
    }

    #[test]
    fn test_inc_preid_on_alpha() {
        let v = inc_preid(
            &Version::parse("3.0.0-alpha.19").unwrap(),
            Identifier::AlphaNumeric("beta".to_string()),
        );
        assert_eq!(v.to_string(), "3.0.0-beta.0");
    }

    #[test]
    fn test_inc_preid_on_num() {
        let v = inc_preid(
            &Version::parse("3.0.0-11.19").unwrap(),
            Identifier::AlphaNumeric("beta".to_string()),
        );
        assert_eq!(v.to_string(), "3.0.0-beta.0");
    }

    #[test]
    fn test_custom_pre() {
        let v = custom_pre(&Version::parse("3.0.0").unwrap());
        assert_eq!(v.0, Identifier::AlphaNumeric("alpha".to_string()));
        assert_eq!(v.1.to_string(), "3.0.1-alpha.0");
    }

    #[test]
    fn test_custom_pre_on_single_alpha() {
        let v = custom_pre(&Version::parse("3.0.0-a").unwrap());
        assert_eq!(v.0, Identifier::AlphaNumeric("a".to_string()));
        assert_eq!(v.1.to_string(), "3.0.0-a.0");
    }

    #[test]
    fn test_custom_pre_on_single_alpha_with_second_num() {
        let v = custom_pre(&Version::parse("3.0.0-a.11").unwrap());
        assert_eq!(v.0, Identifier::AlphaNumeric("a".to_string()));
        assert_eq!(v.1.to_string(), "3.0.0-a.12");
    }

    #[test]
    fn test_custom_pre_on_second_alpha() {
        let v = custom_pre(&Version::parse("3.0.0-a.b").unwrap());
        assert_eq!(v.0, Identifier::AlphaNumeric("a".to_string()));
        assert_eq!(v.1.to_string(), "3.0.0-a.0");
    }

    #[test]
    fn test_custom_pre_on_second_alpha_with_num() {
        let v = custom_pre(&Version::parse("3.0.0-a.b.1").unwrap());
        assert_eq!(v.0, Identifier::AlphaNumeric("a".to_string()));
        assert_eq!(v.1.to_string(), "3.0.0-a.0");
    }

    #[test]
    fn test_custom_pre_on_single_num() {
        let v = custom_pre(&Version::parse("3.0.0-11").unwrap());
        assert_eq!(v.0, Identifier::Numeric(11));
        assert_eq!(v.1.to_string(), "3.0.0-12");
    }

    #[test]
    fn test_custom_pre_on_single_num_with_second_alpha() {
        let v = custom_pre(&Version::parse("3.0.0-11.a").unwrap());
        assert_eq!(v.0, Identifier::Numeric(11));
        assert_eq!(v.1.to_string(), "3.0.0-12.a");
    }

    #[test]
    fn test_custom_pre_on_second_num() {
        let v = custom_pre(&Version::parse("3.0.0-11.20").unwrap());
        assert_eq!(v.0, Identifier::Numeric(11));
        assert_eq!(v.1.to_string(), "3.0.0-11.21");
    }

    #[test]
    fn test_custom_pre_on_multiple_num() {
        let v = custom_pre(&Version::parse("3.0.0-11.20.a.55.c").unwrap());
        assert_eq!(v.0, Identifier::Numeric(11));
        assert_eq!(v.1.to_string(), "3.0.0-11.20.a.56.c");
    }
}
