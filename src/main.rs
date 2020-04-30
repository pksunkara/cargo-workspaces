use cargo_metadata::{CargoOpt, MetadataCommand};
use clap::{AppSettings, Clap};

mod changed;
mod list;
mod publish;
mod version;

mod utils;

use changed::Changed;
use list::List;
use publish::Publish;
use version::Version;

use console::Term;

#[derive(Debug, Clap)]
enum Subcommand {
    #[clap(alias = "ls")]
    List(List),
    Changed(Changed),
    Version(Version),
    Publish(Publish),
}

#[derive(Debug, Clap)]
#[clap(
    version,
    global_setting(AppSettings::VersionlessSubcommands),
    replace("la", &["list", "-a"]),
    replace("ll", &["list", "-l"])
)]
struct Opt {
    #[clap(long)]
    manifest_path: Option<String>,

    #[clap(long)]
    debug: bool,

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

    if opt.debug {
        utils::error::set_debug();
    }

    let mut cmd = MetadataCommand::new();

    cmd.features(CargoOpt::AllFeatures);

    if let Some(path) = opt.manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().unwrap();
    let stdout = Term::stdout();
    let stderr = Term::stderr();

    let err = match opt.subcommand {
        Subcommand::List(x) => x.run(metadata, &stdout, &stderr),
        Subcommand::Changed(x) => x.run(metadata, &stdout, &stderr),
        Subcommand::Version(x) => x.run(metadata, &stdout, &stderr),
        Subcommand::Publish(x) => x.run(metadata, &stdout, &stderr),
    }
    .err();

    if let Some(err) = err {
        err.print(&stderr).unwrap();
    }

    stderr.flush().unwrap();
    stdout.flush().unwrap();
}
