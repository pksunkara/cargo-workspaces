use crate::utils::{debug, get_debug, info, Error, Result, INTERNAL_ERR};

use camino::Utf8Path;
use crates_index::Index;
use lazy_static::lazy_static;
use oclif::term::TERM_ERR;
use regex::{Captures, Regex};
use semver::{Version, VersionReq};

use std::{
    collections::BTreeMap as Map,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};

const CRLF: &str = "\r\n";
const LF: &str = "\n";

lazy_static! {
    static ref NAME: Regex =
        Regex::new(r#"^(\s*['"]?name['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*)$"#).expect(INTERNAL_ERR);
    static ref VERSION: Regex =
        Regex::new(r#"^(\s*['"]?version['"]?\s*=\s*['"])([^'"]+)(['"].*)$"#)
            .expect(INTERNAL_ERR);
    static ref PACKAGE: Regex =
        Regex::new(r#"^(\s*['"]?package['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*)$"#).expect(INTERNAL_ERR);
    static ref DEP_TABLE: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?dependencies]"#).expect(INTERNAL_ERR);
    static ref DEP_ENTRY: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?dependencies\.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref BUILD_DEP_TABLE: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?build-dependencies]"#).expect(INTERNAL_ERR);
    static ref BUILD_DEP_ENTRY: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?build-dependencies\.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
    static ref DEV_DEP_TABLE: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?dev-dependencies]"#).expect(INTERNAL_ERR);
    static ref DEV_DEP_ENTRY: Regex =
        Regex::new(r#"^\[(target\.'?([^']+)'?\.|workspace\.)?dev-dependencies\.([0-9A-Za-z-_]+)]"#).expect(INTERNAL_ERR);
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
        Regex::new(r#"^(\s*['"]?([0-9A-Za-z-_]+)['"]?\s*=\s*\{(.*[^\s])?)(\s*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref DEP_OBJ_RENAME_NAME: Regex =
        Regex::new(r#"^(\s*['"]?[0-9A-Za-z-_]+['"]?\s*=\s*\{.*['"]?package['"]?\s*=\s*['"])([0-9A-Za-z-_]+)(['"].*}.*)$"#)
            .expect(INTERNAL_ERR);
    static ref WORKSPACE_KEY: Regex =
        Regex::new(r#"['"]?workspace['"]?\s*=\s*true"#).expect(INTERNAL_ERR);
}

pub fn cargo<'a>(
    root: &Utf8Path,
    args: &[&'a str],
    env: &[(&'a str, &'a str)],
) -> Result<(String, String)> {
    debug!("cargo", args.join(" "));

    let mut args = args.to_vec();

    if TERM_ERR.features().colors_supported() {
        args.push("--color");
        args.push("always");
    }

    if get_debug() {
        args.push("-v");
    }

    let args_text = args.iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let mut stderr_lines = vec![];

    let mut child = Command::new("cargo")
        .current_dir(root)
        .args(&args)
        .envs(env.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| Error::Cargo {
            err,
            args: args_text.clone(),
        })?;

    {
        let stderr = child.stderr.as_mut().expect(INTERNAL_ERR);

        for line in BufReader::new(stderr).lines() {
            let line = line?;

            eprintln!("{}", line);
            stderr_lines.push(line);
        }
    }

    let output = child.wait_with_output().map_err(|err| Error::Cargo {
        err,
        args: args_text,
    })?;

    let output_stdout = String::from_utf8(output.stdout)?;
    let output_stderr = stderr_lines.join("\n");

    debug!("cargo stderr", output_stderr);
    debug!("cargo stdout", output_stdout);

    Ok((
        output_stdout.trim().to_owned(),
        output_stderr.trim().to_owned(),
    ))
}

pub fn cargo_config_get(root: &Utf8Path, name: &str) -> Result<String> {
    // You know how we sometimes have to make the best of an unfortunate
    // situation? This is one of those situations.
    //
    // In order to support private registries, we need to know the URL of their
    // index. That's stored in a `.cargo/config.toml` file, which could be in
    // the `root` directory, or someplace else, like `~/.cargo/config.toml`.
    //
    // In order to match cargo's lookup strategy, the best option is to use
    // `cargo config get`. However, that's unstable. Since we don't want
    // cargo-workspaces to require nightly, we can use two combined escape
    // hatches:
    //
    // 1. Set the `RUSTC_BOOTSTRAP` environment variable to `1`
    // 2. Pass `-Z unstable-options` to cargo
    //
    // This works because stable rustc versions contain exactly the same code as
    // nightly versions, but all the nightly features are gated. This allows
    // stable rustc versions to recognize nightly features and tell you: "no, you
    // need a nightly for this". But rustc should be able to compile rustc, and
    // the rustc codebase uses nightly features, so `RUSTC_BOOTSTRAP` removes that
    // gating.
    //
    // This is generally frowned upon (it's only supposed to be used to
    // bootstrap rustc), but here it's _just_ to get access to `cargo config`,
    // we're not actually building crates with
    // rustc-stable-masquerading-as-nightly.

    debug!("cargo config get", name);

    let args = vec!["-Z", "unstable-options", "config", "get", name];
    let env = &[("RUSTC_BOOTSTRAP", "1")];

    let (stdout, _) = cargo(root, &args, env)?;

    // `cargo config get` returns TOML output, like so:
    //
    //      $ RUSTC_BOOTSTRAP=1 cargo -Z unstable-options config get registries.foobar.index
    //      registries.foobar.index = "https://dl.cloudsmith.io/basic/some-org/foobar/cargo/index.git"
    //
    // The right thing to do is probably to pull in a TOML crate, but since the
    // output is so predictable, and in the interest of keeping dependencies low,
    // we just do some text wrangling instead:

    // tokens is ["registries.foobar.index", "\"some-url\""]
    let tokens = stdout
        .split(" = ")
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    // value is "\"some-url\""
    let value = tokens.get(1).ok_or(Error::BadConfigGetOutput(stdout))?;

    // we return "some-url"
    Ok(value
        .trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .into())
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

        #[allow(clippy::if_same_then_else)]
        if trimmed.starts_with("[package]") || trimmed.starts_with("[workspace.package]") {
            context = Context::Package;
        } else if let Some(_) = DEP_TABLE.captures(trimmed) {
            context = Context::Dependencies;
        } else if let Some(_) = BUILD_DEP_TABLE.captures(trimmed) {
            context = Context::Dependencies;
        } else if let Some(_) = DEV_DEP_TABLE.captures(trimmed) {
            // TODO: let-chain
            if dev_deps {
                context = Context::Dependencies;
            }
        } else if let Some(caps) = DEP_ENTRY.captures(trimmed) {
            context = Context::DependencyEntry(caps[3].to_string());
        } else if let Some(caps) = BUILD_DEP_ENTRY.captures(trimmed) {
            context = Context::DependencyEntry(caps[3].to_string());
        } else if let Some(caps) = DEV_DEP_ENTRY.captures(trimmed) {
            // TODO: let-chain
            if dev_deps {
                context = Context::DependencyEntry(caps[3].to_string());
            }
        } else if trimmed.starts_with('[') {
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
                rename_dep(caps, new_lines, renames, 2)?;
            } else if let Some(caps) = DEP_OBJ_NAME.captures(line) {
                if let Some(new_name) = renames.get(&caps[2]) {
                    if WORKSPACE_KEY.captures(&caps[3]).is_none() {
                        new_lines.push(format!(
                            "{}, package = \"{}\"{}",
                            &caps[1], new_name, &caps[4]
                        ));
                    }
                }
            }

            Ok(())
        },
        |_, line, new_lines| {
            if let Some(caps) = PACKAGE.captures(line) {
                rename_dep(caps, new_lines, renames, 2)?;
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
                edit_version(caps, new_lines, versions, exact, 2)?;
            } else if let Some(caps) = DEP_OBJ_RENAME_VERSION.captures(line) {
                edit_version(caps, new_lines, versions, exact, 5)?;
            } else if let Some(caps) = DEP_OBJ_RENAME_BEFORE_VERSION.captures(line) {
                edit_version(caps, new_lines, versions, exact, 2)?;
            } else if let Some(caps) = DEP_OBJ_VERSION.captures(line) {
                edit_version(caps, new_lines, versions, exact, 2)?;
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

pub fn is_published(index: &mut Index, name: &str, version: &str) -> Result<bool> {
    // See if we already have the crate (and version) in cache
    if let Some(crate_data) = index.crate_(name) {
        if crate_data.versions().iter().any(|v| v.version() == version) {
            return Ok(true);
        }
    }

    // We don't? Okay, update the cache then
    index.update()?;

    // Try again with updated index:
    if let Some(crate_data) = index.crate_(name) {
        if crate_data.versions().iter().any(|v| v.version() == version) {
            return Ok(true);
        }
    }

    // I guess we didn't have it
    Ok(false)
}

pub fn check_index(index: &mut Index, name: &str, version: &str) -> Result<()> {
    let now = Instant::now();
    let sleep_time = Duration::from_secs(2);
    let timeout = Duration::from_secs(300);
    let mut logged = false;

    loop {
        if is_published(index, name, version)? {
            break;
        } else if timeout < now.elapsed() {
            return Err(Error::PublishTimeout);
        }

        if !logged {
            info!("waiting", "...");
            logged = true;
        }

        sleep(sleep_time);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_version() {
        let m = indoc! {r#"
            [package]
            version = "0.1.0"
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "this", &v, false).unwrap(),
            indoc! {r#"
                [package]
                version = "0.3.0""#
            }
        );
    }

    #[test]
    fn test_version_comments() {
        let m = indoc! {r#"
            [package]
            version="0.1.0" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "this", &v, false).unwrap(),
            indoc! {r#"
                [package]
                version="0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_quotes() {
        let m = indoc! {r#"
            [package]
            "version"	=	"0.1.0"
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "this", &v, false).unwrap(),
            indoc! {r#"
                [package]
                "version"	=	"0.3.0""#
            }
        );
    }

    #[test]
    fn test_version_single_quotes() {
        let m = indoc! {r#"
            [package]
            'version'='0.1.0'# hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "this", &v, false).unwrap(),
            indoc! {r#"
                [package]
                'version'='0.3.0'# hello"#
            }
        );
    }

    #[test]
    fn test_version_workspace() {
        let m = indoc! {r#"
            [workspace.package]
            version = "0.1.0"
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "this", &v, false).unwrap(),
            indoc! {r#"
                [workspace.package]
                version = "0.3.0""#
            }
        );
    }

    #[test]
    fn test_version_dependencies() {
        let m = indoc! {r#"
            [dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this = "0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_dependencies_object() {
        let m = indoc! {r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { path = "../", version = "0.3.0" } # hello"#
            }
        );
    }

    #[test]
    fn test_version_dependencies_object_renamed() {
        let m = indoc! {r#"
            [dependencies]
            this2 = { path = "../", version = "0.0.1", package = "this" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this2 = { path = "../", version = "0.3.0", package = "this" } # hello"#
            }
        );
    }

    #[test]
    fn test_version_dependencies_object_renamed_before_version() {
        let m = indoc! {r#"
            [dependencies]
            this2 = { path = "../", package = "this", version = "0.0.1" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this2 = { path = "../", package = "this", version = "0.3.0" } # hello"#
            }
        );
    }

    #[test]
    fn test_version_dependency_table() {
        let m = indoc! {r#"
            [dependencies.this]
            path = "../"
            version = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies.this]
                path = "../"
                version = "0.3.0" # hello"#
            }
        );
    }

    // #[test]
    // fn test_dependency_table_renamed() {
    //     // TODO: Not correct when `package` key exists
    //     let m = indoc! {r#"
    //         [dependencies.this2]
    //         path = "../"
    //         version = "0.0.1" # hello"
    //         package = "this"
    //     "#};

    //     let mut v = Map::new();
    //     v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

    //     assert_eq!(
    //         change_versions(m, "another", &v, false).unwrap(),
    //         indoc! {r#"
    //             [dependencies.this2]
    //             path = "../"
    //             version = "0.3.0" # hello"
    //             package = "this""#
    //         }
    //     );
    // }

    #[test]
    fn test_version_dependency_table_renamed_before_version() {
        let m = indoc! {r#"
            [dependencies.this2]
            path = "../"
            package = "this"
            version = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies.this2]
                path = "../"
                package = "this"
                version = "0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_target_dependencies() {
        let m = indoc! {r#"
            [target.x86_64-pc-windows-gnu.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [target.x86_64-pc-windows-gnu.dependencies]
                this = "0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_target_cfg_dependencies() {
        let m = indoc! {r#"
            [target.'cfg(not(any(target_arch = "wasm32", target_os = "emscripten")))'.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [target.'cfg(not(any(target_arch = "wasm32", target_os = "emscripten")))'.dependencies]
                this = "0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_workspace_dependencies() {
        let m = indoc! {r#"
            [workspace.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [workspace.dependencies]
                this = "0.3.0" # hello"#
            }
        );
    }

    #[test]
    fn test_version_ignore_workspace() {
        let m = indoc! {r#"
            [dependencies]
            this = { workspace = true } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { workspace = true } # hello"#
            }
        );
    }

    #[test]
    fn test_version_ignore_dotted_workspace() {
        let m = indoc! {r#"
            [dependencies]
            this.workspace = true # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, false).unwrap(),
            indoc! {r#"
                [dependencies]
                this.workspace = true # hello"#
            }
        );
    }

    #[test]
    fn test_exact() {
        let m = indoc! {r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), Version::parse("0.3.0").unwrap());

        assert_eq!(
            change_versions(m.into(), "another", &v, true).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { path = "../", version = "=0.3.0" } # hello"#
            }
        );
    }

    #[test]
    fn test_name() {
        let m = indoc! {r#"
            [package]
            name = "this"
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "this", &v).unwrap(),
            indoc! {r#"
                [package]
                name = "ra_this""#
            }
        );
    }

    #[test]
    fn test_name_dependencies() {
        let m = indoc! {r#"
            [dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_dependencies_object() {
        let m = indoc! {r#"
            [dependencies]
            this = { path = "../", version = "0.0.1" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { path = "../", version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_dependencies_object_renamed() {
        let m = indoc! {r#"
            [dependencies]
            this2 = { path = "../", version = "0.0.1", package = "this" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this2 = { path = "../", version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_dependencies_object_renamed_before_version() {
        let m = indoc! {r#"
            [dependencies]
            this2 = { path = "../", package = "this", version = "0.0.1" } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this2 = { path = "../", package = "ra_this", version = "0.0.1" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_dependency_table() {
        let m = indoc! {r#"
            [dependencies.this]
            path = "../"
            version = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies.this]
                path = "../"
                version = "0.0.1" # hello
                package = "ra_this""#
            }
        );
    }

    #[test]
    fn test_name_dependency_table_renamed() {
        let m = indoc! {r#"
            [dependencies.this2]
            path = "../"
            version = "0.0.1" # hello"
            package = "this"
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies.this2]
                path = "../"
                version = "0.0.1" # hello"
                package = "ra_this""#
            }
        );
    }

    #[test]
    fn test_name_dependency_table_renamed_before_version() {
        let m = indoc! {r#"
            [dependencies.this2]
            path = "../"
            package = "this"
            version = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies.this2]
                path = "../"
                package = "ra_this"
                version = "0.0.1" # hello"#
            }
        );
    }

    #[test]
    fn test_name_target_dependencies() {
        let m = indoc! {r#"
            [target.x86_64-pc-windows-gnu.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [target.x86_64-pc-windows-gnu.dependencies]
                this = { version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_target_cfg_dependencies() {
        let m = indoc! {r#"
            [target.'cfg(not(any(target_arch = "wasm32", target_os = "emscripten")))'.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [target.'cfg(not(any(target_arch = "wasm32", target_os = "emscripten")))'.dependencies]
                this = { version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_workspace_dependencies() {
        let m = indoc! {r#"
            [workspace.dependencies]
            this = "0.0.1" # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [workspace.dependencies]
                this = { version = "0.0.1", package = "ra_this" } # hello"#
            }
        );
    }

    #[test]
    fn test_name_ignore_workspace() {
        let m = indoc! {r#"
            [dependencies]
            this = { workspace = true } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { workspace = true } # hello"#
            }
        );
    }

    #[test]
    fn test_name_ignore_workspace_with_keys() {
        let m = indoc! {r#"
            [dependencies]
            this = { workspace = true, optional = true } # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this = { workspace = true, optional = true } # hello"#
            }
        );
    }

    #[test]
    fn test_name_ignore_dotted_workspace() {
        let m = indoc! {r#"
            [dependencies]
            this.workspace = true # hello
        "#};

        let mut v = Map::new();
        v.insert("this".to_string(), "ra_this".to_string());

        assert_eq!(
            rename_packages(m.into(), "another", &v).unwrap(),
            indoc! {r#"
                [dependencies]
                this.workspace = true # hello"#
            }
        );
    }
}
