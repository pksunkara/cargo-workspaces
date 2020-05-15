<!-- omit in TOC -->
# cargo-workspaces

Inspired by [Lerna](https://lerna.js.org/)

A tool that optimizes the workflow around cargo workspaces with `git` and `cargo` by providing utilities to
version, publish, execute commands and more.

I made this to work on [clap](https://github.com/clap-rs/clap) and other projects that rely on workspaces.
But this will also work on single crates because by default every individual crate is a workspace.

1. [Installation](#installation)
2. [Usage](#usage)
   1. [Create](#create)
   2. [List](#list)
   3. [Changed](#changed)
   4. [Exec](#exec)
   5. [Version](#version)
      1. [Fixed or Independent](#fixed-or-independent)
   6. [Publish](#publish)
3. [Changelog](#changelog)

## Installation

```
cargo install cargo-workspaces
```

## Usage

The installed tool can be called by `cargo workspaces` or `cargo ws`. Both of them point to the same.

You can use `cargo ws help` or `cargo ws help <subcmd>` anytime to understand allowed options.

The basic commands available for this tool are given below. Assuming you run them inside a cargo workspace.

### Create

Interactively creates a new crate in the workspace. *We recommend using this instead of `cargo new`*. All
the crates start with `0.0.0` version because the [version](#version) is responsible for determining the
version.

```
USAGE:
    cargo workspaces create <path>

ARGS:
    <path>    Path for the crate relative to the workspace manifest

FLAGS:
    -h, --help    Prints help information
```

### List

Lists crates in the workspace.

```
USAGE:
    cargo workspaces list [FLAGS]

FLAGS:
    -a, --all     Show private crates that are normally hidden
    -h, --help    Prints help information
        --json    Show information as a JSON array
    -l, --long    Show extended information
```

Several aliases are available.

* `cargo ws ls` implies `cargo ws list`
* `cargo ws ll` implies `cargo ws list --long`
* `cargo ws la` implies `cargo ws list --all`

### Changed

List crates that have changed since the last git tag. This is useful to see the list of crates that
would be the subjects of the next [version](#version) or [publish](#publish) command.

```
USAGE:
    cargo workspaces changed [FLAGS] [OPTIONS]

FLAGS:
    -a, --all                    Show private crates that are normally hidden
    -h, --help                   Prints help information
        --include-merged-tags    Include tags from merged branches
        --json                   Show information as a JSON array
    -l, --long                   Show extended information

OPTIONS:
        --force <pattern>             Always include targeted crates matched by glob
        --ignore-changes <pattern>    Ignore changes in files matched by glob
        --since <since>               Use this git reference instead of the last tag
```

### Exec

Executes an arbitrary command in each crate of the workspace.

```
USAGE:
    cargo workspaces exec [FLAGS] <args>...

ARGS:
    <args>...

FLAGS:
    -h, --help       Prints help information
        --no-bail    Continue executing command despite non-zero exit in a given crate
```

For example, if you want to run `ls -l` in each crate, you can simply do `cargo ws exec ls -l`.

### Version

Bump versions of the crates in the worksapce. This command does the following:

1. Identifies crates that have been updated since the previous tagged release
2. Prompts for a new version according to the crate
3. Modifies crate manifest to reflect new release
4. Update intra-workspace dependency version constraints if needed
5. Commits those changes
6. Tags the commit
7. Pushes to the git remote

You can influence the above steps with the flags and options for this command.

```
USAGE:
    cargo workspaces version [FLAGS] [OPTIONS]

FLAGS:
    -a, --all                    Also do versioning for private crates (will not be published)
        --amend                  Amend the existing commit, instead of generating a new one
        --exact                  Specify inter dependency version numbers exactly with `=`
    -h, --help                   Prints help information
        --include-merged-tags    Include tags from merged branches
        --no-git-commit          Do not commit version changes
        --no-git-push            Do not push generated commit and tags to git remote
        --no-git-tag             Do not tag generated commit
        --no-individual-tags     Do not tag individual versions for crates

OPTIONS:
        --allow-branch <pattern>            Specify which branches to allow from [default: master]
        --force <pattern>                   Always include targeted crates matched by glob
        --git-remote <remote>               Push git changes to the specified remote [default: origin]
        --ignore-changes <pattern>          Ignore changes in files matched by glob
        --individual-tag-prefix <prefix>    Customize prefix for individual tags (should contain `%n`) [default: %n@]
    -m, --message <message>                 Use a custom commit message when creating the version commit
        --tag-prefix <prefix>               Customize tag prefix (can be empty) [default: v]
```

#### Fixed or Independent

By default, all the crates in the workspace will share a single version. But if you want the crate to have
it's version be independent of the other crates, you can add the following to that crate:

```toml
[package.metadata.workspaces]
independent = true
```

### Publish

Publish all the crates from the workspace in the correct order according to the dependencies. By default,
this command runs [version](#version) first. If you do not want that to happen, you can supply the
`--from-git` option.

```
USAGE:
    cargo workspaces publish [FLAGS] [OPTIONS]

FLAGS:
    -a, --all                    Also do versioning for private crates (will not be published)
        --amend                  Amend the existing commit, instead of generating a new one
        --exact                  Specify inter dependency version numbers exactly with `=`
        --from-git               Publish crates from the current commit without versioning
    -h, --help                   Prints help information
        --include-merged-tags    Include tags from merged branches
        --no-git-commit          Do not commit version changes
        --no-git-push            Do not push generated commit and tags to git remote
        --no-git-tag             Do not tag generated commit
        --no-individual-tags     Do not tag individual versions for crates
        --skip-published         Allow skipping already published crate versions

OPTIONS:
        --allow-branch <pattern>            Specify which branches to allow from [default: master]
        --force <pattern>                   Always include targeted crates matched by glob
        --git-remote <remote>               Push git changes to the specified remote [default: origin]
        --ignore-changes <pattern>          Ignore changes in files matched by glob
        --individual-tag-prefix <prefix>    Customize prefix for individual tags (should contain `%n`) [default: %n@]
    -m, --message <message>                 Use a custom commit message when creating the version commit
        --tag-prefix <prefix>               Customize tag prefix (can be empty) [default: v]
```

<!-- omit in TOC -->
## Contributors
Here is a list of [Contributors](http://github.com/pksunkara/cargo-workspaces/contributors)

<!-- omit in TOC -->
### TODO

## Changelog
Please see [CHANGELOG.md](CHANGELOG.md).

<!-- omit in TOC -->
## License
MIT/X11

<!-- omit in TOC -->
## Bug Reports
Report [here](http://github.com/pksunkara/cargo-workspaces/issues).

<!-- omit in TOC -->
## Creator
Pavan Kumar Sunkara (pavan.sss1991@gmail.com)

Follow me on [github](https://github.com/users/follow?target=pksunkara), [twitter](http://twitter.com/pksunkara)
