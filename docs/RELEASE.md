# Release Guide

The Release process is still somewhat manual, and only works on macOS for now.

## Dependencies

- [clog][], get it with `cargo install clog-cli`.

## Process

1. Ensure all changes are pushed, check that CI on the latest commit was green.
  You can also check this badge: ![Master CI Status](https://github.com/gibfahn/up-rs/workflows/Rust/badge.svg)
2. Generate the changelog:
  ```shell
  # Change --patch to --major or --minor as required.
  clog --patch
  ```
  - Manually update the [CHANGELOG.md][] title to be a link to the release (see existing).
3. Build Linux (static) and Darwin binaries locally:
  ```shell
  cargo test --release # Builds Darwin
  bin/cargo-docker # Builds and tests musl static Linux.
  cargo doc # Check the documentation is buildable.
  ```
4. Commit changes:
  ```shell
  version=$(awk -F\" '/^version = /{print $2; exit}' Cargo.toml)
  git add Cargo.toml Cargo.lock
  git commit -m "Bump version to $version"
  git add CHANGELOG.md
  git commit -m "Update changelog for $version"
  git show # Check version is correct.
  ```
4. Publish to crates.io:
  ```shell
  cargo publish
  ```
5. Create and push the tag:
  ```shell
  # Set $version to the version you updated the Cargo.toml with.
  git tag $version
  git push up $version # Change up to whatever your remote is called.
  ```
6. Go to the [GitHub Releases][] page and click the tag you just pushed and click `Edit Tag`.
  - Release title: $version
  - Add the Linux and Darwin binaries with names: `up-Darwin` and `up-Linux`. This allows them to be
    downloaded as `up-$(uname)`.

[CHANGELOG.md]: /CHANGELOG.md
[GitHub Releases]: https://github.com/gibfahn/up-rs/releases
[clog]: https://github.com/clog-tool/clog-cli
