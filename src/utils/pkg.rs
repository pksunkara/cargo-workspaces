use crate::utils::{Error, ListOpt, Listable, Writer};
use cargo_metadata::{Metadata, PackageId};
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use std::{cmp::max, path::PathBuf};

#[derive(Serialize, Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Pkg {
    #[serde(skip)]
    pub id: PackageId,
    pub name: String,
    pub version: Version,
    pub location: PathBuf,
    #[serde(skip)]
    pub path: String,
    pub private: bool,
    pub independent: bool,
}

impl Listable for Vec<Pkg> {
    fn list(&self, mut w: Writer, list: ListOpt) -> Result<(), Error> {
        if list.json {
            return self.json(w);
        }

        let first = self.iter().map(|x| x.name.len()).max().unwrap();
        let second = self
            .iter()
            .map(|x| x.version.to_string().len() + 1)
            .max()
            .unwrap();
        let third = self.iter().map(|x| max(1, x.path.len())).max().unwrap();

        for pkg in self {
            w.none(&pkg.name)?;
            let mut width = first - pkg.name.len();

            if list.long {
                w.none(&format!("{:w$} ", "", w = width))?;
                w.green(&format!("v{}", pkg.version))?;
                w.none(&format!(
                    "{:w$} ",
                    "",
                    w = second - pkg.version.to_string().len() - 1
                ))?;

                if pkg.path.is_empty() {
                    w.br_black(".")?;
                } else {
                    w.br_black(&pkg.path)?;
                }

                width = third - pkg.path.len();
            }

            if list.all && pkg.private {
                w.none(&format!("{:w$} (", "", w = width))?;
                w.red("PRIVATE")?;
                w.none(")")?;
            }

            w.none("\n")?;
        }

        Ok(())
    }
}

fn is_independent(metadata: &Value) -> bool {
    if let Value::Object(v) = metadata {
        if let Some(Value::Object(v)) = v.get("workspaces") {
            if let Some(Value::Bool(v)) = v.get("independent") {
                return *v;
            }
        }
    }

    false
}

pub fn get_pkgs(metadata: &Metadata, all: bool) -> Result<Vec<Pkg>, Error> {
    let mut pkgs = vec![];

    for id in &metadata.workspace_members {
        if let Some(pkg) = metadata.packages.iter().find(|x| x.id == *id) {
            let private = pkg.publish.is_some() && pkg.publish.as_ref().unwrap().is_empty();

            if !all && private {
                continue;
            }

            let loc = pkg.manifest_path.strip_prefix(&metadata.workspace_root);

            if loc.is_err() {
                return Err(Error::PackageNotInWorkspace {
                    id: pkg.id.repr.clone(),
                    ws: metadata.workspace_root.to_string_lossy().to_string(),
                });
            }

            let loc = loc.unwrap().to_string_lossy();
            let loc = loc.trim_end_matches("Cargo.toml").trim_end_matches("/");

            pkgs.push(Pkg {
                id: pkg.id.clone(),
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                location: metadata.workspace_root.join(loc),
                path: loc.to_string(),
                private,
                independent: is_independent(&pkg.metadata),
            });
        } else {
            Error::PackageNotFound {
                id: id.repr.clone(),
            }
            .print()?;
        }
    }

    if pkgs.is_empty() {
        return Err(Error::EmptyWorkspace);
    }

    pkgs.sort();
    Ok(pkgs)
}
