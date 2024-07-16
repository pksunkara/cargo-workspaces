use crate::utils::{Pkg, Result, INTERNAL_ERR};

use clap::Parser;
use oclif::{console::style, term::TERM_OUT};
use serde_json::to_string_pretty;

use std::{cmp::max, path::Path};

#[derive(Debug, Parser)]
pub struct ListOpt {
    /// Show extended information
    #[clap(short, long)]
    pub long: bool,

    /// Show private crates that are normally hidden
    #[clap(short, long)]
    pub all: bool,

    /// Show information as a JSON array
    #[clap(long, conflicts_with = "long")]
    pub json: bool,
}

pub fn list(pkgs: &[Pkg], list: ListOpt) -> Result {
    if list.json {
        return Ok(TERM_OUT.write_line(&to_string_pretty(pkgs)?)?);
    }

    if pkgs.is_empty() {
        return Ok(());
    }

    let first = pkgs.iter().map(|x| x.name.len()).max().expect(INTERNAL_ERR);
    let second = pkgs
        .iter()
        .map(|x| x.version.to_string().len() + 1)
        .max()
        .expect(INTERNAL_ERR);
    let third = pkgs
        .iter()
        .map(|x| max(1, x.path.as_os_str().len()))
        .max()
        .expect(INTERNAL_ERR);

    for pkg in pkgs {
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
