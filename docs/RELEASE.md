# Release Guide

The Release process is still somewhat manual, and only works on macOS for now.

1. Ensure all changes are pushed, check that CI on the latest commit was green.
  You can also check this badge: ![Master CI Status](https://github.com/gibfahn/up-rs/workflows/Rust/badge.svg)
2. Bump the version in [Cargo.toml](/Cargo.toml).
  <!-- TODO(gib): use a semver-parsing tool to work out the semverness from commit messages. -->
3. Build Linux (static) and Darwin binaries locally:
  ```shell
  cargo test --release # Builds Darwin
  bin/cargo-docker # Builds and tests musl static Linux.
  ```
4. Publish to crates.io:
  ```shell
  cargo publish
  ```
5. Create and push the tag:
  ```shell
  # Set $version to the version you updated the Cargo.toml with.
  git tag $version
  git push $version
  ```
6. Go to the [GitHub Releases][] page and select the latest release.
  - Add the Linux and Darwin binaries with names: `up-Darwin` and `up-Linux`. This allows them to be
    downloaded as `up-$(uname)`.

[GitHub Releases]: https://github.com/gibfahn/up-rs/releases
