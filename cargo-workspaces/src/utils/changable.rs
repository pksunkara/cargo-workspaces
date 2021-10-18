use crate::utils::{get_pkgs, git, info, Error, Pkg, INTERNAL_ERR};
use cargo_metadata::Metadata;
use clap::Parser;
use glob::{Pattern, PatternError};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Parser)]
pub struct ChangeOpt {
    // TODO: include_dirty
    /// Include tags from merged branches
    #[clap(long)]
    pub include_merged_tags: bool,

    /// Always include targeted crates matched by glob even when there are no changes
    #[clap(long, value_name = "pattern")]
    pub force: Option<String>,

    /// Ignore changes in files matched by glob
    #[clap(long, value_name = "pattern")]
    pub ignore_changes: Option<String>,
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

        let (description, _) = git(&metadata.workspace_root, &args)?;

        let sha_regex = Regex::new("^([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);
        let tag_regex =
            Regex::new("^((?:.*@)?v?(.*))-(\\d+)-g([0-9a-f]{7,40})(-dirty)?$").expect(INTERNAL_ERR);

        let mut ret = Self::default();

        if sha_regex.is_match(&description) {
            let caps = sha_regex.captures(&description).expect(INTERNAL_ERR);

            ret.sha = caps.get(1).expect(INTERNAL_ERR).as_str().to_string();
            ret.dirty = caps.get(2).is_some();

            let (count, _) = git(&metadata.workspace_root, &["rev-list", "--count", &ret.sha])?;

            ret.count = count;
        } else if tag_regex.is_match(&description) {
            let caps = tag_regex.captures(&description).expect(INTERNAL_ERR);

            ret.since = Some(caps.get(1).expect(INTERNAL_ERR).as_str().to_string());
            ret.version = Some(caps.get(2).expect(INTERNAL_ERR).as_str().to_string());

            ret.sha = caps.get(4).expect(INTERNAL_ERR).as_str().to_string();
            ret.dirty = caps.get(5).is_some();
            ret.count = caps.get(3).expect(INTERNAL_ERR).as_str().to_string();
        }

        Ok(ret)
    }
}

impl ChangeOpt {
    pub fn get_changed_pkgs(
        &self,
        metadata: &Metadata,
        since: &Option<String>,
        private: bool,
    ) -> Result<(Vec<Pkg>, Vec<Pkg>), Error> {
        let pkgs = get_pkgs(&metadata, private)?;

        let pkgs = if let Some(since) = since {
            info!("looking for changes since", since);

            let (changed_files, _) = git(
                &metadata.workspace_root,
                &["diff", "--name-only", "--relative", since],
            )?;

            let changed_files = changed_files.split("\n").map(Path::new).collect::<Vec<_>>();
            let force = self
                .force
                .clone()
                .map(|x| Pattern::new(&x))
                .map_or::<Result<_, PatternError>, _>(Ok(None), |x| Ok(x.ok()))?;
            let ignore_changes = self
                .ignore_changes
                .clone()
                .map(|x| Pattern::new(&x))
                .map_or::<Result<_, PatternError>, _>(Ok(None), |x| Ok(x.ok()))?;

            pkgs.into_iter().partition(|p: &Pkg| {
                if let Some(pattern) = &force {
                    if pattern.matches(&p.name) {
                        return true;
                    }
                }

                changed_files.iter().any(|f| {
                    if let Some(pattern) = &ignore_changes {
                        if pattern.matches(f.to_str().expect(INTERNAL_ERR)) {
                            return false;
                        }
                    }

                    f.starts_with(&p.path)
                })
            })
        } else {
            (pkgs, vec![])
        };

        Ok(pkgs)
    }
}
