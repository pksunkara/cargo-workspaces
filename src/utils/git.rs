use crate::utils::{Error, INTERNAL_ERR};
use clap::Clap;
use glob::Pattern;
use semver::Version;
use std::collections::BTreeMap as Map;
use std::{path::PathBuf, process::Command};

pub fn git<'a>(root: &PathBuf, args: &[&'a str]) -> Result<(String, String), Error> {
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

#[derive(Debug, Clap)]
pub struct GitOpt {
    /// Do not commit changes
    #[clap(long, conflicts_with_all = &["allow-branch", "amend", "message", "no-git-tag", "tag-prefix", "no-independent-tags", "no-git-push", "git-remote"])]
    pub no_git_commit: bool,

    /// Specify which branches to allow from
    #[clap(long, default_value = "master", value_name = "pattern")]
    pub allow_branch: String,

    #[clap(long)]
    pub amend: bool,

    #[clap(short, long)]
    pub message: Option<String>,

    #[clap(long, conflicts_with_all = &["tag-prefix", "no-independent-tags"])]
    pub no_git_tag: bool,

    #[clap(long, default_value = "v")]
    pub tag_prefix: String,

    #[clap(long)]
    pub no_independent_tags: bool,

    /// Do not push commit to git remote
    #[clap(long, conflicts_with_all = &["git-remote"])]
    pub no_git_push: bool,

    /// Push git changes to the specified remote
    #[clap(long, default_value = "origin", value_name = "remote")]
    pub git_remote: String,
}

impl GitOpt {
    pub fn validate(&self, root: &PathBuf) -> Result<Option<String>, Error> {
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

            let pattern = Pattern::new(&self.allow_branch)?;

            if !pattern.matches(&branch) {
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
        root: &PathBuf,
        new_version: &Option<Version>,
        new_versions: &Map<String, Version>,
        branch: Option<String>,
    ) -> Result<(), Error> {
        if !self.no_git_commit {
            let added = git(root, &["add", "."])?;

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

                let mut msg = self.append_message(msg, new_versions);

                if let Some(version) = new_version {
                    msg = msg.replace("%v", &format!("{}", version));
                }

                args.push(msg);
            }

            let committed = git(root, &args.iter().map(|x| x.as_str()).collect::<Vec<_>>())?;

            if !committed.0.contains("files changed") || !committed.1.is_empty() {
                return Err(Error::NotCommitted(committed.0, committed.1));
            }

            if !self.no_git_tag {
                if let Some(version) = new_version {
                    self.tag(root, &self.tag_prefix, version)?;
                }

                if !self.no_independent_tags {
                    for (p, v) in new_versions {
                        self.tag(root, &format!("{}@", p), v)?;
                    }
                }
            }

            if !self.no_git_push {
                let pushed = git(
                    root,
                    &[
                        "push",
                        &self.git_remote,
                        &branch.as_ref().expect(INTERNAL_ERR),
                    ],
                )?;

                if !pushed.0.is_empty() || !pushed.1.starts_with("To") {
                    return Err(Error::NotPushed(pushed.0, pushed.1));
                }

                if !self.no_git_tag {
                    let pushed = git(
                        root,
                        &[
                            "push",
                            "--tags",
                            &self.git_remote,
                            &branch.as_ref().expect(INTERNAL_ERR),
                        ],
                    )?;

                    if !pushed.0.is_empty() || !pushed.1.starts_with("To") {
                        return Err(Error::NotPushed(pushed.0, pushed.1));
                    }
                }
            }
        }

        Ok(())
    }

    fn tag(&self, root: &PathBuf, prefix: &str, version: &Version) -> Result<(), Error> {
        let tag = format!("{}{}", prefix, version);
        let tagged = git(root, &["tag", &tag])?;

        if !tagged.0.is_empty() || !tagged.1.is_empty() {
            return Err(Error::NotTagged(tag, tagged.0, tagged.1));
        }

        Ok(())
    }

    fn append_message(&self, msg: &str, new_versions: &Map<String, Version>) -> String {
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
