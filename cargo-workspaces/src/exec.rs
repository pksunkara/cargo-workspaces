use crate::utils::{dag, filter_private, info, Error, Result, INTERNAL_ERR};

use cargo_metadata::Metadata;
use clap::Parser;
use globset::{Error as GlobsetError, Glob};

use std::{process::Command, result::Result as StdResult};

/// Execute an arbitrary command in each crate
#[derive(Debug, Parser)]
#[clap(trailing_var_arg(true))]
pub struct Exec {
    /// Continue executing command despite non-zero exit in a given crate
    #[clap(long)]
    no_bail: bool,

    /// Ignore the crates matched by glob
    #[clap(long, value_name = "PATTERN")]
    ignore: Option<String>,

    /// Ignore private crates
    #[clap(long)]
    ignore_private: bool,

    #[clap(required = true)]
    args: Vec<String>,
}

impl Exec {
    pub fn run(&self, metadata: Metadata) -> Result {
        let pkgs = metadata
            .packages
            .iter()
            .map(|x| (x.clone(), x.version.to_string()))
            .collect::<Vec<_>>();

        let (names, mut visited) = dag(&pkgs);

        if self.ignore_private {
            visited = filter_private(visited, &pkgs);
        }

        let ignore = self
            .ignore
            .clone()
            .map(|x| Glob::new(&x))
            .map_or::<StdResult<_, GlobsetError>, _>(Ok(None), |x| Ok(x.ok()))?;

        let mut errored = false;
        for p in &visited {
            let (pkg, _) = names.get(p).expect(INTERNAL_ERR);

            if let Some(pattern) = &ignore
                && pattern.compile_matcher().is_match(&pkg.name) {
                    continue;
                }

            let dir = pkg
                .manifest_path
                .parent()
                .ok_or_else(|| Error::ManifestHasNoParent(pkg.name.clone()))?;

            let status = Command::new(self.args.first().expect(INTERNAL_ERR))
                .args(&self.args[1..])
                .current_dir(dir)
                .status()?;

            if !status.success() {
                match self.no_bail {
                    true => errored = true,
                    false => return Err(Error::Bail),
                }
            }
        }

        match errored {
            true => {
                info!("failed", "error(s) occurred");
                Err(Error::Bail)
            }
            false => {
                info!("success", "ok");
                Ok(())
            }
        }
    }
}
