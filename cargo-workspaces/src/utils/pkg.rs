use crate::utils::{Config, Error, ListOpt, Listable, Result, INTERNAL_ERR};

use cargo_metadata::{Metadata, PackageId};
use oclif::{console::style, term::TERM_OUT, CliError};
use semver::Version;
use serde::Serialize;

use std::{
    cmp::max,
    path::{Path, PathBuf},
};

#[derive(Serialize, Debug, Clone, Ord, Eq, PartialOrd, PartialEq)]
pub struct Pkg {
    #[serde(skip)]
    pub id: PackageId,
    pub name: String,
    pub version: Version,
    pub location: PathBuf,
    #[serde(skip)]
    pub path: PathBuf,
    pub private: bool,
    #[serde(skip)]
    pub config: Config,
}

impl Listable for Vec<Pkg> {
    fn list(&self, list: ListOpt) -> Result {
        if list.json {
            return self.json();
        }

        if self.is_empty() {
            return Ok(());
        }

        let first = self.iter().map(|x| x.name.len()).max().expect(INTERNAL_ERR);
        let second = self
            .iter()
            .map(|x| x.version.to_string().len() + 1)
            .max()
            .expect(INTERNAL_ERR);
        let third = self
            .iter()
            .map(|x| max(1, x.path.as_os_str().len()))
            .max()
            .expect(INTERNAL_ERR);

        for pkg in self {
            TERM_OUT.write_str(&pkg.name)?;
            let mut width = first - pkg.name.len();

            if list.long {
                let path = if pkg.path.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    pkg.path.as_path()
                };

                TERM_OUT.write_str(&format!(
                    "{:f$} {}{:s$} {}",
                    "",
                    style(format!("v{}", pkg.version)).green(),
                    "",
                    style(path.display()).black().bright(),
                    f = width,
                    s = second - pkg.version.to_string().len() - 1,
                ))?;

                width = third - pkg.path.as_os_str().len();
            }

            if list.all && pkg.private {
                TERM_OUT.write_str(&format!(
                    "{:w$} ({})",
                    "",
                    style("PRIVATE").red(),
                    w = width
                ))?;
            }

            TERM_OUT.write_line("")?;
        }

        Ok(())
    }
}

pub fn get_pkgs(metadata: &Metadata, all: bool) -> Result<Vec<Pkg>> {
    let mut pkgs = vec![];

    for id in &metadata.workspace_members {
        if let Some(pkg) = metadata.packages.iter().find(|x| x.id == *id) {
            let private =
                pkg.publish.is_some() && pkg.publish.as_ref().expect(INTERNAL_ERR).is_empty();

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

            let loc = loc.expect(INTERNAL_ERR);
            let loc = if loc.is_file() {
                loc.parent().expect(INTERNAL_ERR)
            } else {
                loc
            };

            pkgs.push(Pkg {
                id: pkg.id.clone(),
                name: pkg.name.clone(),
                version: pkg.version.clone(),
                location: metadata.workspace_root.join(loc),
                path: loc.into(),
                private,
                config: Config::new(&pkg.metadata)?,
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
