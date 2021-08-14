mod changed;
mod create;
mod exec;
mod init;
mod list;
mod publish;
mod rename;
mod version;

mod utils;

use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::{AppSettings, Clap};
use oclif::finish;

#[derive(Debug, Clap)]
enum Subcommand {
    // TODO: add
    List(list::List),
    Changed(changed::Changed),
    Version(version::Version),
    Publish(publish::Publish),
    Exec(exec::Exec),
    Create(create::Create),
    Rename(rename::Rename),
    Init(init::Init),
}

#[derive(Debug, Clap)]
#[clap(
    version,
    global_setting(AppSettings::DisableVersionForSubcommands),
    replace("la", &["list", "-a"]),
    replace("ll", &["list", "-l"])
)]
struct Opt {
    /// Path to workspace Cargo.toml
    #[clap(long, value_name = "path")]
    manifest_path: Option<String>,

    /// Verbose mode
    #[clap(short)]
    verbose: bool,

    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, Clap)]
#[clap(
    name = "cargo-workspaces",
    bin_name = "cargo",
    version,
    global_setting(AppSettings::ColoredHelp)
)]
enum Cargo {
    #[clap(alias = "ws")]
    Workspaces(Opt),
}

fn main() {
    let Cargo::Workspaces(opt) = Cargo::parse();

    if opt.verbose {
        utils::set_debug();
    }

    let result = if let Subcommand::Init(ref init) = opt.subcommand {
        init.run()
    } else {
        let mut cmd = MetadataCommand::new();

        cmd.features(CargoOpt::AllFeatures);
        cmd.no_deps();

        if let Some(path) = opt.manifest_path {
            cmd.manifest_path(path);
        }

        let metadata = cmd.exec().unwrap();

        match opt.subcommand {
            Subcommand::List(x) => x.run(metadata),
            Subcommand::Changed(x) => x.run(metadata),
            Subcommand::Version(x) => x.run(metadata),
            Subcommand::Publish(x) => x.run(metadata),
            Subcommand::Exec(x) => x.run(metadata),
            Subcommand::Create(x) => x.run(metadata),
            Subcommand::Rename(x) => x.run(metadata),
            _ => unreachable!(),
        }
    };

    finish(result)
}
