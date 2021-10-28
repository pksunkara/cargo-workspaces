use crate::utils::{debug, info, validate_value_containing_name, Error, INTERNAL_ERR};

use camino::Utf8PathBuf;
use clap::{ArgSettings, Parser};
use glob::Pattern;
use semver::Version;

use std::{collections::BTreeMap as Map, process::Command};

pub fn git<'a>(root: &Utf8PathBuf, args: &[&'a str]) -> Result<(String, String), Error> {
    debug!("git", args.clone().join(" "));

    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|err| Error::Git {
            err,
            args: args.iter().map(|x| x.to_string()).collect(),
        })?;

    Ok((
        String::from_utf8(output.stdout)?.trim().to_owned(),
        String::from_utf8(output.stderr)?.trim().to_owned(),
    ))
}

#[derive(Debug, Parser)]
pub struct GitOpt {
    /// Do not commit version changes
    #[clap(long, conflicts_with_all = &[
        "allow-branch", "amend", "message", "no-git-tag",
        "tag-prefix", "individual-tag-prefix", "no-individual-tags",
        "no-git-push", "git-remote", "no-global-tag"
    ])]
    pub no_git_commit: bool,

    /// Specify which branches to allow from
    #[clap(
        long,
        default_value = "master",
        value_name = "pattern",
        setting(ArgSettings::ForbidEmptyValues)
    )]
    pub allow_branch: String,

    /// Amend the existing commit, instead of generating a new one
    #[clap(long)]
    pub amend: bool,

    /// Use a custom commit message when creating the version commit
    #[clap(
        short,
        long,
        conflicts_with_all = &["amend"],
        setting(ArgSettings::ForbidEmptyValues)
    )]
    pub message: Option<String>,

    /// Do not tag generated commit
    #[clap(long, conflicts_with_all = &["tag-prefix", "individual-tag-prefix", "no-individual-tags"])]
    pub no_git_tag: bool,

    /// Do not tag individual versions for crates
    #[clap(long, conflicts_with_all = &["individual-tag-prefix"])]
    pub no_individual_tags: bool,

    /// Do not create a global tag for a workspace
    #[clap(long)]
    pub no_global_tag: bool,

    /// Customize tag prefix (can be empty)
    #[clap(long, default_value = "v", value_name = "prefix")]
    pub tag_prefix: String,

    /// Customize prefix for individual tags (should contain `%n`)
    #[clap(
        long,
        default_value = "%n@",
        value_name = "prefix",
        validator = validate_value_containing_name,
        setting(ArgSettings::ForbidEmptyValues)
    )]
    pub individual_tag_prefix: String,

    /// Do not push generated commit and tags to git remote
    #[clap(long, conflicts_with_all = &["git-remote"])]
    pub no_git_push: bool,

    /// Push git changes to the specified remote
    #[clap(
        long,
        default_value = "origin",
        value_name = "remote",
        setting(ArgSettings::ForbidEmptyValues)
    )]
    pub git_remote: String,
}

impl GitOpt {
    pub fn validate(&self, root: &Utf8PathBuf) -> Result<Option<String>, Error> {
        let mut ret = None;

        if !self.no_git_commit {
            let (out, err) = git(root, &["rev-list", "--count", "--all", "--max-count=1"])?;

            if err.contains("not a git repository") {
                return Err(Error::NotGit);
            }

            if out == "0" {
                return Err(Error::NoCommits);
            }

            let (branch, _) = git(root, &["rev-parse", "--abbrev-ref", "HEAD"])?;

            if branch == "HEAD" {
                return Err(Error::NotBranch);
            }

            ret = Some(branch.clone());

            // Treat `main` as `master`
            let test_branch = if branch == "main" && self.allow_branch == "master" {
                "master".into()
            } else {
                branch.clone()
            };

            let pattern = Pattern::new(&self.allow_branch)?;

            if !pattern.matches(&test_branch) {
                return Err(Error::BranchNotAllowed {
                    branch,
                    pattern: pattern.as_str().to_string(),
                });
            }

            if !self.no_git_push {
                let remote_branch = format!("{}/{}", self.git_remote, branch);

                let (out, _) = git(
                    root,
                    &[
                        "show-ref",
                        "--verify",
                        &format!("refs/remotes/{}", remote_branch),
                    ],
                )?;

                if out.is_empty() {
                    return Err(Error::NoRemote {
                        remote: self.git_remote.clone(),
                        branch,
                    });
                }

                git(root, &["remote", "update"])?;

                let (out, _) = git(
                    root,
                    &[
                        "rev-list",
                        "--left-only",
                        "--count",
                        &format!("{}...{}", remote_branch, branch),
                    ],
                )?;

                if out != "0" {
                    return Err(Error::BehindRemote {
                        branch,
                        upstream: remote_branch,
                    });
                }
            }
        }

        Ok(ret)
    }

    pub fn commit(
        &self,
        root: &Utf8PathBuf,
        new_version: &Option<Version>,
        new_versions: &Map<String, Version>,
        branch: Option<String>,
    ) -> Result<(), Error> {
        if !self.no_git_commit {
            info!("version", "committing changes");

            let branch = branch.expect(INTERNAL_ERR);
            let added = git(root, &["add", "-u"])?;

            if !added.0.is_empty() || !added.1.is_empty() {
                return Err(Error::NotAdded(added.0, added.1));
            }

            let mut args = vec!["commit".to_string()];

            if self.amend {
                args.push("--amend".to_string());
                args.push("--no-edit".to_string());
            } else {
                args.push("-m".to_string());

                let mut msg = "Release %v";

                if let Some(supplied) = &self.message {
                    msg = supplied;
                }

                let mut msg = self.commit_msg(msg, new_versions);

                msg = msg.replace(
                    "%v",
                    &new_version
                        .as_ref()
                        .map_or("independent packages".to_string(), |x| format!("{}", x)),
                );

                args.push(msg);
            }

            let committed = git(root, &args.iter().map(|x| x.as_str()).collect::<Vec<_>>())?;

            if !committed.0.contains(&branch) || !committed.1.is_empty() {
                return Err(Error::NotCommitted(committed.0, committed.1));
            }

            if !self.no_git_tag {
                info!("version", "tagging");

                if !self.no_global_tag {
                    if let Some(version) = new_version {
                        let tag = format!("{}{}", &self.tag_prefix, version);
                        self.tag(root, &tag, &tag)?;
                    }
                }

                if !self.no_individual_tags {
                    for (p, v) in new_versions {
                        let tag = format!("{}{}", self.individual_tag_prefix.replace("%n", p), v);
                        self.tag(root, &tag, &tag)?;
                    }
                }
            }

            if !self.no_git_push {
                info!("git", "pushing");

                let pushed = git(root, &["push", "--follow-tags", &self.git_remote, &branch])?;

                if !pushed.0.is_empty() || !pushed.1.starts_with("To") {
                    return Err(Error::NotPushed(pushed.0, pushed.1));
                }
            }
        }

        Ok(())
    }

    fn tag(&self, root: &Utf8PathBuf, tag: &str, msg: &str) -> Result<(), Error> {
        let tagged = git(root, &["tag", &tag, "-m", &msg])?;

        if !tagged.0.is_empty() || !tagged.1.is_empty() {
            return Err(Error::NotTagged(tag.to_string(), tagged.0, tagged.1));
        }

        Ok(())
    }

    fn commit_msg(&self, msg: &str, new_versions: &Map<String, Version>) -> String {
        format!(
            "{}\n\n{}\n\nGenerated by cargo-workspaces",
            msg,
            new_versions
                .iter()
                .map(|x| format!("{}@{}", x.0, x.1))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
