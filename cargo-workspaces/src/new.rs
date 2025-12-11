use crate::{
    init::Resolver,
    utils::{git, warn, Error, Result},
};

use camino::Utf8PathBuf;
use clap::Parser;
use std::{env, fs, path::PathBuf};

/// Create a new workspace with the specified path
#[derive(Debug, Parser)]
pub struct New {
    /// Path for the new workspace root
    #[clap(parse(from_os_str))]
    pub path: PathBuf,

    /// Workspace feature resolver version
    /// [default: 3]
    #[clap(short, long, arg_enum)]
    pub resolver: Option<Resolver>,
}

impl New {
    pub fn run(&self) -> Result {
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(get_current_dir_err) => return Err(Error::Io(get_current_dir_err)),
        };

        // Create absolute path by joining current directory with the provided path
        let new_dir = current_dir.join(&self.path);

        // Create directory if it doesn't exist
        if let Err(create_dir_err) = fs::create_dir_all(&new_dir) {
            return Err(Error::Io(create_dir_err));
        }

        // Run git init command
        let new_dir_utf8 = Utf8PathBuf::from_path_buf(new_dir.clone())
            .unwrap_or_else(|_| panic!("{} is not valid UTF-8.", &new_dir.display()));

        let (exit_status, ..) = git(&new_dir_utf8, &["init"])?;
        if !exit_status.success() {
            warn!("git repository init failed ", &new_dir.display());
        }

        // Create .gitignore file with content "/target"
        let gitignore_path = new_dir.join(".gitignore");

        if fs::write(&gitignore_path, "/target").is_err() {
            warn!(
                "create or write .gitignore failed ",
                &gitignore_path.display()
            );
        }

        let resolver = self.resolver.or(Some(Resolver::V3));

        // Call init module functionality to initialize the workspace
        let init = crate::init::Init {
            path: new_dir.clone(),
            resolver,
        };
        init.run()
    }
}
