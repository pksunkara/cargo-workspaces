use crate::utils::{debug, Error, INTERNAL_ERR};
use lazy_static::lazy_static;
use regex::Regex;
use semver::{Version, VersionReq};
use std::collections::BTreeMap as Map;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

pub fn cargo<'a>(root: &PathBuf, args: &[&'a str]) -> Result<(String, String), Error> {
    debug!("cargo", args.clone().join(" "))?;

    let mut output_stderr = vec![];
    let mut child = Command::new("cargo")
        .current_dir(root)
        .args(args)
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
) -> Result<String, Error> {
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
                            if !VersionReq::parse(&caps[3])?.matches(new_version) {
                                new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[4]));
                                continue;
                            }
                        }
                    }

                    if let Some(caps) = DEP_OBJ_VERSION.captures(line) {
                        if let Some(new_version) = versions.get(&caps[2]) {
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

    Ok(new_lines.join("\n"))
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
            change_versions(m, "this", &v).unwrap(),
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
            change_versions(m, "this", &v).unwrap(),
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
            change_versions(m, "this", &v).unwrap(),
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
            change_versions(m, "this", &v).unwrap(),
            r#"
            [package]
            'version'='0.3.0'# hello"#
        );
    }

    #[test]
    fn test_dependencies() {
        let m = r#"
            [dependencies]
            this = ">=0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v).unwrap(),
            r#"
            [dependencies]
            this = "0.3.0" # hello"#
        );
    }

    #[test]
    fn test_dependencies_object() {
        let m = r#"
            [dependencies]
            this = { path = "../", version = ">=0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v).unwrap(),
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
            version = ">=0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v).unwrap(),
            r#"
            [dependencies.this]
            path = "../"
            version = "0.3.0" # hello"#
        );
    }
}
