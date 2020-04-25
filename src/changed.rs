use crate::utils::{git, Error, ListOpt, Writer, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::Clap;
use regex::Regex;

/// List local packages that have changed since the last tagged release
#[derive(Debug, Clap)]
pub struct Changed {
    #[clap(flatten)]
    list: ListOpt,

    // TODO: ignore_changes
    /// Include tags from merged branches when detecting changed packages
    #[clap(long)]
    include_merged_tags: bool,
}

impl Changed {
    pub fn run(self, metadata: Metadata, mut stdout: Writer) -> Result<(), Error> {
        // let mut pkgs = vec![];

        let mut args = vec!["describe", "--always", "--long", "--dirty", "--tags"];

        if !self.include_merged_tags {
            args.push("--first-parent");
        }

        let description = git(&metadata.workspace_root, &args)?;

        let sha_regex = Regex::new("^([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);

        if sha_regex.is_match(&description) {
            let caps = sha_regex.captures(&description).expect(INTERNAL_ERR);

            let sha = caps.get(1).expect(INTERNAL_ERR).as_str();
            let dirty = caps.get(2).is_some();

            let count = git(&metadata.workspace_root, &["rev-list", "--count", sha])?;

            println!("{:#?}", (count, sha, dirty))
        }

        let tag_regex =
            Regex::new("^((?:.*@)?v?(.*))-(\\d+)-g([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);

        if tag_regex.is_match(&description) {
            let caps = tag_regex.captures(&description).expect(INTERNAL_ERR);

            let tag = caps.get(1).expect(INTERNAL_ERR).as_str();
            let version = caps.get(2).expect(INTERNAL_ERR).as_str();
            let count = caps.get(3).expect(INTERNAL_ERR).as_str();
            let sha = caps.get(4).expect(INTERNAL_ERR).as_str();
            let dirty = caps.get(5).is_some();

            println!("{:#?}", (tag, version, count, sha, dirty));
        }

        Ok(())
    }
}
