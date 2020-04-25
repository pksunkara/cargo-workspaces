use crate::utils::{get_pkgs, git, Error, Pkg, Writer, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::Clap;
use regex::Regex;

#[derive(Debug, Clap)]
pub struct ChangeOpt {
    // TODO: ignore_changes, force_publish (glob), include_dirty
    /// Include tags from merged branches when detecting changed packages
    #[clap(long)]
    pub include_merged_tags: bool,
}

#[derive(Debug, Default)]
pub struct ChangeData {
    pub since: Option<String>,
    pub version: Option<String>,
    pub sha: String,
    pub count: String,
    pub dirty: bool,
}

impl ChangeData {
    pub fn new(metadata: &Metadata, change: &ChangeOpt) -> Result<Self, Error> {
        let mut args = vec!["describe", "--always", "--long", "--dirty", "--tags"];

        if !change.include_merged_tags {
            args.push("--first-parent");
        }

        let description = git(&metadata.workspace_root, &args)?;

        let sha_regex = Regex::new("^([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);
        let tag_regex =
            Regex::new("^((?:.*@)?v?(.*))-(\\d+)-g([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);

        let mut ret = Self::default();

        if sha_regex.is_match(&description) {
            let caps = sha_regex.captures(&description).expect(INTERNAL_ERR);

            ret.sha = caps.get(1).expect(INTERNAL_ERR).as_str().to_string();
            ret.dirty = caps.get(2).is_some();
            ret.count = git(&metadata.workspace_root, &["rev-list", "--count", &ret.sha])?;
        } else if tag_regex.is_match(&description) {
            let caps = tag_regex.captures(&description).expect(INTERNAL_ERR);

            ret.since = Some(caps.get(1).expect(INTERNAL_ERR).as_str().to_string());
            ret.version = Some(caps.get(2).expect(INTERNAL_ERR).as_str().to_string());

            ret.sha = caps.get(4).expect(INTERNAL_ERR).as_str().to_string();
            ret.dirty = caps.get(5).is_some();
            ret.count = caps.get(3).expect(INTERNAL_ERR).as_str().to_string();
        } else {
            return Err(Error::DescribeRefFail(description));
        }

        Ok(ret)
    }
}

pub fn get_changed_pkgs(
    metadata: &Metadata,
    change: &ChangeOpt,
    since: &Option<String>,
    private: bool,
) -> Result<Vec<Pkg>, Error> {
    let pkgs = get_pkgs(&metadata, private)?;

    let pkgs = if let Some(since) = since {
        let mut stderr = Writer::new(true);

        stderr.none("Looking for changed packages since ")?;
        stderr.cyan(&since)?;
        stderr.none("\n")?;

        let changed_files = git(
            &metadata.workspace_root,
            &["diff", "--name-only", "--relative", since],
        )?;

        let changed_files = changed_files.split("\n").collect::<Vec<_>>();

        pkgs.into_iter()
            .filter(|p: &Pkg| {
                //
                changed_files.iter().any(|f| f.starts_with(&p.path))
            })
            .collect()
    } else {
        pkgs
    };

    Ok(pkgs)
}
