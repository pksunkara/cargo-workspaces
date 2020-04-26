use crate::utils::Error;
use clap::Clap;
use glob::Pattern;
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
    /// Specify which branches to allow from
    #[clap(long, default_value = "master", value_name = "pattern")]
    pub allow_branch: String,

    /// Push git changes to the specified remote
    #[clap(long, default_value = "origin", value_name = "remote")]
    pub git_remote: String,

    /// Do not commit changes
    #[clap(long, conflicts_with_all = &["no-git-push", "git-remote", "allow-branch"])]
    pub no_git_commit: bool,

    /// Do not push commit to git remote
    #[clap(long, conflicts_with_all = &["git-remote"])]
    pub no_git_push: bool,
}

impl GitOpt {
    pub fn validate(&self, root: &PathBuf) -> Result<(), Error> {
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

        Ok(())
    }
}
