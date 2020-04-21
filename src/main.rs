use cargo_metadata::MetadataCommand;
use clap::{AppSettings, Clap};

mod list;
mod utils;

use list::List;

#[derive(Debug, Clap)]
enum Subcommand {
    #[clap(alias = "ls")]
    List(List),
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
    #[clap(name = "workspaces", alias = "ws")]
    Workspaces(Opt),
}

fn main() {
    let Cargo::Workspaces(opt) = Cargo::parse();
    let mut cmd = MetadataCommand::new();

    if let Some(path) = opt.manifest_path {
        cmd.manifest_path(path);
    }

    let metadata = cmd.exec().unwrap();

    match opt.subcommand {
        Subcommand::List(list) => list.run(metadata),
    }
    .unwrap();
}
