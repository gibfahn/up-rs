# Release Guide

The Release process is still somewhat manual, and only works on macOS for now.

## Dependencies

- [clog][], get it with `cargo install clog-cli`.
- [GitHub CLI][], get it with `brew install gh`.

## Process

1. Ensure all changes are pushed, check that CI on the latest commit was green.
  You can also check this badge: ![Master CI Status](https://github.com/gibfahn/up-rs/workflows/Rust/badge.svg)
2. Run the [bin/release.sh][] script.
3. Go to the [GitHub Releases][] page and check everything is working properly.

[CHANGELOG.md]: /CHANGELOG.md
[GitHub CLI]: https://github.com/cli/cli
[GitHub Releases]: https://github.com/gibfahn/up-rs/releases
[clog]: https://github.com/clog-tool/clog-cli
