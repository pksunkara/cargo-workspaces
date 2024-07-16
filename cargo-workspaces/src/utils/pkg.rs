use crate::utils::{read_config, Error, PackageConfig, Result, INTERNAL_ERR};

use cargo_metadata::{Metadata, Package, PackageId};
use oclif::CliError;
use semver::Version;
use serde::Serialize;

use std::path::PathBuf;

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
    pub config: PackageConfig,
}

pub fn is_private(pkg: &Package) -> bool {
    pkg.publish.is_some() && pkg.publish.as_ref().expect(INTERNAL_ERR).is_empty()
}

pub fn get_pkgs(metadata: &Metadata, all: bool) -> Result<Vec<Pkg>> {
    let mut pkgs = vec![];

    for id in &metadata.workspace_members {
        if let Some(pkg) = metadata.packages.iter().find(|x| x.id == *id) {
            let private = is_private(pkg);

            if !all && private {
                continue;
            }

            let loc = pkg.manifest_path.strip_prefix(&metadata.workspace_root);

            if loc.is_err() {
                return Err(Error::PackageNotInWorkspace {
                    id: pkg.id.repr.clone(),
                    ws: metadata.workspace_root.to_string(),
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
                location: metadata.workspace_root.join(loc).into(),
                path: loc.into(),
                private,
                config: read_config(&pkg.metadata)?,
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
