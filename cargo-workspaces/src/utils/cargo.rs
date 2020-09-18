use crate::utils::{debug, info, Error, Result, INTERNAL_ERR, TERM_ERR};
use crates_index::BareIndex;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
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
    static ref NAME: Regex =
        Regex::new(r#"^(\s*['"]?name['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*)$"#).expect(INTERNAL_ERR);
    static ref VERSION: Regex =
        Regex::new(r#"^(\s*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*)$"#)
            .expect(INTERNAL_ERR);
    static ref PACKAGE: Regex =
        Regex::new(r#"^(\s*['"]?package['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*)$"#).expect(INTERNAL_ERR);
    static ref DEP_ENTRY: Regex =
        Regex::new(r#"^\[dependencies.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref BUILD_DEP_ENTRY: Regex =
        Regex::new(r#"^\[build-dependencies.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref DEV_DEP_ENTRY: Regex =
        Regex::new(r#"^\[dev-dependencies.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref DEP_DIRECT_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*['"])([^'"]+)(['"].*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*\{.*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_RENAME_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*\{.*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*['"]?package['"]?\s*=\s*['"]([0-9A-Za-z-_]+)['"].*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_RENAME_BEFORE_VERSION: Regex =
        Regex::new(r#"^(\s*['"]?[0-9A-Za-z-_]+['"]?\s*=\s*\{.*['"]?package['"]?\s*=\s*['"]([0-9A-Za-z-_]+)['"].*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_DIRECT_NAME: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*)(['"][^'"]+['"])(.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_NAME: Regex =
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*\{.*?)(\s*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_RENAME_NAME: Regex =
        Regex::new(r#"^(\s*['"]?[0-9A-Za-z-_]+['"]?\s*=\s*\{.*['"]?package['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*}.*)$"#)
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
    DontCare,
}

fn edit_version(
    caps: Captures,
    new_lines: &mut Vec<String>,
    versions: &Map<String, Version>,
    exact: bool,
    version_index: usize,
) -> Result<()> {
    if let Some(new_version) = versions.get(&caps[version_index]) {
        if exact {
            new_lines.push(format!("{}={}{}", &caps[1], new_version, &caps[4]));
        } else if !VersionReq::parse(&caps[3])?.matches(new_version) {
            new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[4]));
        }
    }

    Ok(())
}

fn rename_dep(
    caps: Captures,
    new_lines: &mut Vec<String>,
    renames: &Map<String, String>,
    name_index: usize,
) -> Result<()> {
    if let Some(new_name) = renames.get(&caps[name_index]) {
        new_lines.push(format!("{}{}{}", &caps[1], new_name, &caps[3]));
    }

    Ok(())
}

fn parse<P, D, DE, DP>(
    manifest: String,
    dev_deps: bool,
    package_f: P,
    dependencies_f: D,
    dependency_entries_f: DE,
    dependency_pkg_f: DP,
) -> Result<String>
where
    P: Fn(&str, &mut Vec<String>) -> Result,
    D: Fn(&str, &mut Vec<String>) -> Result,
    DE: Fn(&str, &str, &mut Vec<String>) -> Result<Option<Context>>,
    DP: Fn(&str, &mut Vec<String>) -> Result,
{
    let mut context = Context::Beginning;
    let mut new_lines = vec![];

    for line in manifest.lines() {
        let trimmed = line.trim();
        let count = new_lines.len();
        if trimmed.starts_with("[package]") {
            context = Context::Package;
        } else if trimmed.starts_with("[dependencies]") {
            context = Context::Dependencies;
        } else if trimmed.starts_with("[build-dependencies]") {
            context = Context::Dependencies;
        } else if dev_deps && trimmed.starts_with("[dev-dependencies]") {
            context = Context::Dependencies;
        } else if let Some(caps) = DEP_ENTRY.captures(trimmed) {
            context = Context::DependencyEntry(caps[1].to_string());
        } else if let Some(caps) = BUILD_DEP_ENTRY.captures(trimmed) {
            context = Context::DependencyEntry(caps[1].to_string());
        } else if let Some(caps) = DEV_DEP_ENTRY.captures(trimmed) {
            if dev_deps {
                context = Context::DependencyEntry(caps[1].to_string());
            }
        } else if trimmed.starts_with("[") {
            if let Context::DependencyEntry(ref dep) = context {
                dependency_pkg_f(dep, &mut new_lines)?;
            }

            context = Context::DontCare;
        } else {
            // TODO: Support `package.version` like stuff (with quotes) at beginning
            match context {
                Context::Package => package_f(line, &mut new_lines)?,
                Context::Dependencies => dependencies_f(line, &mut new_lines)?,
                Context::DependencyEntry(ref dep) => {
                    if let Some(new_context) = dependency_entries_f(dep, line, &mut new_lines)? {
                        context = new_context;
                    }
                }
                _ => {}
            }
        }

        if new_lines.len() == count {
            new_lines.push(line.to_string());
        }
    }

    if let Context::DependencyEntry(ref dep) = context {
        dependency_pkg_f(dep, &mut new_lines)?;
    }

    Ok(new_lines.join(if manifest.contains(CRLF) { CRLF } else { LF }))
}

pub fn rename_packages(
    manifest: String,
    pkg_name: &str,
    renames: &Map<String, String>,
) -> Result<String> {
    parse(
        manifest,
        true,
        |line, new_lines| {
            if let Some(to) = renames.get(pkg_name) {
                if let Some(caps) = NAME.captures(line) {
                    new_lines.push(format!("{}{}{}", &caps[1], to, &caps[3]));
                }
            }

            Ok(())
        },
        |line, new_lines| {
            if let Some(caps) = DEP_DIRECT_NAME.captures(line) {
                if let Some(new_name) = renames.get(&caps[2]) {
                    new_lines.push(format!(
                        "{}{{ version = {}, package = \"{}\" }}{}",
                        &caps[1], &caps[3], new_name, &caps[4]
                    ));
                }
            } else if let Some(caps) = DEP_OBJ_RENAME_NAME.captures(line) {
                rename_dep(caps, new_lines, &renames, 2)?;
            } else if let Some(caps) = DEP_OBJ_NAME.captures(line) {
                if let Some(new_name) = renames.get(&caps[2]) {
                    new_lines.push(format!(
                        "{}, package = \"{}\"{}",
                        &caps[1], new_name, &caps[3]
                    ));
                }
            }

            Ok(())
        },
        |_, line, new_lines| {
            if let Some(caps) = PACKAGE.captures(line) {
                rename_dep(caps, new_lines, &renames, 2)?;
                Ok(Some(Context::DontCare))
            } else {
                Ok(None)
            }
        },
        |dep, new_lines| {
            if let Some(new_name) = renames.get(dep) {
                new_lines.push(format!("package = \"{}\"", new_name));
            }

            Ok(())
        },
    )
}

pub fn change_versions(
    manifest: String,
    pkg_name: &str,
    versions: &Map<String, Version>,
    exact: bool,
) -> Result<String> {
    parse(
        manifest,
        false,
        |line, new_lines| {
            if let Some(new_version) = versions.get(pkg_name) {
                if let Some(caps) = VERSION.captures(line) {
                    new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[3]));
                }
            }

            Ok(())
        },
        |line, new_lines| {
            if let Some(caps) = DEP_DIRECT_VERSION.captures(line) {
                edit_version(caps, new_lines, &versions, exact, 2)?;
            } else if let Some(caps) = DEP_OBJ_RENAME_VERSION.captures(line) {
                edit_version(caps, new_lines, &versions, exact, 5)?;
            } else if let Some(caps) = DEP_OBJ_RENAME_BEFORE_VERSION.captures(line) {
                edit_version(caps, new_lines, &versions, exact, 2)?;
            } else if let Some(caps) = DEP_OBJ_VERSION.captures(line) {
                edit_version(caps, new_lines, &versions, exact, 2)?;
            }

            Ok(())
        },
        |dep, line, new_lines| {
            if let Some(caps) = PACKAGE.captures(line) {
                return Ok(Some(Context::DependencyEntry(caps[2].to_string())));
            } else if let Some(caps) = VERSION.captures(line) {
                if let Some(new_version) = versions.get(dep) {
                    if exact {
                        new_lines.push(format!("{}={}{}", &caps[1], new_version, &caps[3]));
                    } else if !VersionReq::parse(&caps[2])?.matches(new_version) {
                        new_lines.push(format!("{}{}{}", &caps[1], new_version, &caps[3]));
                    }
                }
            }

            Ok(None)
        },
        |_, _| Ok(()),
    )
}

pub fn check_index(name: &str, version: &str) -> Result<()> {
    let index = BareIndex::new_cargo_default();
    let now = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let timeout = Duration::from_secs(300);
    let mut logged = false;

    loop {
        let crate_data = match index.open_or_clone() {
            Ok(mut bare_index) => {
                if let Err(e) = bare_index.retrieve() {
                    Error::IndexUpdate(e).print_err()?;
                    None
                } else {
                    bare_index.crate_(name)
                }
            }
            Err(e) => {
                Error::IndexUpdate(e).print_err()?;
                None
            }
        };

        let published = crate_data
            .iter()
            .flat_map(|c| c.versions().iter())
            .find(|v| v.version() == version)
            .is_some();

        if published {
            break;
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
    fn test_version_dependencies() {
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
    fn test_version_dependencies_object() {
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
    fn test_version_dependencies_object_renamed() {
        let m = r#"
            [dependencies]
            this2 = { path = "../", version = "0.0.1", package = "this" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies]
            this2 = { path = "../", version = "0.3.0", package = "this" } # hello"#
        );
    }

    #[test]
    fn test_version_dependencies_object_renamed_before_version() {
        let m = r#"
            [dependencies]
            this2 = { path = "../", package = "this", version = "0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies]
            this2 = { path = "../", package = "this", version = "0.3.0" } # hello"#
        );
    }

    #[test]
    fn test_version_dependency_table() {
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

    // #[test]
    // fn test_dependency_table_renamed() {
    //     // TODO: Not correct when `package` key exists
    //     let m = r#"
    //         [dependencies.this2]
    //         path = "../"
    //         version = "0.0.1" # hello"
    //         package = "this""#
    //         .to_string();

    //     let mut v = Map::new();
    //     v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

    //     assert_eq!(
    //         change_versions(m, "this", &v, false).unwrap(),
    //         r#"
    //         [dependencies.this2]
    //         path = "../"
    //         version = "0.3.0" # hello"
    //         package = "this""#
    //     );
    // }

    #[test]
    fn test_version_dependency_table_renamed_before_version() {
        let m = r#"
            [dependencies.this2]
            path = "../"
            package = "this"
            version = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m, "this", &v, false).unwrap(),
            r#"
            [dependencies.this2]
            path = "../"
            package = "this"
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

    #[test]
    fn test_name() {
        let m = r#"
            [package]
            name = "this""#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [package]
            name = "ra_this""#
        );
    }

    #[test]
    fn test_name_dependencies() {
        let m = r#"
            [dependencies]
            this = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies]
            this = { version = "0.0.1", package = "ra_this" } # hello"#
        );
    }

    #[test]
    fn test_name_dependencies_object() {
        let m = r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies]
            this = { path = "../", version = "0.0.1", package = "ra_this" } # hello"#
        );
    }

    #[test]
    fn test_name_dependencies_object_renamed() {
        let m = r#"
            [dependencies]
            this2 = { path = "../", version = "0.0.1", package = "this" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies]
            this2 = { path = "../", version = "0.0.1", package = "ra_this" } # hello"#
        );
    }

    #[test]
    fn test_name_dependencies_object_renamed_before_version() {
        let m = r#"
            [dependencies]
            this2 = { path = "../", package = "this", version = "0.0.1" } # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies]
            this2 = { path = "../", package = "ra_this", version = "0.0.1" } # hello"#
        );
    }

    #[test]
    fn test_name_dependency_table() {
        let m = r#"
            [dependencies.this]
            path = "../"
            version = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies.this]
            path = "../"
            version = "0.0.1" # hello
            package = "ra_this""#
        );
    }

    #[test]
    fn test_name_dependency_table_renamed() {
        let m = r#"
            [dependencies.this2]
            path = "../"
            version = "0.0.1" # hello"
            package = "this""#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies.this2]
            path = "../"
            version = "0.0.1" # hello"
            package = "ra_this""#
        );
    }

    #[test]
    fn test_name_dependency_table_renamed_before_version() {
        let m = r#"
            [dependencies.this2]
            path = "../"
            package = "this"
            version = "0.0.1" # hello"#
            .to_string();

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m, "this", &v).unwrap(),
            r#"
            [dependencies.this2]
            path = "../"
            package = "ra_this"
            version = "0.0.1" # hello"#
        );
    }
}
