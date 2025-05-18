# Changelog

## 0.4.0

### BREAKING
* Update MSRV to 1.78.0

### Enhancements
* Allow `init` subcommand to use existing manifest
* Improve error message when no public packages found with `list`
* Added support for `2024` edition when creating package

## 0.3.6

### Bug Fixes
* Fix issue with Ctrl+C termination
* Fix issue with ignoring dev-dependencies for targets when publishing

## 0.3.5

### Enhancements
* Allow error on no changes detected
* Added `plan` subcommand to list crates to be published
* Add dry-run mode for publishing

## 0.3.4

### Bug Fixes
* Fixes issue with listing public members when private members exist

## 0.3.3

### Enhancements
* Allow ignore private crates for `exec`
* Listing now follows DAG order of the dependencies

## 0.3.2

### Bug Fixes
* Fix issue with creating a package with newer versions of Rust

## 0.3.1

### Enhancements
* Infer package name when creating a workspace member crate
* Improve time taken for versioning and releasing
* Support TLS and authentication for sparse registries

### Bug Fixes
* Fix issue with publishing to sparse registries

## 0.3.0

### BREAKING
* Update MSRV to 1.70.0

### Enhancements
* Renamed `--from-git` flag to `--publish-as-is`
* Add new workspace member entry automatically when creating it

### Bug Fixes
* Respect protocols in registry URLs

## 0.2.44

### Enhancements
* Better recognition on when to ignore dev-dependencies, avoids some publishing issues with `--from-git` flag

## 0.2.43

### Enhancements
* Ignore dev-dependencies when publishing to Cargo, avoids some versioning issues

### Bug Fixes
* Respect `registry` option when checking index during publishing

## 0.2.42

### Enhancements
* Added `ignore` flag to `exec` subcommand

## 0.2.41

### Bug Fixes
* Fix bug with some dependency entries not being updated

## 0.2.39

### Bug Fixes
* Fix bug with not updating version in `workspace.dependencies`

## 0.2.38

### Enhancements
* Supports cargo workspace inheritance

## 0.2.37

### Enhancements
* Added `skip` option during versioning

### Bug Fixes
* Restores cursor if versioning is cancelled

## 0.2.36

### Enhancements
* Improve the glob pattern support allowed in arguments of subcommands

## 0.2.35

### Enhancements
* Allow renaming single crates

## 0.2.34

### Enhancements
* Added `registry` flag to `publish` subcommand

## 0.2.33

### Bug Fixes
* Support target dependencies when changing version and renaming packages

## 0.2.30

### Bug Fixes
* Remove some flakiness in detecting git command success

## 0.2.29

### Enhancements
* Added `lib`, `bin` flags to `create` subcommand
* Added `edition`, `name` options to `create` subcommand

## 0.2.28

### Enhancements
* Support reading some options from manifests

## 0.2.26

### Enhancements
* Support private registries
* Skipping published crates is now the default behaviour

## 0.2.24

### Bug fixes
* Don't add untracked files when publishing/versioning

## 0.2.23

### Enhancements
* Added `--no-global-tag` flag

## 0.2.17

### Enhancements
* Treat `main` branch similarily to `master`

## 0.2.16

### Enhancements
* Forward verbose to cargo where possible

## 0.2.15

### Enhancements
* Added init subcommand

## 0.2.14

### Bug Fixes
* Allow tag prefix to be actually empty.

### Enhancements
* Executing a command now follows DAG order of the dependencies.

## 0.2.12

### Enhancements
* Allow dirty working directories to be published

## 0.2.11

### Bug Fixes
* Support cases where dependencies are renamed

### Enhancements
* Added rename subcommand

## 0.2.10

### Bug Fixes
* Improve tag pushing to work with followTags config

## 0.2.9

### Enhancements
* Added custom option to skipping prompt

## 0.2.8

### Bug Fixes
* Fix issue with crates-index not being up to date even after refreshing

## 0.2.4

### Bug Fixes
* Verify each crate during publishing only and not before
* Wait for crates-index to be up to date before publishing the next package

### Enhancements
* Added option to skip verification

## 0.2.3

### Bug Fixes
* Improve detection of LF

## 0.2.2

### Bug Fixes
* Improve change detection on windows

## 0.2.1

### Enhancements
* Don't complain about no changes when force option is specified during versioning

## 0.2.0

#### Breaking
* Improved the next version determination for prereleases

#### Enhancements
* Added prerelease identifier selection option for versioning
* Added prerelease option to skipping prompt

## 0.1.9

#### Enhancements
* Update Cargo.lock for the versioned packages

## 0.1.8

#### Enhancements
* Improved CI usage by implementing prompt skipping

## 0.1.7

#### Enhancements
* Allow versioning for private packages

## 0.1.5

#### Bug Fixes
* Verify all the crates first before publishing
* Fixed windows LF issues with git

## 0.1.4

#### Enhancements
* Annotate generated tags
* Allow individual tag prefixes

## 0.1.3

#### Enhancements
* Add readme to crates.io

## 0.1.2

#### Bug Fixes
* Fixed path issues with long listing crates on windows

## 0.1.1

* Initial release
