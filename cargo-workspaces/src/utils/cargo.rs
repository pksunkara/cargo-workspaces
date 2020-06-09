use crate::utils::{debug, git, info, Error, Result, INTERNAL_ERR, TERM_ERR};
use crates_index::Index;
use lazy_static::lazy_static;
use regex::Regex;
use semver::{Version, VersionReq};
use std::{
    collections::BTreeMap as Map,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

const CRLF: &'static str = "\r\n";
const LF: &'static str = "\n";

lazy_static! {
    static ref VERSION: Regex =
        Regex::new(r#"^(\s*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_ENTRY: Regex =
        Regex::new(r#"^\[dependencies.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref BUILD_DEP_ENTRY: Regex =
        Regex::new(r#"^\[build-dependencies.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref DEP_DIRECT_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*['"])([^'"]+)(['"].*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*\{.*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*}.*)$"#)
            .expect(INTERNAL_ERR);
}

pub fn cargo<'a>(root: &PathBuf, args: &[&'a str]) -> Result<(String, String)> {
    debug!("cargo", args.clone().join(" "))?;

    let mut args = args.to_vec();

    if TERM_ERR.features().colors_supported() {
        args.push("--color");
        args.push("always");
    }

    let mut output_stderr = vec![];
    let mut child = Command::new("cargo")
        .current_dir(root)
        .args(&args)
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Cargo {
            err,
            args: args.iter().map(|x| x.to_string()).collect(),
        })?;

    {
        let stderr = child.stderr.as_mut().expect(INTERNAL_ERR);

        for line in BufReader::new(stderr).lines() {
            let line = line?;

            eprintln!("{}", line);
            output_stderr.push(line);
        }
    }

    let output = child.wait_with_output().map_err(|err| Error::Cargo {
        err,
        args: args.iter().map(|x| x.to_string()).collect(),
    })?;

    Ok((
        String::from_utf8(output.stdout)?.trim().to_owned(),
        output_stderr.join("\n").trim().to_owned(),
    ))
}

#[derive(Debug)]
enum Context {
    Beginning,
    Package,
    Dependencies,
    DependencyEntry(String),
    BuildDependencies,
    BuildDependencyEntry(String),
    DontCare,
}

pub fn change_versions(
    manifest: String,
    pkg_name: &str,
    versions: &Map<String, Version>,
    exact: bool,
) -> Result<String> {
    let mut context = Context::Beginning;
    let mut new_lines = vec![];

    for line in manifest.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("[package]") {
            context = Context::Package;
        } else if trimmed.starts_with("[dependencies]") {
            context = Context::Dependencies;
        } else if trimmed.starts_with("[build-dependencies]") {
            context = Context::BuildDependencies;
        } else if let Some(caps) = DEP_ENTRY.captures(trimmed) {
            context = Context::DependencyEntry(caps[1].to_string());
        } else if let Some(caps) = BUILD_DEP_ENTRY.captures(trimmed) {
            context = Context::BuildDependencyEntry(caps[1].to_string());
        } else if trimmed.starts_with("[") {
            context = Context::DontCare;
        } else {
            // TODO: Support `package.version` like stuff (with quotes) at beginning

            match context {
                Context::Package => {
                    if let Some(new_version) = versions.get(pkg_name) {
                        if let Some(caps) = VERSION.captures(line) {
                            new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[3]));
                            continue;
                        }
                    }
                }
                Context::Dependencies | Context::BuildDependencies => {
                    if let Some(caps) = DEP_DIRECT_VERSION.captures(line) {
                        if let Some(new_version) = versions.get(&caps[2]) {
                            if exact {
                                new_lines.push(format!("{}={}{}", &caps[1], new_version, &caps[4]));
                                continue;
                            }

                            if !VersionReq::parse(&caps[3])?.matches(new_version) {
                                new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[4]));
                                continue;
                            }
                        }
                    }

                    if let Some(caps) = DEP_OBJ_VERSION.captures(line) {
                        if let Some(new_version) = versions.get(&caps[2]) {
                            if exact {
                                new_lines.push(format!("{}={}{}", &caps[1], new_version, &caps[4]));
                                continue;
                            }

                            if !VersionReq::parse(&caps[3])?.matches(new_version) {
                                new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[4]));
                                continue;
                            }
                        }
                    }
                }
                Context::DependencyEntry(ref dep) | Context::BuildDependencyEntry(ref dep) => {
                    if let Some(new_version) = versions.get(dep) {
                        if let Some(caps) = VERSION.captures(line) {
                            if exact {
                                new_lines.push(format!("{}={}{}", &caps[1], new_version, &caps[3]));
                                continue;
                            }

                            if !VersionReq::parse(&caps[2])?.matches(new_version) {
                                new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[3]));
                                continue;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        new_lines.push(line.to_string());
    }

    Ok(new_lines.join(if manifest.contains(CRLF) { CRLF } else { LF }))
}

pub fn check_index(name: &str, version: &str) -> Result<()> {
    let index = Index::new_cargo_default();
    let now = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let timeout = Duration::from_secs(300);
    let mut logged = false;

    loop {
        if let Err(e) = index.update() {
            Error::IndexUpdate(e).print_err()?;
        }

        let crate_data = index.crate_(name);
        let published = crate_data
            .iter()
            .flat_map(|c| c.versions().iter())
            .find(|v| v.version() == version)
            .is_some();

        if published {
            match git(
                &index.path().to_owned(),
                &[
                    "fetch",
                    "https://github.com/rust-lang/crates.io-index",
                    "refs/heads/master:refs/remotes/origin/master",
                ],
            ) {
                Ok((out, err)) => {
                    println!("updating index\nout\n{}\nerr\n{}", out, err);
                    break;
                }
                Err(e) => {
                    e.print_err()?;
                }
            }
        } else if timeout < now.elapsed() {
            return Err(Error::PublishTimeout);
        }

        if !logged {
            info!("waiting", "...")?;
            logged = true;
        }

        sleep(sleep_time);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version() {
        let m = r#"
            [package]
            version = "0.1.0""#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [package]
            version = "0.3.0""#
        );
    }

    #[test]
    fn test_version_comments() {
        let m = r#"
            [package]
            version="0.1.0" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [package]
            version="0.3.0" # hello"#
        );
    }

    #[test]
    fn test_version_quotes() {
        let m = r#"
            [package]
            "version"	=	"0.1.0""#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [package]
            "version"	=	"0.3.0""#
        );
    }

    #[test]
    fn test_version_single_quotes() {
        let m = r#"
            [package]
            'version'='0.1.0'# hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [package]
            'version'='0.3.0'# hello"#
        );
    }

    #[test]
    fn test_dependencies() {
        let m = r#"
            [dependencies]
            this = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies]
            this = "0.3.0" # hello"#
        );
    }

    #[test]
    fn test_dependencies_object() {
        let m = r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies]
            this = { path = "../", version = "0.3.0" } # hello"#
        );
    }

    #[test]
    fn test_dependency_table() {
        let m = r#"
            [dependencies.this]
            path = "../"
            version = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies.this]
            path = "../"
            version = "0.3.0" # hello"#
        );
    }

    #[test]
    fn test_exact() {
        let m = r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, true).unwrap(),
            r#"
            [dependencies]
            this = { path = "../", version = "=0.3.0" } # hello"#
        );
    }
}
