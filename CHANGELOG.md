# Changelog

## 0.2.13

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
