use crate::utils::{Error, INTERNAL_ERR};
use dialoguer::{Input, Select};
use enquirer::ColoredTheme;
use semver::{Identifier, Version};

pub fn ask_version(cur_version: &Version, pkg_name: Option<String>) -> Result<Version, Error> {
    let mut items = version_items(cur_version);

    items.push(("Custom Prerelease".to_string(), None));
    items.push(("Custom Version".to_string(), None));

    let prompt = if let Some(name) = pkg_name {
        format!("for {} ", name)
    } else {
        "".to_string()
    };

    let selected = Select::with_theme(&ColoredTheme::default())
        .with_prompt(&format!(
            "Select a new version {}(currently {})",
            prompt, cur_version
        ))
        .items(&items.iter().map(|x| &x.0).collect::<Vec<_>>())
        .default(0)
        .interact()?;

    let new_version = if selected == 6 {
        let custom = custom_pre(&cur_version);

        let preid = Input::with_theme(&ColoredTheme::default())
            .with_prompt(&format!(
                "Enter a prerelease identifier (default: '{}', yielding {})",
                custom.0, custom.1
            ))
            .default(custom.0.to_string())
            .interact()?;

        inc_preid(&cur_version, Identifier::AlphaNumeric(preid))
    } else if selected == 7 {
        Input::with_theme(&ColoredTheme::default())
            .with_prompt("Enter a custom version")
            .interact()?
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
