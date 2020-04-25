use std::{path::PathBuf, process::Command};

mod error;
mod listable;
mod pkg;
mod writer;

pub use error::Error;
pub use listable::{ListOpt, Listable};
pub use pkg::Pkg;
pub use writer::Writer;

pub const INTERNAL_ERR: &'static str = "Internal error message. Please create an issue on https://github.com/pksunkara/cargo-workspaces";

pub fn git<'a>(dir: &PathBuf, args: &[&'a str]) -> Result<String, Error> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|err| Error::Git {
            err,
            args: args.iter().map(|x| x.to_string()).collect(),
        })?;

    // println!("{:#?}", output);

    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
