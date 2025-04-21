# Contributing to `cargo-workspaces`

If you want to contribute to the project, take a look at [open issues](https://github.com/pksunkara/cargo-workspaces/issues)
or create a PR.

Please make sure that new functionality has tests.

## Running tests

The recommended way to run tests for the crate is as follows:

```sh
cargo test --manifest-path cargo-workspaces/Cargo.toml -j1
```

The integration tests manipulate the file system, so running them from multiple threads
may cause race conditions and unexpected failures.

### Adding tests and updating snapshots

`cargo-workspaces` uses [`insta`](https://docs.rs/insta/) for snapshot testing.

When updating tests, you may run tests via:

```
INSTA_UPDATE=always cargo test <args>
```

If you don't want to override any existing snapshots, use:

```
INSTA_UPDATE=unseen cargo test <args>
```

Always make sure that generated snapshots match what you expect the test to produce.
Do not blindly commit changes in snapshots that you do not anticipate.

For more details, check the `insta` documentation.

### Troubleshooting

If you observe unexpected differences in snapshots, you may want to override your compiler to
the same version as used in [CI](.github/workflows/ci.yml), e.g.:

```
cargo override set 1.70
```

The newer versions of `cargo` may produce different output, which would break snapshot tests.
