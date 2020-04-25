use crate::utils::{get_pkgs, git, Error, ListOpt, Listable, Result, Writer, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::Clap;
use regex::Regex;

/// List local packages that have changed since the last tagged release
#[derive(Debug, Clap)]
pub struct Changed {
    #[clap(flatten)]
    list: ListOpt,

    // TODO: ignore_changes, force_publish (glob), include_dirty
    /// Include tags from merged branches when detecting changed packages
    #[clap(long)]
    include_merged_tags: bool,

    /// Use this git reference instead of the last tag
    #[clap(long)]
    since: Option<String>,
}

impl Changed {
    pub fn run(self, metadata: Metadata, mut stdout: Writer, mut stderr: Writer) -> Result {
        let mut args = vec!["describe", "--always", "--long", "--dirty", "--tags"];

        if !self.include_merged_tags {
            args.push("--first-parent");
        }

        let description = git(&metadata.workspace_root, &args)?;

        let sha_regex = Regex::new("^([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);
        let tag_regex =
            Regex::new("^((?:.*@)?v?(.*))-(\\d+)-g([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);

        let count;
        let sha;
        let dirty;
        let mut since = self.since.clone();

        if self.since.is_none() {
            if sha_regex.is_match(&description) {
                let caps = sha_regex.captures(&description).expect(INTERNAL_ERR);

                since = None;
                sha = caps.get(1).expect(INTERNAL_ERR).as_str().to_string();
                dirty = caps.get(2).is_some();

                count = git(&metadata.workspace_root, &["rev-list", "--count", &sha])?;
            } else if tag_regex.is_match(&description) {
                let caps = tag_regex.captures(&description).expect(INTERNAL_ERR);

                since = Some(caps.get(1).expect(INTERNAL_ERR).as_str().to_string());
                let version = caps.get(2).expect(INTERNAL_ERR).as_str();

                count = caps.get(3).expect(INTERNAL_ERR).as_str().to_string();
                sha = caps.get(4).expect(INTERNAL_ERR).as_str().to_string();
                dirty = caps.get(5).is_some();
            } else {
                return Err(Error::DescribeRefFail(description));
            }

            if count == "0" {
                return Ok(
                    stderr.none("Current HEAD is already released, skipping change detection")?
                );
            }
        }

        if since.is_none() {
            return Ok(());
        }

        let since = since.expect(INTERNAL_ERR);

        stderr.none("Looking for changed packages since ")?;
        stderr.cyan(&since)?;
        stderr.none("\n")?;

        let changed_files = git(
            &metadata.workspace_root,
            &["diff", "--name-only", "--relative", &since],
        )?;
        let changed_files = changed_files.split("\n").collect::<Vec<_>>();

        let pkgs = get_pkgs(&metadata, self.list.all)?
            .into_iter()
            .filter(|p| {
                //
                changed_files.iter().any(|f| f.starts_with(&p.path))
            })
            .collect::<Vec<_>>();

        pkgs.list(stdout, self.list)
    }
}
