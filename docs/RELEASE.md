# Release Guide

The Release process is still somewhat manual, and only works on macOS for now.

## Dependencies

- [clog][], get it with `cargo install clog-cli`.

## Process

1. Ensure all changes are pushed, check that CI on the latest commit was green.
  You can also check this badge: ![Master CI Status](https://github.com/gibfahn/up-rs/workflows/Rust/badge.svg)
2. Work out old and new versions:
  ```shell
  old_version=$(awk -F\" '/^version = /{print $2; exit}' Cargo.toml)
  read "new_version?New version (old version is $old_version): "
  ```
2. Generate and commit the changelog:
  ```shell
  clog -C CHANGELOG.md --from="$old_version" --setversion="$new_version"
  # Make the header a link pointing to the release.
  gsed -i "s/^## $new_version (/## [$new_version][] (/"  CHANGELOG.md
  echo "[$new_version]: https://github.com/gibfahn/up-rs/releases/tag/$new_version" >> CHANGELOG.md
  git add CHANGELOG.md
  git commit -m "docs(changelog): update changelog for $new_version"
  ```
3. Bump version:
  ```shell
  gsed -i -E "0,/^version = \"$old_version\"\$/s//version = \"$new_version\"/" Cargo.toml
  cargo check # Bumps version in lockfile too.
  git add Cargo.toml Cargo.lock
  git commit -m "docs(version): bump version to $new_version"
  git show # Check version is correct.
  ```
4. Build and test Linux (static) and Darwin binaries locally:
  ```shell
  cargo test --release --ignored # Builds Darwin
  bin/cargo-docker # Builds and tests musl static Linux.
  cargo doc # Check the documentation is buildable.
  ```
5. Publish to crates.io:
  ```shell
  cargo publish
  ```
6. Create and push the tag, and create a release:
  ```shell
  git tag "$new_version"
  git push up "$new_version"
  # This allows them to be downloaded as `up-$(uname)`.
  cp target/release/up up-Darwin
  cp target/x86_64-unknown-linux-musl/release/up up-Linux
  hub release create --commitish=master --browse \
    --attach=up-Darwin --attach=up-Linux \
    -F- <<<"$(clog --from="$old_version" --setversion="$new_version")" \
    "$new_version"
  rm up-Darwin up-Linux
  ```
6. Go to the [GitHub Releases][] page and check everything is working properly.

[CHANGELOG.md]: /CHANGELOG.md
[GitHub Releases]: https://github.com/gibfahn/up-rs/releases
[clog]: https://github.com/clog-tool/clog-cli
