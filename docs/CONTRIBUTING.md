# Contributing Guide

## Building & Testing

### macOS

The simplest way to develop this app is to build the dynamically-linked Darwin (macOS) target.

#### macOS Install

First you need to install Rust. The easiest way is to use [rustup][].

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

If you don't want to install to the default locations, just set the variables correctly.

```bash
# You need to set $RUSTUP_HOME and $CARGO_HOME in your rc file and add $CARGO_HOME/bin to your path.
RUSTUP_HOME="$XDG_DATA_HOME"/rustup CARGO_HOME="$XDG_DATA_HOME"/cargo \
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs           \
  | sh -s -- -y --no-modify-path
```

#### macOS Build

You can then build and run this project with:

```bash
cargo build                          # Debug   binary in target/debug/up
cargo build --release                # Release binary in target/release/up

cargo run --release -- <args>        # Build the release binary and run it with <args>.

./target/release/up --help # Run release binary directly.
```

The compiled binary can be copied anywhere.

#### macOS Test

```bash
cargo test                        # Run tests in debug mode.
cargo test --release              # Run tests in release mode.
cargo test --release -- --ignored # Run all tests in release mode (what CI runs).
cargo test --no-fail-fast         # Don't stop at first error.

cargo test --test test_module     # Only run tests in <test_module> (e.g. file in tests/).
cargo test -- test_name           # Only test functions containing <test_name>.
```

### Linux

The binary that is used in Rio is a statically-linked Linux binary.

#### Docker Linux Build

The easiest way to build is with docker.

Run this script to build and test locally.

```bash
bin/cargo-docker
```

#### Cross-Compiled Linux Build

See [this blog post][Rust Barebones] for more info.

```bash
brew install FiloSottile/musl-cross/musl-cross
rustup target install x86_64-unknown-linux-musl
grep -q 'linker = "x86_64-linux-musl-gcc"' "${CARGO_HOME}"/config \
  || echo -e '[target.x87_64-unknown-linux-musl]
  linker = "x86_64-linux-musl-gcc"' >> "${CARGO_HOME:-~/.cargo}"/config
TARGET_CC=x86_64-linux-musl-gcc cargo build --release --target=x86_64-unknown-linux-musl
```

## Development

### Helpful tools:

You probably want to install some extra rust components to make life easier:

- [`rls`][]: the Rust Language Server. Provides IDE capabilities for anything with
  [Language Server Protocol][] support.
- [`clippy`][]: warns about unidiomatic or dangerous code.
- [`rustfmt`][]: use `cargo fmt` to auto-format code in this repo
- `rust-docs`: offline rust docs, open with `rustup doc`.
- [`cargo-edit`]: Provides `cargo add`, `cargo rm`, and `cargo upgrade` for dependency management.
- [`cargo-watch`]: Run commands when files change (are saved).

```bash
# Use the latest stable rust by default.
cargo default stable

# Add useful rust components.
rustup component add rls rust-analysis rust-src clippy rustfmt rust-docs

cargo install cargo-edit cargo-watch
```

### Updating

You can keep your build tools up to date with:

```bash
# Updates rustup and installed toolchains + components.
rustup update

# Updates everything installed globally with `cargo install`.
cargo install-update --all
```

### Debugging

Rust code can be debugged with either [`gdb`][] or [`lldb`][]. VS Code or CLion
should both work.

See these links for more info:
- [StackOverflow][SO rust debugging]
- [Some blog post][Blog rust debugging]

[Blog rust debugging]: https://bryce.fisher-fleig.org/blog/debugging-rust-programs-with-lldb/index.html
[SO rust debugging]: https://stackoverflow.com/questions/37586216/step-by-step-interactive-debugger-for-rust
[`gdb`]: https://www.gnu.org/software/gdb/
[`lldb`]: https://lldb.llvm.org/

### Show docs

You can show the Rust documentation for the current version of Rust with:

```bash
rustup doc
```

You can show the docs for the current versions of this tool and all its dependencies with:

```bash
# Use --document-private-items to see doc comments for private functions/items/modules.
cargo doc --open --document-private-items
```

### Run on save

```bash
cargo watch -x test                # Run `cargo test` when files change.
cargo watch -x 'run -- --some-arg' # Run `cargo run -- --some-arg` when files change.
```

You can also run arbitrary commands on your own system as you develop, for example with:

```shell
# Run on file change.
fd | entr -s 'cargo +nightly fmt && RUST_BACKTRACE=1 cargo +nightly r -- --log-level=trace'

# In another window, replace with your command logging output.
less ~/tmp/dot-tmp.log
```

### Command Aliases

You can also define command aliases as in git, for an example see [my dotfiles][cargo config].

## Managing Dependencies

To add a prod dependency (compiled into the final binary) use:

```bash
# `cargo add --help` for more info.
cargo add <dependency>    # Add a dependency (compiled into the binary).
cargo add -D <dependency> # Add a dev dependency (only used in tests).
```

To update the ranges in the `Cargo.toml`:

```bash
cargo upgrade
```

Update pinned versions in `Cargo.lock` to the latest versions matching the ranges in `Cargo.toml`:

```bash
cargo update
```

## Writing doc comments

Useful links:
- [Rust By Example][Documentation - Rust By Example]
- [The Book][Documentation - The Book]
- [The API Reference][Documentation - API Reference]
- [RFC 1574: API Documentation Conventions][]
- [RFC 1946: intra-rustdoc links][]
- [Documentation - Reddit][]

## Commit Messages

This project uses [Conventional Commit messages][], with the following categories:

- build: Changes that affect the build system or external dependencies (example scopes: gulp, broccoli, npm)
- chore: (updating grunt tasks etc; no production code change)
- ci: Changes to our CI configuration files and scripts (example scopes: Travis, Circle, BrowserStack, SauceLabs)
- docs: Documentation only changes
- feat: A new feature
- fix: A bug fix
- perf: A code change that improves performance
- refactor: A code change that neither fixes a bug nor adds a feature
- revert: revert of a previous commit
- style: Changes that do not affect the meaning of the code (white-space, formatting, missing semi-colons, etc)
- test: Adding missing tests or correcting existing tests

[CONTRIBUTING.md]: /docs/CONTRIBUTING.md
[Conventional Commit messages]: https://www.conventionalcommits.org/en/v1.0.0-beta.4/
[Documentation - API Reference]: https://doc.rust-lang.org/stable/reference/comments.html#doc-comments
[Documentation - Reddit]: https://www.reddit.com/r/rust/comments/ahb50s/is_there_any_documentation_style_guide_for/
[Documentation - Rust By Example]: https://doc.rust-lang.org/rust-by-example/meta/doc.html
[Documentation - The Book]: https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments
[Language Server Protocol]: https://langserver.org/
[RFC 1574: API Documentation Conventions]: https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html#appendix-a-full-conventions-text
[RFC 1946: intra-rustdoc links]: https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html
[Rust Barebones]: https://anderspitman.net/blog/rust-docker-barebones/
[`cargo-edit`]: https://github.com/killercup/cargo-edit
[`cargo-watch`]: https://github.com/passcod/cargo-watch
[`clippy`]: https://github.com/rust-lang/rust-clippy
[`rls`]: https://github.com/rust-lang/rls
[`rustfmt`]: https://github.com/rust-lang/rustfmt
[cargo config]: https://github.com/gibfahn/dot/blob/master/dotfiles/.local/share/cargo/config
[rustup]: https://rustup.rs/
